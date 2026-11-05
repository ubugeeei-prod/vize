//! Diagnostic provenance is an owned transport result, not a numeric-code rule.

use super::LspDiagnostic;
use serde::{Deserialize, Serialize};

/// Compiler phase reported by the producer that collected this diagnostic.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticPhase {
    /// The transport did not provide a compiler phase.
    #[default]
    Unknown,
    Syntactic,
    Semantic,
    Suggestion,
    Declaration,
}

/// A diagnostic and its producer-owned phase, retained as one result.
#[derive(Debug, Clone)]
pub struct PhasedLspDiagnostic {
    pub diagnostic: LspDiagnostic,
    pub phase: DiagnosticPhase,
}
