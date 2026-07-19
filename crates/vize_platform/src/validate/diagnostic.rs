use serde::{Deserialize, Serialize};
use vize_carton::String;

/// Severity of a platform-contract diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticSeverity {
    /// Contract execution must stop.
    Error,
    /// Contract is valid but contains a likely configuration mistake.
    Warning,
}

/// Stable, source-addressable application-contract diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractDiagnostic {
    /// Stable machine-readable diagnostic code.
    pub code: &'static str,
    /// Severity used by CLI, editor, and CI consumers.
    pub severity: DiagnosticSeverity,
    /// JSON-style path into the authored contract.
    pub path: String,
    /// Human-readable explanation and next action.
    pub message: String,
}

impl ContractDiagnostic {
    pub(crate) fn error(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    pub(crate) fn warning(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Warning,
            path: path.into(),
            message: message.into(),
        }
    }
}
