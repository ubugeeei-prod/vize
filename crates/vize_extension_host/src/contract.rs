//! The host's mirror of the `vize:contracts@0.1.0` WIT types, and the
//! [`InputDialectGuest`] trait every hosting mode implements.
//!
//! Field order and names follow `contracts/wit/` exactly (serialized with
//! the WIT kebab-case spellings). The wasmtime bindings convert to and from
//! these types with exhaustive struct literals, so a WIT change that this
//! mirror does not follow stops the `extension-host` build.

use core::fmt;

use serde::{Deserialize, Serialize};
use vize_davinci::diagnostic as davinci;
use vize_s0::String;

/// The WIT package this host implements.
pub const PACKAGE: &str = "vize:contracts@0.1.0";
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
}

impl fmt::Display for GuestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instantiate(message) => write!(f, "guest instantiation failed: {message}"),
            Self::Trap(message) => write!(f, "guest trapped: {message}"),
            Self::Transport(message) => write!(f, "guest transport failed: {message}"),
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
            severity: match diagnostic.severity {
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
            witness: diagnostic.witness.as_ref().map(|witness| match witness {
                davinci::Witness::LegacyExempt(producer) => Witness::LegacyExempt(producer.clone()),
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
    fn from(diagnostic: &Diagnostic) -> Self {
        Self {
            severity: match diagnostic.severity {
                Severity::Error => davinci::Severity::Error,
                Severity::Warning => davinci::Severity::Warning,
                Severity::Info => davinci::Severity::Info,
                Severity::Hint => davinci::Severity::Hint,
            },
            stage: match diagnostic.stage {
                Stage::Source => davinci::Stage::Source,
                Stage::Surface => davinci::Stage::Surface,
                Stage::Semantic => davinci::Stage::Semantic,
                Stage::Lowered => davinci::Stage::Lowered,
                Stage::Emit => davinci::Stage::Emit,
            },
            span: diagnostic.span.into(),
            message: diagnostic.message.clone(),
            parts: diagnostic
                .parts
                .iter()
                .map(davinci::DiagnosticPart::from)
                .collect(),
            witness: diagnostic.witness.as_ref().map(|witness| match witness {
                Witness::LegacyExempt(producer) => davinci::Witness::LegacyExempt(producer.clone()),
            }),
        }
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
