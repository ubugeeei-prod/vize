//! Diagnostics aggregation from multiple sources.
//!
//! Aggregates diagnostics from:
//! - SFC parser errors
//! - Template parser errors
//! - vize_patina (linter)
//! - Future: vize_canon (type checker)
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod collectors;
#[cfg(feature = "native")]
mod corsa;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range, Url};

use crate::server::ServerState;

/// Diagnostic source identifiers.
pub mod sources {
    pub const SFC_PARSER: &str = "vize/sfc";
    pub const TEMPLATE_PARSER: &str = "vize/template";
    pub const SCRIPT_PARSER: &str = "vize/script";
    pub const LINTER: &str = "vize/lint";
    pub const TYPE_CHECKER: &str = "vize/types";
    pub const MUSEA: &str = "vize/musea";
}

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

/// Source position mapping from @vize-map comments.
#[cfg(feature = "native")]
#[derive(Debug, Clone)]
pub(super) struct SourceMapping {
    /// Byte offset start in SFC
    pub(super) start: u32,
    /// Byte offset end in SFC
    pub(super) end: u32,
}

/// Virtual TypeScript generation result with position mapping info.
#[cfg(feature = "native")]
pub(super) struct VirtualTsResult {
    /// Generated TypeScript code
    pub(super) code: String,
    /// Line number where user code starts in virtual TS (0-indexed)
    pub(super) user_code_start_line: u32,
    /// Line number where script starts in original SFC (1-indexed)
    pub(super) sfc_script_start_line: u32,
    /// Line number where template scope starts in virtual TS (0-indexed)
    pub(super) template_scope_start_line: u32,
    /// Line-to-source mappings from @vize-map comments
    /// Index is virtual TS line number (0-indexed), value is source position in SFC
    pub(super) line_mappings: Vec<Option<SourceMapping>>,
    /// Number of import lines skipped from user code (to adjust line mapping)
    pub(super) skipped_import_lines: u32,
}

/// Diagnostic service for collecting and aggregating diagnostics.
pub struct DiagnosticService;

impl DiagnosticService {
    /// Collect all diagnostics for a document.
    pub fn collect(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        let Some(doc) = state.documents.get(uri) else {
            tracing::warn!("collect: document not found for {}", uri);
            return vec![];
        };

        let content = doc.text();
        let mut diagnostics = Vec::new();

        // Check if this is an Art file (*.art.vue)
        let path = uri.path();
        if path.ends_with(".art.vue") {
            // Musea-specific diagnostics for Art files
            diagnostics.extend(Self::collect_musea_diagnostics(uri, &content));
            // Don't return early here; async collection still adds Corsa diagnostics.
            return diagnostics;
        }

        // Standard SFC processing
        // Collect SFC parser diagnostics
        let sfc_diags = Self::collect_sfc_diagnostics(uri, &content);
        tracing::info!("collect: SFC parser diagnostics: {}", sfc_diags.len());
        diagnostics.extend(sfc_diags);

        // Collect template parser diagnostics
        let template_diags = Self::collect_template_diagnostics(uri, &content);
        tracing::info!(
            "collect: template parser diagnostics: {}",
            template_diags.len()
        );
        diagnostics.extend(template_diags);

        // Collect linter diagnostics (vize_patina)
        let lint_diags = Self::collect_lint_diagnostics(uri, &content);
        tracing::info!("collect: patina lint diagnostics: {}", lint_diags.len());
        diagnostics.extend(lint_diags);

        // Collect type checker diagnostics (vize_canon)
        let type_diags = super::TypeService::collect_diagnostics(state, uri);
        tracing::info!("collect: type checker diagnostics: {}", type_diags.len());
        diagnostics.extend(type_diags);

        // Also lint inline <art> blocks in regular .vue files
        let inline_art_diags = Self::collect_inline_art_diagnostics(uri, &content);
        tracing::info!(
            "collect: inline art diagnostics: {}",
            inline_art_diags.len()
        );
        diagnostics.extend(inline_art_diags);

        diagnostics
    }

    /// Collect diagnostics asynchronously (includes Corsa diagnostics when available).
    #[cfg(feature = "native")]
    pub async fn collect_async(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        tracing::info!("collect_async: {}", uri);

        // Start with sync diagnostics (patina, etc.)
        let mut diagnostics = Self::collect(state, uri);
        tracing::info!("sync diagnostics count: {}", diagnostics.len());

        // Try to get Corsa diagnostics (with timeout, skip on failure).
        // Use 10s timeout - polling for diagnostics internally uses 5s
        let corsa_future = Self::collect_corsa_diagnostics(state, uri);
        match tokio::time::timeout(std::time::Duration::from_secs(10), corsa_future).await {
            Ok(corsa_diags) => {
                tracing::info!("corsa diagnostics count: {}", corsa_diags.len());
                diagnostics.extend(corsa_diags);
            }
            Err(_) => {
                tracing::warn!("corsa diagnostics timed out for {}", uri);
            }
        }

        diagnostics
    }

    /// Create a diagnostic from a custom error.
    pub fn create_diagnostic(
        range: Range,
        severity: Severity,
        source: &str,
        code: Option<i32>,
        message: String,
    ) -> Diagnostic {
        Diagnostic {
            range,
            severity: Some(severity.into()),
            code: code.map(NumberOrString::Number),
            source: Some(source.to_string()),
            message,
            ..Default::default()
        }
    }
}

/// Builder for creating diagnostics.
pub struct DiagnosticBuilder {
    range: Range,
    severity: Severity,
    source: String,
    code: Option<i32>,
    message: String,
    related_information: Vec<tower_lsp::lsp_types::DiagnosticRelatedInformation>,
}

impl DiagnosticBuilder {
    /// Create a new diagnostic builder.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            range: Range::default(),
            severity: Severity::Error,
            source: "vize".to_string(),
            code: None,
            message: message.into(),
            related_information: Vec::new(),
        }
    }

    /// Set the range.
    pub fn range(mut self, range: Range) -> Self {
        self.range = range;
        self
    }

    /// Set the severity.
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the source.
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set the error code.
    pub fn code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    /// Add related information.
    pub fn related(
        mut self,
        location: tower_lsp::lsp_types::Location,
        message: impl Into<String>,
    ) -> Self {
        self.related_information
            .push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
                location,
                message: message.into(),
            });
        self
    }

    /// Build the diagnostic.
    pub fn build(self) -> Diagnostic {
        Diagnostic {
            range: self.range,
            severity: Some(self.severity.into()),
            code: self.code.map(NumberOrString::Number),
            source: Some(self.source),
            message: self.message,
            related_information: if self.related_information.is_empty() {
                None
            } else {
                Some(self.related_information)
            },
            ..Default::default()
        }
    }
}

/// Convert byte offset to (line, column) - both 0-indexed for LSP.
pub(super) fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut current_offset = 0;

    for ch in source.chars() {
        if current_offset >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticBuilder, Severity};
    use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString};

    #[test]
    fn test_diagnostic_builder() {
        let diagnostic = DiagnosticBuilder::new("Test error")
            .severity(Severity::Warning)
            .source("test")
            .code(42)
            .build();

        assert_eq!(diagnostic.message, "Test error");
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diagnostic.source, Some("test".to_string()));
        assert_eq!(diagnostic.code, Some(NumberOrString::Number(42)));
    }

    #[test]
    fn test_severity_conversion() {
        assert_eq!(
            DiagnosticSeverity::from(Severity::Error),
            DiagnosticSeverity::ERROR
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Warning),
            DiagnosticSeverity::WARNING
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Information),
            DiagnosticSeverity::INFORMATION
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Hint),
            DiagnosticSeverity::HINT
        );
    }
}
