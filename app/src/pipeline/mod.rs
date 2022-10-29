mod camera;
mod create;
mod run;

pub use create::*;
use once_cell::sync::Lazy;
pub use run::*;

pub(self) static PIPE_CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_PIPE",
        gst::DebugColorFlags::empty(),
        Some("Auto-Arena Inference"),
    )
});

pub(self) static RUN_CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_RUN",
        gst::DebugColorFlags::FG_WHITE | gst::DebugColorFlags::BG_GREEN,
        Some("Auto-Arena Inference"),
    )
});
