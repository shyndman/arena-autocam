mod context;
mod format_event;
mod format_fields;
mod level;
mod target;
mod thread;

pub use format_event::*;
pub use format_fields::*;
use palette::Srgba;

pub fn to_ansi_color(from: Srgba<u8>) -> nu_ansi_term::Color {
    nu_ansi_term::Color::Rgb(from.red, from.green, from.blue)
}
