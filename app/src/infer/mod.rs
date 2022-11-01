mod detection_overlay;
mod detection_sink;
mod tf_buffer_adapter;

pub use detection_overlay::*;
pub use detection_sink::*;
use once_cell::sync::Lazy;

pub(crate) static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_INFER",
        gst::DebugColorFlags::empty(),
        Some("Auto-Arena Inference"),
    )
});
