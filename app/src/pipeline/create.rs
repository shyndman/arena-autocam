use anyhow::Error;
use gst::prelude::*;

use super::camera::{setup_camera_src, CameraSrcInfo};
use super::PIPE_CAT;
use crate::{
    infer::{build_detection_overlay, DetectionSink},
    logging::*,
    util::find_sink_pad,
};

pub fn create_pipeline() -> Result<(glib::MainLoop, gst::Pipeline), Error> {
    gst::init()?;
    gst::update_registry()?;

    info!(PIPE_CAT, "Creating Arena-Autocam pipeline");

    // Build a main loop, which will allow us to send/receive bus messages asynchronous
    let main_loop = glib::MainLoop::new(None, false);

    // Build the pipeline
    let pipeline = gst::Pipeline::default();
    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    let CameraSrcInfo {
        display_stream_src_pad,
        inference_stream_src_pad,
    } = setup_camera_src(&pipeline)?;
    create_display_stream_pipeline(&pipeline, &bus, &display_stream_src_pad)?;
    create_infer_stream_pipeline(&pipeline, &bus, &inference_stream_src_pad)?;

    Ok((main_loop, pipeline))
}

fn create_display_stream_pipeline(
    pipeline: &gst::Pipeline,
    bus: &gst::Bus,
    display_stream_src: &gst::Pad,
) -> Result<(), Error> {
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
) -> Result<(), Error> {
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
        .name("display.persist.multifile_sink")
        .property("max-size-time", 10.minutes().nseconds())
        .property("muxer-factory", "matroskamux")
        .property("location", "video%05d.mp4")
        .build()?;

    pipeline.add_many(&[
        &encode_queue,
        &encoder,
        &caps,
        &h264parse,
        &chunk_file_writer,
        // &fakesink,
    ])?;
    src_pad.link(&find_sink_pad(&encode_queue)?)?;
    gst::Element::link_many(&[
        &encode_queue,
        &encoder,
        &caps,
        &h264parse,
        &chunk_file_writer,
        // &fakesink,
    ])?;

    Ok(())
}

/// Creates the pipeline branch that
fn create_display_stream_debug_branch(
    pipeline: &gst::Pipeline,
    bus: &gst::Bus,
    src_pad: gst::Pad,
) -> Result<(), Error> {
    let queue = gst::ElementFactory::make("queue")
        .name("display.debug.queue")
        .property("max-size-time", 10.seconds().nseconds() as u64)
        .property("max-size-bytes", 0u32) // disable
        .property("max-size-buffers", 0u32) // disable
        .build()?;
    let overlay_convert1 = gst::ElementFactory::make("videoconvert")
        .name("display.debug.overlay_convert_in")
        .build()?;
    let overlay_display = build_detection_overlay("display.debug.overlay", bus)?;
    let overlay_convert2 = gst::ElementFactory::make("videoconvert")
        .name("display.debug.overlay_convert_out")
        .build()?;
    let sink_display = gst::ElementFactory::make("autovideosink")
        .name("display.debug.autovideosink")
        .build()?;
    pipeline.add_many(&[
        &queue,
        &overlay_convert1,
        &overlay_display,
        &overlay_convert2,
        &sink_display,
    ])?;

    src_pad.link(&find_sink_pad(&queue)?)?;
    gst::Element::link_many(&[
        &queue,
        &overlay_convert1,
        &overlay_display,
        &overlay_convert2,
        &sink_display,
    ])?;

    Ok(())
}

fn create_infer_stream_pipeline(
    pipeline: &gst::Pipeline,
    bus: &gst::Bus,
    infer_src_pad: &gst::Pad,
) -> Result<(), Error> {
    let infer_leaky_queue = gst::ElementFactory::make("queue")
        .name("infer.leaky-queue")
        .property_from_str("leaky", "downstream")
        .property("max-size-buffers", 1u32)
        .build()?;

    let infer_detection_sink = DetectionSink::new(Some("infer.detection_sink"));
    infer_detection_sink.set_property("bus", &bus);

    pipeline.add(&infer_leaky_queue)?;
    pipeline.add(&infer_detection_sink)?;

    infer_src_pad.link(&find_sink_pad(&infer_leaky_queue)?)?;
    infer_leaky_queue.link(&infer_detection_sink)?;

    Ok(())
}
