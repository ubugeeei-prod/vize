//! Neutral external wire records and protocol constants.
//!
//! These are serialized ABI records, not L1/L2 native IR. Native diagnostics
//! and producer exemption lookup stay at their producing boundary.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::String;

/// The WIT package this host implements.
pub const PACKAGE: &str = "vize:contracts@0.1.3";
/// The integer protocol version this host speaks.
pub const PROTOCOL_VERSION: u32 = 1;
/// The L1 page schema version this host reads and writes.
pub const L1_PAGE_SCHEMA: u32 = 1;
/// The L2 page schema version this host reads and writes.
pub const L2_PAGE_SCHEMA: u32 = 1;
/// The feature naming the L1 page schema a guest writes.
pub const L1_PAGE_FEATURE: &str = "s1-page@1";
/// The feature naming the L2 page schema a guest writes.
pub const L2_PAGE_FEATURE: &str = "s2-page@1";
/// Compatibility names for the original contract constants.
pub use self::{
    L1_PAGE_FEATURE as S1_PAGE_FEATURE, L1_PAGE_SCHEMA as S1_PAGE_SCHEMA,
    L2_PAGE_FEATURE as S2_PAGE_FEATURE, L2_PAGE_SCHEMA as S2_PAGE_SCHEMA,
};
/// Features the input-dialect world requires, sorted.
pub const REQUIRED_FEATURES: &[&str] = &[L1_PAGE_FEATURE, L2_PAGE_FEATURE];
/// Prefix of the optional features declaring a `lang` value a guest lowers.
pub const LANG_FEATURE_PREFIX: &str = "lang:";

/// `handshake.capability`: what a guest offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Capability {
    pub protocol_version: u32,
    pub features: Vec<String>,
}

/// `types.span`: file-absolute UTF-8 byte offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

/// `types.page`: one folio page in `Full` mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Page {
    pub schema_version: u32,
    pub text: String,
}

/// `types.severity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// `types.stage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    Source,
    Surface,
    Semantic,
    Lowered,
    Emit,
}

/// `types.part-kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PartKind {
    Primary,
    Secondary,
    Help,
    Suggestion,
}

/// `input-lowering.source-block`: one block, whole.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBlock {
    pub source: String,
    pub base: u32,
    pub lang: Option<String>,
}

/// Why a call into a guest returned no value. Distinct from a contract
/// refusal: the guest never answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GuestError {
    /// The guest could not be loaded or instantiated.
    Instantiate(String),
    /// The guest trapped during the call.
    Trap(String),
    /// The transport to an out-of-process guest failed.
    Transport(String),
    /// The call used up the guest's per-call fuel budget and was stopped.
    OutOfFuel { budget: u64 },
    /// The guest tried to grow its memory past its limit.
    MemoryLimit { limit: u64 },
}

/// Per-guest resource limits a wasmtime host enforces in both hosting modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GuestLimits {
    /// Fuel (roughly, executed wasm instructions) granted to each call.
    pub fuel_per_call: u64,
    /// The largest linear memory the guest may grow to, in bytes.
    pub max_memory_bytes: u64,
}

impl Default for GuestLimits {
    /// One billion units of fuel per call and 128 MiB of memory: far above
    /// what lowering one block needs, low enough that a runaway guest is
    /// stopped in about a second.
    fn default() -> Self {
        Self {
            fuel_per_call: 1_000_000_000,
            max_memory_bytes: 128 * 1024 * 1024,
        }
    }
}

impl fmt::Display for GuestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instantiate(message) => write!(f, "guest instantiation failed: {message}"),
            Self::Trap(message) => write!(f, "guest trapped: {message}"),
            Self::Transport(message) => write!(f, "guest transport failed: {message}"),
            Self::OutOfFuel { budget } => {
                write!(
                    f,
                    "guest stopped: it used up its fuel budget of {budget} per call"
                )
            }
            Self::MemoryLimit { limit } => {
                write!(
                    f,
                    "guest stopped: it tried to grow its memory past {limit} bytes"
                )
            }
        }
    }
}

impl From<crate::Span> for Span {
    fn from(span: crate::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<Span> for crate::Span {
    fn from(span: Span) -> Self {
        crate::Span::new(span.start, span.end)
    }
}
