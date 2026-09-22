//! The toolchain boundary: how the dialect asks `moonc` to check a
//! projection.
//!
//! One coarse-grained call per SFC — the whole virtual file in, every
//! diagnostic out — the same shape as the WIT worlds (charter #15), so the
//! P6-4a hosting choice ([`crate::native::NativeMoonc`], behind the `moonc`
//! feature) and the recorded fallback (the `@moonbit/moonc-worker` build
//! of the same compiler under Node) are interchangeable behind
//! [`MooncHost`]. The answer is `moonc`'s own JSON lines, verbatim: the
//! mapping back to the SFC is the dialect's job ([`crate::diagnostic`]),
//! never the host's, so every host shares one mapping.

use core::fmt;

use vize_s0::{String, ToCompactString};

/// One check request: a single virtual file in one package.
#[derive(Debug, Clone, Copy)]
pub struct CheckUnit<'a> {
    /// The package the file is checked as (`vize/sfc`).
    pub package: &'a str,
    /// The virtual file's name (`App.vue.mbt`).
    pub file_name: &'a str,
    /// The virtual file's text.
    pub source: &'a str,
}

/// What a checker answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCheck {
    /// The toolchain version that answered (`0.10.7+bc794d341`): results
    /// are only comparable, and cacheable, under the same version.
    pub toolchain: String,
    /// `moonc`'s JSON diagnostic lines, verbatim, in emission order.
    pub lines: Vec<String>,
}

/// Why a host produced no answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostError {
    /// The toolchain could not be found or started.
    Unavailable(String),
    /// The virtual file exceeds what the transport carries in one call.
    TooLarge {
        /// The virtual file's size in bytes.
        bytes: usize,
        /// The transport's limit in bytes.
        limit: usize,
    },
    /// The toolchain failed without diagnostics (its exit status and
    /// stderr).
    Failed(String),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(why) => write!(f, "moonc is unavailable: {why}"),
            Self::TooLarge { bytes, limit } => write!(
                f,
                "the projection is {bytes} bytes; one moonc call carries at most {limit}"
            ),
            Self::Failed(why) => write!(f, "moonc failed: {why}"),
        }
    }
}

/// A MoonBit checker behind the dialect's capability boundary.
pub trait MooncHost {
    /// The toolchain version every answer is tied to.
    fn toolchain(&self) -> &str;

    /// Check one virtual file.
    ///
    /// # Errors
    ///
    /// [`HostError`] when the toolchain produced no answer; diagnostics,
    /// errors included, are an answer.
    fn check(&mut self, unit: &CheckUnit<'_>) -> Result<RawCheck, HostError>;
}

/// A host replaying one recorded answer.
///
/// The default build has no toolchain, so its tests drive the whole
/// mapping from committed `moonc` output through this host; the
/// `moonc`-feature suite proves the live toolchain still answers exactly
/// that output.
#[derive(Debug, Clone)]
pub struct Replay {
    toolchain: String,
    lines: Vec<String>,
}

impl Replay {
    /// Replay `jsonl` (one `moonc` JSON line per line) as `toolchain`.
    #[must_use]
    pub fn new(toolchain: &str, jsonl: &str) -> Self {
        Self {
            toolchain: toolchain.to_compact_string(),
            lines: jsonl
                .lines()
                .filter(|line| !line.is_empty())
                .map(ToCompactString::to_compact_string)
                .collect(),
        }
    }
}

impl MooncHost for Replay {
    fn toolchain(&self) -> &str {
        &self.toolchain
    }

    fn check(&mut self, _unit: &CheckUnit<'_>) -> Result<RawCheck, HostError> {
        Ok(RawCheck {
            toolchain: self.toolchain.clone(),
            lines: self.lines.clone(),
        })
    }
}
