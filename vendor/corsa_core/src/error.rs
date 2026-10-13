use crate::{
    RpcResponseError,
    fast::{CompactString, compact_format},
};
use std::{io, time::Duration};

/// Workspace-wide error type for process, transport, and protocol failures.
#[derive(Debug, thiserror::Error)]
pub enum CorsaError {
    /// Underlying OS or process I/O failure.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// JSON serialization or deserialization failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Base64 decoding failure for binary JSON payloads.
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    /// Error returned by the remote JSON-RPC peer.
    #[error("rpc error {}: {}", .0.code, .0.message)]
    Rpc(RpcResponseError),
    /// Protocol-level invariant violation or user-facing contract error.
    #[error("protocol error: {0}")]
    Protocol(CompactString),
    /// Message shape did not match what the transport expected.
    #[error("unexpected message: {0}")]
    UnexpectedMessage(CompactString),
    /// Opaque handle payload could not be parsed.
    #[error("invalid handle: {0}")]
    InvalidHandle(CompactString),
    /// Operation could not continue because the underlying resource is closed.
    #[error("process is closed: {0}")]
    Closed(&'static str),
    /// Requested feature or transport is not supported.
    #[error("unsupported: {0}")]
    Unsupported(&'static str),
    /// Thread/task join failure surfaced as a stable string.
    #[error("join error: {0}")]
    Join(CompactString),
    /// Operation did not finish before the configured deadline.
    #[error("timeout: {0}")]
    Timeout(CompactString),
}

/// Standard result alias used across the workspace.
pub type Result<T, E = CorsaError> = std::result::Result<T, E>;

impl CorsaError {
    /// Formats the error for command-line tools with a short recovery hint.
    pub fn diagnostic(&self) -> CompactString {
        let mut message = compact_format(format_args!("error: {self}"));
        if let Some(hint) = self.recovery_hint() {
            message.push_str("\nhelp: ");
            message.push_str(hint);
        }
        message
    }

    /// Returns a user-facing hint for broad classes of operational failures.
    pub fn recovery_hint(&self) -> Option<&'static str> {
        match self {
            Self::Io(_) => Some(
                "check that the referenced paths exist and that the current user has permission to read, write, and execute them",
            ),
            Self::Json(_) => {
                Some("check the JSON payload shape and ensure the peer is returning valid JSON")
            }
            Self::Base64(_) => Some("check that binary JSON fields are valid base64"),
            Self::Closed(_) => Some("retry after starting a fresh corsa process or client session"),
            Self::Unsupported(_) => {
                Some("use a compatible corsa build or disable the unsupported feature for this run")
            }
            Self::Join(_) => Some(
                "retry the operation; if it repeats, inspect the worker thread or child-process logs above this message",
            ),
            Self::Timeout(_) => Some(
                "increase the request timeout or check whether the corsa process is still responsive",
            ),
            Self::Rpc(_)
            | Self::Protocol(_)
            | Self::UnexpectedMessage(_)
            | Self::InvalidHandle(_) => None,
        }
    }

    /// Clones an error into a form safe to send to pending waiters.
    ///
    /// Some inner error types are not cheaply cloneable, so this method
    /// preserves the important semantics while normalizing them into owned
    /// variants.
    pub fn clone_for_pending(&self) -> Self {
        match self {
            Self::Io(err) => Self::Io(std::io::Error::new(
                err.kind(),
                compact_format(format_args!("{err}")).to_string(),
            )),
            Self::Json(err) => Self::Protocol(compact_format(format_args!("{err}"))),
            Self::Base64(err) => Self::Protocol(compact_format(format_args!("{err}"))),
            Self::Rpc(err) => Self::Rpc(err.clone()),
            Self::Protocol(err) => Self::Protocol(err.clone()),
            Self::UnexpectedMessage(err) => Self::UnexpectedMessage(err.clone()),
            Self::InvalidHandle(err) => Self::InvalidHandle(err.clone()),
            Self::Closed(err) => Self::Closed(err),
            Self::Unsupported(err) => Self::Unsupported(err),
            Self::Join(err) => Self::Join(err.clone()),
            Self::Timeout(err) => Self::Timeout(err.clone()),
        }
    }

    /// Creates a timeout error for a named operation.
    pub fn timeout(operation: &str, duration: Duration) -> Self {
        Self::Timeout(compact_format(format_args!(
            "{operation} timed out after {} ms",
            duration.as_millis()
        )))
    }
}
