//! Builder for constructing [`Diagnostic`] values with a fluent API.

use tower_lsp::lsp_types::{Diagnostic, NumberOrString, Range};

use super::Severity;

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
