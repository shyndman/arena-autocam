extern crate tracing;

pub mod category;
pub mod fmt;
mod subscriber;

pub use subscriber::*;

pub mod base_macros {
    pub use tracing::{debug, error, info, trace, warn as warning};
}

pub mod valuable_crate {
    pub use valuable::*;
}
