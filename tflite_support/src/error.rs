use std::fmt::Display;

use super::bindings::TfLiteSupportErrorCode;

/// The error type for TensorFlow Lite operations.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Error {
    // TODO(shyndman): Have this mirror the support error type
    pub code: TfLiteSupportErrorCode,
    pub message: String,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "An error occurred in Tensorflow lite ({:?}). {}",
            self.code, self.message
        )
    }
}
