extern crate tracing;

pub mod fmt;
mod macros;
mod subscriber;

pub use subscriber::*;

pub mod base_macros {
    pub use tracing::{debug, error, info, trace, warn as warning};
}
