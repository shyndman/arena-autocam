extern crate tracing;

pub mod fmt;
pub mod category;
mod subscriber;

pub use subscriber::*;

pub mod base_macros {
    pub use tracing::{debug, error, info, trace, warn as warning};
}

pub mod valuable_crate {
    pub use valuable::*;
}
