use glib::prelude::*;
use gst::traits::*;

use super::CAT;
use crate::foundation::gst::name_of_enum_value;
use crate::logging::*;

/// Traces graph state as a .dot file on a `gst::message::StateChanged` message, if it
/// represents a pipeline change (rather than the change to an individual element).
///
/// Only writes if we're in a debug configuration — otherwise this is a noop.
#[cfg(debug_assertions)]
pub(crate) fn trace_graph_state_change(
    pipeline: &gst::Pipeline,
    state_change_details: &gst::message::StateChanged,
) {
    let event_src = &state_change_details.src().unwrap();
    let state_value = state_change_details.current().to_value();
    let state_name = name_of_enum_value(&state_value).unwrap().to_lowercase();
    let element = event_src.dynamic_cast_ref::<gst::Element>().unwrap();
    trace_graph(pipeline, element.name().into(), state_name);
}

#[cfg(not(debug_assertions))]
pub(crate) fn trace_graph_state_change(
    _pipeline: &gst::Pipeline,
    _state_change_details: &gst::message::StateChanged,
) {
}

/// Traces graph state as a .dot file on a `gst::message::StateChanged` message, if it
/// represents a pipeline change (rather than the change to an individual element).
///
/// Only writes if we're in a debug configuration — otherwise this is a noop.
#[cfg(debug_assertions)]
pub(crate) fn trace_graph(pipeline: &gst::Pipeline, src_name: String, state_name: String) {
    info!(
        CAT,
        "Dumping graph state, name=client-{}-{}", src_name, state_name
    );
    gst::debug_bin_to_dot_file_with_ts(
        pipeline,
        gst::DebugGraphDetails::all(),
        format!("client-{}-{}", src_name, state_name),
    );
}

#[cfg(not(debug_assertions))]
pub(crate) fn trace_graph(_pipeline: &gst::Pipeline, _src_name: String, _state_name: String) {
}
