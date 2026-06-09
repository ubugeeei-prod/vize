//! Diagnostic severity levels and their mapping to LSP severities.

use tower_lsp::lsp_types::DiagnosticSeverity;

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

impl From<Severity> for DiagnosticSeverity {
    fn from(s: Severity) -> Self {
        match s {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Information => DiagnosticSeverity::INFORMATION,
            Severity::Hint => DiagnosticSeverity::HINT,
        }
    }
}
