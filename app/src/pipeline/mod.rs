mod configure;
mod create;
pub(self) mod names;
mod run;
pub(self) mod source;

pub use configure::*;
pub use create::*;
use once_cell::sync::Lazy;
pub use run::*;

pub(self) static CONFIGURE_CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_CONFIGURE",
        gst::DebugColorFlags::FG_WHITE | gst::DebugColorFlags::BG_BLUE,
        Some("Auto-Arena Configure"),
    )
});

pub(self) static CREATE_CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_CREATE",
        gst::DebugColorFlags::empty(),
        Some("Auto-Arena Creation"),
    )
});

pub(self) static RUN_CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "AA_RUN",
        gst::DebugColorFlags::FG_GREEN,
        Some("Auto-Arena Run"),
    )
});
