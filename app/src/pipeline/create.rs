use anyhow::{Error, Result};
use gst::prelude::*;
use gst_app::prelude::BaseSinkExt;

use super::source::{create_media_sources, SourcePads};
use super::{names, CREATE_CAT as CAT};
use crate::config::Config;
use crate::{
    foundation::gst::find_sink_pad,
    infer::{build_detection_overlay, DetectionSink},
    logging::*,
};

pub fn create_pipeline(config: &Config) -> Result<(glib::MainLoop, gst::Pipeline)> {
    gst::init()?;
    gst::update_registry()?;

    info!(CAT, "Creating Arena-Autocam pipeline");

    // Build a main loop, which will allow us to send/receive bus messages asynchronous
    let main_loop = glib::MainLoop::new(None, false);

    // Build the pipeline
    let pipeline = gst::Pipeline::default();
    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    let SourcePads {
        display_stream_src_pad,
        infer_stream_src_pad,
    } = create_media_sources(config, &pipeline)?;

    // The multiqueue allows us to replace all other queues, and is responsible for
    // ensuring that the encoder's heavy up-front frame requests (so it can determine how
    // to properly encode) does not impact streams that are displayed in near real-time.
    let queue = gst::ElementFactory::make("multiqueue")
        .name("display/infer.multiqueue")
        .property("max-size-bytes", 1024u32 * 1024 * 100)
        .property("max-size-buffers", 0u32) // Disabled
        .property("max-size-time", 0u64) // Disabled
        .property("max-size-bytes", 0u32) // Disabled
        .build()?;
    pipeline.add(&queue)?;

    let display_sink_pad = queue.request_pad_simple("sink_%u").unwrap();
    display_stream_src_pad.link(&display_sink_pad)?;
    let display_src_pad = display_sink_pad.iterate_internal_links().next()?.unwrap();

    let infer_sink_pad = queue.request_pad_simple("sink_%u").unwrap();
    infer_stream_src_pad.link(&infer_sink_pad)?;
    let infer_src_pad = infer_sink_pad.iterate_internal_links().next()?.unwrap();

    create_display_stream_pipeline(&pipeline, &bus, &display_src_pad, config)?;
    create_infer_stream_pipeline(&pipeline, &bus, &infer_src_pad, config)?;

    Ok((main_loop, pipeline))
}

fn create_display_stream_pipeline(
    pipeline: &gst::Pipeline,
    bus: &gst::Bus,
    display_stream_src: &gst::Pad,
    config: &Config,
) -> Result<()> {
    // Splits the display input to the (optional) debug pipeline and the filesystem
    // pipeline.
    let display_splitter = gst::ElementFactory::make("tee")
        .name("display.splitter")
        .build()?;
    pipeline.add(&display_splitter)?;
    display_stream_src.link(&find_sink_pad(&display_splitter)?)?;

    let splitter_src_tmpl = display_splitter
        .pad_template("src_%u")
        .expect("No src template found on tee");

    create_display_stream_debug_branch(
        pipeline,
        bus,
        display_splitter
            .request_pad(&splitter_src_tmpl, None, None)
            .unwrap(),
        config,
    )?;
    create_display_stream_persistence_branch(
        pipeline,
        bus,
        display_splitter
            .request_pad(&splitter_src_tmpl, None, None)
            .unwrap(),
    )?;

    Ok(())
}

/// Creates the pipeline branch that consumes the display stream, encodes it, and writes
/// it in chunks to the filesystem.
fn create_display_stream_persistence_branch(
    pipeline: &gst::Pipeline,
    _bus: &gst::Bus,
    src_pad: gst::Pad,
) -> Result<()> {
    let encode_queue = gst::ElementFactory::make("queue")
        .name("display.persist.encoder.queue")
        .build()?;

    let encoder = if cfg!(feature = "use-v4l2-h264-encoding") {
        gst::ElementFactory::make("v4l2h264enc")
            .name("display.persist.encoder")
            .build()?
    } else {
        gst::ElementFactory::make("nvh264enc")
            .name("display.persist.encoder")
            .property_from_str("preset", "low-latency-hq")
            .build()?
    };

    let caps = gst::ElementFactory::make("capsfilter")
        .name("display.persist.encoder.out.caps")
        .property_from_str("caps", "video/x-h264,level=(string)4")
        .build()?;
    let h264parse = gst::ElementFactory::make("h264parse")
        .name("display.persist.parse_encoded")
        // Ensure that we're sending PTS/DTS with each IDR frame
        .property("config-interval", -1)
        .build()?;
    let chunk_file_writer = gst::ElementFactory::make("splitmuxsink")
        .name(names::PERSISTENCE_SINK)
        .property("max-size-time", 10.minutes().nseconds())
        .property("muxer-factory", "matroskamux")
        .property("location", "video%05d.mp4")
        .property("async-handling", true)
        .build()?;

    pipeline.add_many(&[
        &encode_queue,
        &encoder,
        &caps,
        &h264parse,
        &chunk_file_writer,
    ])?;
    src_pad.link(&find_sink_pad(&encode_queue)?)?;
    gst::Element::link_many(&[
        &encode_queue,
        &encoder,
        &caps,
        &h264parse,
        &chunk_file_writer,
    ])?;

    Ok(())
}

/// Creates the pipeline branch that
fn create_display_stream_debug_branch(
    pipeline: &gst::Pipeline,
    bus: &gst::Bus,
    src_pad: gst::Pad,
    config: &Config,
) -> Result<()> {
    let overlay_convert1 = gst::ElementFactory::make("videoconvert")
        .name("display.debug.overlay_convert_in")
        .build()?;
    let overlay_display = build_detection_overlay("display.debug.overlay", bus, config)?;
    let overlay_convert2 = gst::ElementFactory::make("videoconvert")
        .name("display.debug.overlay_convert_out")
        .build()?;
    let sink_display = gst::ElementFactory::make("autovideosink")
        .name("display.debug.autovideosink")
        .build()?;

    let elements = &[
        &overlay_convert1,
        &overlay_display,
        &overlay_convert2,
        &sink_display,
    ];
    pipeline.add_many(elements)?;
    src_pad.link(&find_sink_pad(&overlay_convert1)?)?;
    gst::Element::link_many(elements)?;

    let duration = config.inference.inference_frame_duration();
    sink_display.connect("element-added", true, move |args| {
        let (sink, child) = if let [sink, child, ..] = args {
            (
                sink.get::<gst::Element>().unwrap(),
                child.get::<gst::Element>().unwrap(),
            )
        } else {
            return None;
        };

        debug!(CAT, obj: &sink, "Child added, {:?}", child);
        if let Ok(sink) = child.dynamic_cast::<gst_base::BaseSink>() {
            sink.set_ts_offset(duration.num_nanoseconds().unwrap());
        }

        None
    });

    Ok(())
}

fn create_infer_stream_pipeline(
    pipeline: &gst::Pipeline,
    bus: &gst::Bus,
    infer_src_pad: &gst::Pad,
    config: &Config,
) -> Result<()> {
    let infer_rate = gst::ElementFactory::make("videorate").build()?;
    let infer_framerate_caps = gst::ElementFactory::make("capsfilter")
        .property(
            "caps",
            gst_video::VideoCapsBuilder::new()
                .framerate(
                    gst::Fraction::approximate_f32(config.inference.rate_per_second).unwrap(),
                )
                .build(),
        )
        .build()?;
    let infer_detection_sink = DetectionSink::new(Some(names::DETECTION_SINK))
        .dynamic_cast::<gst::Element>()
        .unwrap();
    infer_detection_sink.set_property("bus", &bus);
    let elements = &[&infer_rate, &infer_framerate_caps, &infer_detection_sink];
    pipeline.add_many(elements)?;

    infer_src_pad.link(&find_sink_pad(&infer_rate)?)?;
    gst::Element::link_many(elements)?;

    Ok(())
}
