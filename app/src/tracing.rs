use glib::{Cast, ToValue};
use gst::{traits::GstObjectExt, Element};

use crate::util;

/// Traces graph state as a .dot file on a `gst::message::StateChanged` message, if it
/// represents a pipeline change (rather than the change to an individual element).
///
/// Only writes if we're in a debug configuration — otherwise this is a noop.
pub(crate) fn trace_graph_state_change(
    pipeline: &gst::Pipeline,
    state_change_details: &gst::message::StateChanged,
) {
    #[cfg(debug_assertions)]
    {
        let event_src = &state_change_details.src().unwrap();
        if event_src == pipeline {
            let state_value = state_change_details.current().to_value();
            let state_name = util::name_of_enum_value(&state_value)
                .unwrap()
                .to_lowercase();
            let element = event_src.dynamic_cast_ref::<Element>().unwrap();
            gst::debug_bin_to_dot_file_with_ts(
                pipeline,
                gst::DebugGraphDetails::all(),
                format!("client-{}-{}", element.name(), state_name),
            );
        }
    }
}
