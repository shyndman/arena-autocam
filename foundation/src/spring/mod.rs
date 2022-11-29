mod config;
mod system;

pub use config::*;
pub use system::*;

use crate::trace_category;

trace_category!("spring", bg: crate::color::MaterialColorPalette::Red.primary());
