use anyhow::Error;
use glib::{EnumClass, Type};
use gst::{prelude::*, Caps, Fraction};
use gst_video::VideoCapsBuilder;

use crate::util::find_src_pad;

pub(super) struct CameraSrcInfo {
    pub display_stream_src_pad: gst::Pad,
    pub inference_stream_src_pad: gst::Pad,
}

pub(super) fn setup_camera_src<'pipeline>(
    pipeline: &'pipeline gst::Pipeline,
) -> Result<CameraSrcInfo, Error> {
    if cfg!(feature = "synthesize-libcamera-streams") {
        synthesize_libcamera_streams(pipeline)
    } else {
        build_libcamera_streams(pipeline)
    }
}

fn build_libcamera_streams(pipeline: &gst::Pipeline) -> Result<CameraSrcInfo, Error> {
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
    Ok(CameraSrcInfo {
        display_stream_src_pad: display_pad,
        inference_stream_src_pad: inference_pad,
    })
}

fn synthesize_libcamera_streams(pipeline: &gst::Pipeline) -> Result<CameraSrcInfo, Error> {
    let raw_jpeg_caps = Caps::builder("image/jpeg")
        .field("width", 1280)
        .field("height", 720)
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
            .width(1280)
            .height(720)
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
    // let video_convert = gst::ElementFactory::make("videoconvert").build()?;
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

    Ok(CameraSrcInfo {
        display_stream_src_pad: find_src_pad(&queue_display)?,
        inference_stream_src_pad: find_src_pad(&caps_filter_infer)?,
    })
}
