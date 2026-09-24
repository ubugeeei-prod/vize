//! Error type shared by every `vize lib` subcommand.

use std::fmt;
use std::path::Path;

use vize_s0::{String, cstr};

/// A user-facing `vize lib` failure. Every variant renders as one message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibError {
    message: String,
}

impl LibError {
    /// Create an error from a complete message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Wrap an I/O failure with the path it concerns.
    pub fn io(action: &str, path: &Path, error: &std::io::Error) -> Self {
        Self::new(cstr!("failed to {action} {}: {error}", path.display()))
    }

    /// The rendered message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for LibError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for LibError {}

/// Result alias for `vize lib`.
pub type LibResult<T> = Result<T, LibError>;
