use anyhow::Result;
use gst::prelude::*;

use super::RUN_CAT;
use crate::{logging::*, tracing::trace_graph_state_change};

pub fn run_main_loop((main_loop, pipeline): (glib::MainLoop, gst::Pipeline)) -> Result<()> {
    info!(RUN_CAT, obj: &pipeline, "Starting main loop");

    pipeline.set_state(gst::State::Playing)?;

    let main_loop_clone = main_loop.clone();
    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");
    let pipeline_weak = pipeline.downgrade();
    bus.connect_message(None, move |_, msg| {
        debug!(RUN_CAT, "Received message in main loop, {:?}", msg.type_());

        use gst::MessageView;
        let main_loop = &main_loop_clone;

        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None => {
                main_loop.quit();
                return;
            } //return glib::Continue(true),
        };

        match msg.view() {
            MessageView::Eos(..) => {
                // end-of-stream
                let _ = pipeline.set_state(gst::State::Ready);
                main_loop.quit();
            }
            MessageView::Error(err) => {
                error!(
                    RUN_CAT,
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                let _ = pipeline.set_state(gst::State::Ready);
                main_loop.quit();
            }
            MessageView::StateChanged(event_details) => {
                trace_graph_state_change(&pipeline, &event_details);
            }
            MessageView::Latency(latency) => {
                info!(RUN_CAT, "{:?}", latency);
            }
            _ => (),
        }
        // glib::Continue(true)
    });

    bus.add_signal_watch();
    main_loop.run();

    bus.remove_watch()?;
    pipeline.set_state(gst::State::Null)?;

    Ok(())
}
