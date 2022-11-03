use anyhow::Result;
use gst::prelude::*;

use super::RUN_CAT as CAT;
use crate::{foundation::debug::trace_graph_state_change, logging::*};

pub fn run_main_loop((main_loop, pipeline): (glib::MainLoop, gst::Pipeline)) -> Result<()> {
    info!(CAT, obj: &pipeline, "Starting main loop");

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    let pipeline_element = pipeline.dynamic_cast_ref::<gst::Element>().unwrap();
    pipeline_element.set_state(gst::State::Playing)?;

    let main_loop_clone = main_loop.clone();

    debug!(CAT, "Registering message bus observer");
    let pipeline_weak = pipeline.downgrade();
    bus.connect_message(None, move |_, msg| {
        debug!(
            CAT,
            "Received message in main loop, {}",
            format!("{:?}", msg.type_()).to_lowercase()
        );

        use gst::MessageView;
        let main_loop = &main_loop_clone;

        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None => {
                main_loop.quit();
                return;
            }
        };

        let view = msg.view();
        match view {
            MessageView::Eos(..) => {
                // end-of-stream
                let _ = pipeline.set_state(gst::State::Ready);
                main_loop.quit();
            }
            MessageView::Error(err) => {
                // TODO(shyndman): Figure out how to judge the severity of the error
                error!(
                    CAT,
                    "Error from {:?}: {:?} {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error().domain(),
                    err.error(),
                    err.debug()
                );

                let _ = pipeline.set_state(gst::State::Ready);
            }
            MessageView::StateChanged(event_details) => {
                trace_graph_state_change(&pipeline, &event_details);
            }
            MessageView::Application(_) => {}
            _ => {
                let msg_ref = msg.as_ref() as &gst::MessageRef;
                if let Some(structure) = msg_ref.structure() {
                    log!(CAT, "Message contents {:#?}", structure);
                }
            }
        }
    });

    bus.add_signal_watch();
    main_loop.run();
    pipeline.set_state(gst::State::Null)?;

    Ok(())
}
