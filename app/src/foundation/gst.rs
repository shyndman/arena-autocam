use anyhow::{Error, Result};
use glib::EnumValue;
use gst::{prelude::ElementExtManual, traits::GstObjectExt};

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
