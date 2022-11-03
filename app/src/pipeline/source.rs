use std::path::PathBuf;

use anyhow::Result;
use glib::value::FromValue;
use glib::{EnumClass, Type};
use gst::{element_warning, prelude::*, Caps, Fraction};
use gst_app::prelude::BaseSrcExt;
use gst_video::VideoCapsBuilder;

use crate::config::Config;
use crate::foundation::{gst::find_src_pad, path::to_canonicalized_path_string};
use crate::logging::*;
use crate::pipeline::CREATE_CAT as CAT;

pub(super) struct SourcePads {
    pub display_stream_src_pad: gst::Pad,
    pub infer_stream_src_pad: gst::Pad,
}

pub(super) fn create_media_sources(
    config: &Config,
    pipeline: &gst::Pipeline,
) -> Result<SourcePads> {
    if let Some(ref video_path) = config.source.debug_source_video_path {
        setup_debug_video_sources(PathBuf::from(video_path), config, pipeline)
    } else {
        setup_camera_sources(config, pipeline)
    }
}

fn setup_camera_sources(config: &Config, pipeline: &gst::Pipeline) -> Result<SourcePads> {
    if cfg!(feature = "synthesize-libcamera-streams") {
        synthesize_libcamera_streams(config, pipeline)
    } else {
        build_libcamera_streams(pipeline)
    }
}

fn build_libcamera_streams(pipeline: &gst::Pipeline) -> Result<SourcePads> {
    info!(CAT, "Creating libcamera source");

    let camera = gst::ElementFactory::make("libcamerasrc").build()?;
    pipeline.add(&camera)?;
    // This MUST follow the ElementFactory line, to ensure that the plugin is
    // loaded
    let stream_role_enum =
        EnumClass::new(Type::from_name("GstLibcameraStreamRole").unwrap()).unwrap();
    // Build pads
    let pad_template = camera.pad_template("src_%u").unwrap();
    // Build inference pad
    let inference_pad = camera.request_pad(&pad_template, None, None).unwrap();
    inference_pad.set_property(
        "stream-role",
        stream_role_enum.to_value_by_nick("view-finder").unwrap(),
    );
    // Build the display pad
    let display_pad = camera.request_pad(&pad_template, None, None).unwrap();
    display_pad.set_property(
        "stream-role",
        stream_role_enum
            .to_value_by_nick("video-recording")
            .unwrap(),
    );
    Ok(SourcePads {
        display_stream_src_pad: display_pad,
        infer_stream_src_pad: inference_pad,
    })
}

fn synthesize_libcamera_streams(
    config: &Config,
    pipeline: &gst::Pipeline,
) -> Result<SourcePads> {
    info!(CAT, "Creating synthetic libcamera source");

    let raw_jpeg_caps = Caps::builder("image/jpeg")
        .field("width", config.source.record_stream_width)
        .field("height", config.source.record_stream_height)
        .field("framerate", Fraction::new(30, 1))
        .build();

    let camera = gst::ElementFactory::make("libcamerasrc")
        .name("camera.src")
        .build()?;
    let decode_jpeg = gst::ElementFactory::make("jpegdec").build()?;
    let splitter = gst::ElementFactory::make("tee")
        .name("camera.tee")
        .build()?;

    pipeline.add_many(&[&camera, &decode_jpeg, &splitter])?;
    camera.link_pads_filtered(Some("src"), &decode_jpeg, Some("sink"), &raw_jpeg_caps)?;
    decode_jpeg.link_filtered(
        &splitter,
        &VideoCapsBuilder::new()
            .format(gst_video::VideoFormat::I420)
            .width(config.source.record_stream_width)
            .height(config.source.record_stream_height)
            .framerate(Fraction::new(30, 1))
            .build(),
    )?;

    // Build pads
    let pad_template = splitter
        .pad_template("src_%u")
        .expect("No src template found on tee");

    // Build up the inference mini-pipeline
    let splitter_infer_src = splitter.request_pad(&pad_template, None, None).unwrap();
    let queue_infer = gst::ElementFactory::make("queue")
        .name("camera.infer-branch.queue")
        .build()?;

    // Scale/convert the video for inference
    let video_scale_infer = gst::ElementFactory::make("videoscale")
        .name("camera.infer-branch.videoscale")
        .property("add-borders", false)
        .property_from_str("method", "0")
        .build()?;
    let caps_filter_infer = gst::ElementFactory::make("capsfilter")
        .name("camera.infer-branch.caps")
        .property_from_str(
            "caps",
            "video/x-raw,format=I420,width=224,height=224,pixel-aspect-ratio=(fraction)4/3",
        )
        .build()?;

    // Assemble pipeline
    pipeline.add_many(&[&queue_infer, &video_scale_infer, &caps_filter_infer])?;
    splitter.link_pads(Some(&splitter_infer_src.name()), &queue_infer, None)?;
    gst::Element::link_many(&[&queue_infer, &video_scale_infer, &caps_filter_infer])?;

    // Request a src pad for its generated name
    let splitter_display_src = splitter.request_pad(&pad_template, None, None).unwrap();
    let queue_display = gst::ElementFactory::make("queue")
        .name("camera.display.queue")
        .build()?;
    pipeline.add(&queue_display)?;
    splitter.link_pads(Some(&splitter_display_src.name()), &queue_display, None)?;

    Ok(SourcePads {
        display_stream_src_pad: find_src_pad(&queue_display)?,
        infer_stream_src_pad: find_src_pad(&caps_filter_infer)?,
    })
}

fn setup_debug_video_sources(
    video_path: PathBuf,
    config: &Config,
    pipeline: &gst::Pipeline,
) -> Result<SourcePads> {
    let video_uri = format!("pushfile://{}", to_canonicalized_path_string(&video_path)?);
    info!(CAT, "Creating debug video source, location={}", video_uri);

    let decodebin = gst::ElementFactory::make("uridecodebin")
        .name("debug-video.decode")
        .property("uri", video_uri)
        .build()?;
    let convert1 = gst::ElementFactory::make("videoconvert")
        .name("debug-video.convert-to-frames")
        .build()?;
    let scale = gst::ElementFactory::make("videoscale")
        .name("debug-video.scale-frames")
        .build()?;
    let rate = gst::ElementFactory::make("videorate")
        .name("debug-video.change-rate")
        .build()?;
    let caps = gst::ElementFactory::make("capsfilter")
        .name("debug-video.raw_caps")
        .property(
            "caps",
            &VideoCapsBuilder::new()
                .format(gst_video::VideoFormat::I420)
                .width(config.source.record_stream_width)
                .height(config.source.record_stream_height)
                .build(),
        )
        .build()?;
    let splitter = gst::ElementFactory::make("tee")
        .name("debug-video.tee")
        .build()?;

    pipeline.add_many(&[&decodebin, &convert1, &scale, &rate, &caps, &splitter])?;
    gst::Element::link_many(&[&convert1, &scale, &rate, &caps, &splitter])?;

    // Build pads
    let pad_template = splitter
        .pad_template("src_%u")
        .expect("No src template found on tee");

    // Build up the inference mini-pipeline
    let splitter_infer_src = splitter.request_pad(&pad_template, None, None).unwrap();
    let queue_infer = gst::ElementFactory::make("queue")
        .name("debug-video.infer-branch.queue")
        .build()?;

    // Scale/convert the video for inference
    let video_scale_infer = gst::ElementFactory::make("videoscale")
        .name("debug-video.infer-branch.videoscale")
        .property("add-borders", false)
        .property_from_str("method", "0")
        .build()?;
    let caps_filter_infer = gst::ElementFactory::make("capsfilter")
        .name("debug-video.infer-branch.caps")
        .property(
            "caps",
            &VideoCapsBuilder::new()
                .format(gst_video::VideoFormat::I420)
                .width(config.source.infer_stream_width)
                .height(config.source.infer_stream_height)
                .pixel_aspect_ratio(Fraction::new(
                    config.source.infer_stream_width,
                    config.source.infer_stream_height,
                ))
                .build(),
        )
        .build()?;

    // Assemble pipeline
    pipeline.add_many(&[&queue_infer, &video_scale_infer, &caps_filter_infer])?;
    splitter.link_pads(Some(&splitter_infer_src.name()), &queue_infer, None)?;
    gst::Element::link_many(&[&queue_infer, &video_scale_infer, &caps_filter_infer])?;

    // Request a src pad for its generated name
    let splitter_display_src = splitter.request_pad(&pad_template, None, None).unwrap();
    let queue_display = gst::ElementFactory::make("queue")
        .name("debug-video.display.queue")
        .build()?;
    pipeline.add(&queue_display)?;
    splitter.link_pads(Some(&splitter_display_src.name()), &queue_display, None)?;

    // The decodebin takes some time to figure out how to decode the input that we've
    // provided it, so we register a signal handler, then spin until we receive indication
    // that we should continue building the pipeline.
    decodebin.connect_pad_added(move |decodebin: &gst::Element, src_pad: &gst::Pad| {
        let sink_pad = convert1
            .static_pad("sink")
            .expect("Failed to get static sink pad from convert");
        if sink_pad.is_linked() {
            println!("We are already linked. Ignoring.");
            return;
        }

        // Try to detect whether the raw stream decodebin provided us with
        // just now is either audio or video (or none of both, e.g. subtitles).
        let (_is_audio, is_video) = {
            let media_type = src_pad.current_caps().and_then(|caps| {
                caps.structure(0).map(|s| {
                    let name = s.name();
                    (name.starts_with("audio/"), name.starts_with("video/"))
                })
            });

            if let Some(media_type) = media_type {
                media_type
            } else {
                element_warning!(
                    decodebin,
                    gst::CoreError::Negotiation,
                    ("Failed to get media type from pad {}", src_pad.name())
                );
                return;
            }
        };

        if is_video {
            src_pad.link(&sink_pad).expect("Unable to link");
        }
    });

    decodebin.connect("source-setup", false, |args| {
        if let [_, src, ..] = args {
            // src.
            let src = unsafe { gst::Bin::from_value(src) };
            let filesrc = src
                .child_by_index(0)
                .unwrap()
                .dynamic_cast::<gst_base::BaseSrc>()
                .unwrap();
            filesrc.set_live(true);
        }

        None
    });

    Ok(SourcePads {
        display_stream_src_pad: find_src_pad(&queue_display)?,
        infer_stream_src_pad: find_src_pad(&caps_filter_infer)?,
    })
}
