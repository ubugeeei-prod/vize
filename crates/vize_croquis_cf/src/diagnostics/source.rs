//! Coordinate provenance for diagnostics whose kind can arise in either SFC block.

use super::CrossFileDiagnostic;

/// Coordinate space in which a diagnostic's primary offsets were produced.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DiagnosticSource {
    /// Preserve the adapter's existing kind-based mapping.
    #[default]
    Unspecified,
    /// Script content (normal script followed by setup script when both exist).
    Script,
    /// Template content.
    Template,
}

impl CrossFileDiagnostic {
    /// Retain the producer's primary coordinate space without changing message or span.
    pub fn with_primary_source(mut self, source: DiagnosticSource) -> Self {
        self.primary_source = source;
        self
    }
}
