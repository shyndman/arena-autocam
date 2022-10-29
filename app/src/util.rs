use anyhow::Error;
use cairo::Rectangle;
use glib::EnumValue;
use gst::{prelude::ElementExtManual, traits::GstObjectExt};
use rand::Rng;

/// Returns the name of a GLib enum value.
pub fn name_of_enum_value(value: &glib::Value) -> Option<&str> {
    let enum_value = EnumValue::from_value(value)?;
    Some(enum_value.1.name())
}

/// Finds and returns an element's default src pad.
pub fn find_src_pad(element: &gst::Element) -> Result<gst::Pad, Error> {
    element
        .src_pads()
        .iter()
        .filter(|pad| GstObjectExt::name(*pad) == "src")
        .next()
        .map(|pad_ref| pad_ref.to_owned())
        .ok_or(Error::msg("No sink pad found"))
}

/// Finds and returns an element's sink pad.
pub fn find_sink_pad(element: &gst::Element) -> Result<gst::Pad, Error> {
    element
        .sink_pads()
        .first()
        .map(|pad_ref| pad_ref.to_owned())
        .ok_or(Error::msg("No sink pad found"))
}

pub fn random_fraction_rect() -> Rectangle {
    let mut r = rand::thread_rng();
    let w = r.gen_range(0.0_f64..=1.0).clamp(0.03, 0.3);
    let x = r.gen_range(0.0..(1.0 - w));
    let h = r.gen_range(0.0_f64..=1.0).clamp(0.03, 0.3);
    let y = r.gen_range(0.0..(1.0 - h));
    Rectangle::new(x, y, w, h)
}
