use anyhow::{anyhow, Error, Result};
use glib::EnumValue;
use gst::{
    prelude::ElementExtManual,
    traits::{ElementExt, GstObjectExt},
    StateChangeSuccess,
};

use super::CAT;
use crate::logging::*;

/// Returns the name of a GLib enum value.
pub fn name_of_enum_value(value: &glib::Value) -> Option<&str> {
    let enum_value = EnumValue::from_value(value)?;
    Some(enum_value.1.name())
}

/// Finds and returns an element's default src pad.
pub fn find_src_pad(element: &gst::Element) -> Result<gst::Pad> {
    element
        .src_pads()
        .iter()
        .filter(|pad| GstObjectExt::name(*pad) == "src")
        .next()
        .map(|pad_ref| pad_ref.to_owned())
        .ok_or(Error::msg("No sink pad found"))
}

/// Finds and returns an element's sink pad.
pub fn find_sink_pad(element: &gst::Element) -> Result<gst::Pad> {
    element
        .sink_pads()
        .first()
        .map(|pad_ref| pad_ref.to_owned())
        .ok_or(Error::msg("No sink pad found"))
}

pub fn request_state_and_wait_for_change(
    element: &gst::Element,
    state: gst::State,
) -> Result<()> {
    debug!(CAT, obj: element, "Requesting state change to {:?}", state);
    match element.set_state(state) {
        Ok(change_success_type) => {
            if change_success_type == StateChangeSuccess::Async {
                let res = element.state(None);
                eprintln!("!!!! {:?}", res);
                res.0?;
            }
            Ok(())
        }
        Err(e) => Result::Err(anyhow!(e)),
    }
}
