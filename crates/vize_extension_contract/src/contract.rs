//! The host's mirror of the `vize:contracts@0.1.3` WIT types, and the
//! [`InputDialectGuest`] trait every hosting mode implements.
//!
//! Field order and names follow `crates/vize_extension_sdk/wit/` exactly (serialized with
//! the WIT kebab-case spellings). The wasmtime bindings convert to and from
//! these types with exhaustive struct literals, so a WIT change that this
//! mirror does not follow stops the `extension-host` build.

use serde::{Deserialize, Serialize};
use vize_davinci::diagnostic as davinci;
use vize_davinci::diagnostic::Exemption;
use vize_l0::{String, cstr};
use vize_l1_to_l2::exemptions;

pub use vize_l0::extension::wire::{
    Capability, GuestError, GuestLimits, L1_PAGE_FEATURE, L1_PAGE_SCHEMA, L2_PAGE_FEATURE,
    L2_PAGE_SCHEMA, LANG_FEATURE_PREFIX, PACKAGE, PROTOCOL_VERSION, Page, PartKind,
    REQUIRED_FEATURES, S1_PAGE_FEATURE, S1_PAGE_SCHEMA, S2_PAGE_FEATURE, S2_PAGE_SCHEMA, Severity,
    SourceBlock, Span, Stage,
};

/// An error a guest reports without a witness the host can re-check against
/// its fact base, or under an exemption the host does not declare: exempt
/// from the witness law under this one counted row (P4-6a), never silently.
static GUEST_ERROR: Exemption = Exemption::new("vize_extension_contract", "guest-error");

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

/// `input-lowering.lowered-block`: everything one block lowers to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredBlock {
    pub surface: Page,
    pub semantic: Page,
    pub diagnostics: Vec<Diagnostic>,
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
