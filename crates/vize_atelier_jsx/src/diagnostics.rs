//! Diagnostics produced while parsing and lowering JSX/TSX.
//!
//! Every diagnostic carries a Vize byte range so it can be mapped back to the
//! original `.jsx`/`.tsx` source by the compiler, type checker, and LSP.

use vize_carton::String;
use vize_relief::ast::core::SourceLocation;

/// Severity of a JSX lowering diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A hard error: lowering could not faithfully represent the input.
    Error,
    /// A non-fatal warning: lowering continued but something is suspect.
    Warning,
}

/// A diagnostic with a message and a source location.
#[derive(Debug, Clone, PartialEq)]
pub struct JsxDiagnostic {
    /// Severity of the diagnostic.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Inclusive-start / exclusive-end byte range in the original source.
    pub start: u32,
    /// End byte offset.
    pub end: u32,
}

impl JsxDiagnostic {
    /// Build an error diagnostic spanning `[start, end)`.
    pub fn error(message: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            start,
            end,
        }
    }

    /// Build a warning diagnostic spanning `[start, end)`.
    pub fn warning(message: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            start,
            end,
        }
    }

    /// Build an error diagnostic from a Vize [`SourceLocation`].
    pub fn error_at(message: impl Into<String>, loc: &SourceLocation) -> Self {
        Self::error(message, loc.start.offset, loc.end.offset)
    }

    /// Whether this diagnostic is an error.
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_and_warning_constructors() {
        let e = JsxDiagnostic::error("boom", 1, 4);
        assert!(e.is_error());
        assert_eq!((e.start, e.end), (1, 4));

        let w = JsxDiagnostic::warning("hmm", 2, 3);
        assert!(!w.is_error());
        assert_eq!(w.severity, Severity::Warning);
    }
}
