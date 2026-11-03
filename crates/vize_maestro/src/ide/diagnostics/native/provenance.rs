use std::ops::Deref;

use tower_lsp::lsp_types::Diagnostic;
use vize_canon::corsa_bridge::DiagnosticPhase;

/// Native ownership retained internally, independently of the published wire.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativeDiagnostic {
    pub(crate) diagnostic: Diagnostic,
    pub(crate) phase: DiagnosticPhase,
}

impl From<Diagnostic> for NativeDiagnostic {
    fn from(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostic,
            phase: DiagnosticPhase::Unknown,
        }
    }
}

impl Deref for NativeDiagnostic {
    type Target = Diagnostic;

    fn deref(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

impl PartialEq<Diagnostic> for NativeDiagnostic {
    fn eq(&self, other: &Diagnostic) -> bool {
        self.diagnostic == *other
    }
}

impl serde::Serialize for NativeDiagnostic {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&self.diagnostic, serializer)
    }
}
