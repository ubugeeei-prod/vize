//! Keep provenance in the existing diagnostic cache entry and read result.

use crate::corsa_bridge::DiagnosticPhase;
use lsp_types::Diagnostic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PhasedDiagnostic {
    pub(crate) diagnostic: Diagnostic,
    pub(crate) phase: DiagnosticPhase,
}

impl PhasedDiagnostic {
    pub(crate) fn unknown(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostic,
            phase: DiagnosticPhase::Unknown,
        }
    }
}

impl std::borrow::Borrow<Diagnostic> for PhasedDiagnostic {
    fn borrow(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

impl std::ops::Deref for PhasedDiagnostic {
    type Target = Diagnostic;
    fn deref(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn an_unphased_transport_never_classifies_a_numeric_code() {
        let diagnostic = Diagnostic {
            code: Some(lsp_types::NumberOrString::Number(1192)),
            ..Diagnostic::default()
        };
        assert_eq!(
            PhasedDiagnostic::unknown(diagnostic).phase,
            DiagnosticPhase::Unknown
        );
    }
}
