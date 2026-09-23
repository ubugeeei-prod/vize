//! The host's mirror of the `vize:contracts@0.1.2` WIT types, and the
//! [`InputDialectGuest`] trait every hosting mode implements.
//!
//! Field order and names follow `contracts/wit/` exactly (serialized with
//! the WIT kebab-case spellings). The wasmtime bindings convert to and from
//! these types with exhaustive struct literals, so a WIT change that this
//! mirror does not follow stops the `extension-host` build.

use core::fmt;

use serde::{Deserialize, Serialize};
use vize_davinci::diagnostic as davinci;
use vize_davinci::diagnostic::Exemption;
use vize_s0::{String, cstr};
use vize_s1_to_s2::exemptions;

/// An error a guest reports without a witness the host can re-check against
/// its fact base, or under an exemption the host does not declare: exempt
/// from the witness law under this one counted row (P4-6a), never silently.
static GUEST_ERROR: Exemption = Exemption::new("vize_extension_host", "guest-error");

/// The in-tree exemptions a diagnostic may carry across the ABI; a
/// `legacy-exempt("producer/code")` naming one of them converts back to the
/// same declaration, so the built-in Vue guest round-trips exactly.
fn declared_exemption(name: &str) -> Option<&'static Exemption> {
    let (producer, code) = name.split_once('/')?;
    [
        &exemptions::SURFACE_SYNTAX,
        &exemptions::MISSING_END_TAG,
        &exemptions::LOWERING,
        &exemptions::V_SLOT,
        &exemptions::V_MODEL,
    ]
    .into_iter()
    .find(|exemption| exemption.producer() == producer && exemption.code() == code)
}

/// The WIT package this host implements.
pub const PACKAGE: &str = "vize:contracts@0.1.2";
/// The integer protocol version this host speaks.
pub const PROTOCOL_VERSION: u32 = 1;
/// The S1 page schema version this host reads and writes.
pub const S1_PAGE_SCHEMA: u32 = 1;
/// The S2 page schema version this host reads and writes.
pub const S2_PAGE_SCHEMA: u32 = 1;
/// The feature naming the S1 page schema a guest writes.
pub const S1_PAGE_FEATURE: &str = "s1-page@1";
/// The feature naming the S2 page schema a guest writes.
pub const S2_PAGE_FEATURE: &str = "s2-page@1";
/// Features the input-dialect world requires, sorted.
pub const REQUIRED_FEATURES: &[&str] = &[S1_PAGE_FEATURE, S2_PAGE_FEATURE];
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

/// `types.diagnostic-part`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticPart {
    pub kind: PartKind,
    pub span: Span,
    pub message: String,
}

/// `types.witness`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Witness {
    LegacyExempt(String),
}

/// `types.diagnostic`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub stage: Stage,
    pub span: Span,
    pub message: String,
    pub parts: Vec<DiagnosticPart>,
    pub witness: Option<Witness>,
}

/// `input-lowering.source-block`: one block, whole.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBlock {
    pub source: String,
    pub base: u32,
    pub lang: Option<String>,
}

/// `input-lowering.lowered-block`: everything one block lowers to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredBlock {
    pub surface: Page,
    pub semantic: Page,
    pub diagnostics: Vec<Diagnostic>,
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

/// One input-dialect guest, whatever hosts it: compiled in (the first-party
/// tier), a child process, or a wasmtime instance.
pub trait InputDialectGuest {
    /// `handshake.get-capability`.
    fn get_capability(&mut self) -> Result<Capability, GuestError>;
    /// `input-lowering.lower-block`.
    fn lower_block(&mut self, block: &SourceBlock) -> Result<LoweredBlock, GuestError>;
}

impl<G: InputDialectGuest + ?Sized> InputDialectGuest for Box<G> {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        (**self).get_capability()
    }

    fn lower_block(&mut self, block: &SourceBlock) -> Result<LoweredBlock, GuestError> {
        (**self).lower_block(block)
    }
}

impl From<vize_s0::Span> for Span {
    fn from(span: vize_s0::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<Span> for vize_s0::Span {
    fn from(span: Span) -> Self {
        vize_s0::Span::new(span.start, span.end)
    }
}

impl From<&davinci::Diagnostic> for Diagnostic {
    fn from(diagnostic: &davinci::Diagnostic) -> Self {
        Self {
            severity: match diagnostic.severity() {
                davinci::Severity::Error => Severity::Error,
                davinci::Severity::Warning => Severity::Warning,
                davinci::Severity::Info => Severity::Info,
                davinci::Severity::Hint => Severity::Hint,
            },
            stage: match diagnostic.stage {
                davinci::Stage::Source => Stage::Source,
                davinci::Stage::Surface => Stage::Surface,
                davinci::Stage::Semantic => Stage::Semantic,
                davinci::Stage::Lowered => Stage::Lowered,
                davinci::Stage::Emit => Stage::Emit,
            },
            span: diagnostic.span.into(),
            message: diagnostic.message.clone(),
            parts: diagnostic.parts.iter().map(DiagnosticPart::from).collect(),
            // WIT 0.1 carries exemptions only; a proof chain has no ABI shape
            // yet, so a proven diagnostic crosses without its witness.
            witness: diagnostic.exemption().map(|exemption| {
                Witness::LegacyExempt(cstr!("{}/{}", exemption.producer(), exemption.code()))
            }),
        }
    }
}

impl From<&davinci::DiagnosticPart> for DiagnosticPart {
    fn from(part: &davinci::DiagnosticPart) -> Self {
        Self {
            kind: match part.kind {
                davinci::PartKind::Primary => PartKind::Primary,
                davinci::PartKind::Secondary => PartKind::Secondary,
                davinci::PartKind::Help => PartKind::Help,
                davinci::PartKind::Suggestion => PartKind::Suggestion,
            },
            span: part.span.into(),
            message: part.message.clone(),
        }
    }
}

impl From<&Diagnostic> for davinci::Diagnostic {
    /// A guest diagnostic on the in-tree channel. An error keeps the in-tree
    /// exemption it names, or reports under [`GUEST_ERROR`]; a warning, note
    /// or hint is an advisory and carries no witness.
    fn from(diagnostic: &Diagnostic) -> Self {
        let stage = match diagnostic.stage {
            Stage::Source => davinci::Stage::Source,
            Stage::Surface => davinci::Stage::Surface,
            Stage::Semantic => davinci::Stage::Semantic,
            Stage::Lowered => davinci::Stage::Lowered,
            Stage::Emit => davinci::Stage::Emit,
        };
        let span = diagnostic.span.into();
        let message = diagnostic.message.clone();
        let advisory = match diagnostic.severity {
            Severity::Error => None,
            Severity::Warning => Some(davinci::Advisory::Warning),
            Severity::Info => Some(davinci::Advisory::Info),
            Severity::Hint => Some(davinci::Advisory::Hint),
        };
        let mut converted = match advisory {
            Some(advisory) => davinci::Diagnostic::new(advisory, stage, span, message),
            None => {
                let exemption = diagnostic
                    .witness
                    .as_ref()
                    .and_then(|Witness::LegacyExempt(name)| declared_exemption(name))
                    .unwrap_or(&GUEST_ERROR);
                davinci::Diagnostic::legacy_error(exemption, stage, span, message)
            }
        };
        for part in &diagnostic.parts {
            converted = converted.with_part(davinci::DiagnosticPart::from(part));
        }
        converted
    }
}

impl From<&DiagnosticPart> for davinci::DiagnosticPart {
    fn from(part: &DiagnosticPart) -> Self {
        Self {
            kind: match part.kind {
                PartKind::Primary => davinci::PartKind::Primary,
                PartKind::Secondary => davinci::PartKind::Secondary,
                PartKind::Help => davinci::PartKind::Help,
                PartKind::Suggestion => davinci::PartKind::Suggestion,
            },
            span: part.span.into(),
            message: part.message.clone(),
        }
    }
}
