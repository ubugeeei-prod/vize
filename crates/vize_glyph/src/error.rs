//! Error types for vize_glyph formatter.

use thiserror::Error;
use vize_carton::String;

/// Errors that can occur during formatting
#[derive(Debug, Error)]
pub enum FormatError {
    /// Error parsing the SFC
    #[error("Failed to parse SFC: {0}")]
    ParseError(String),

    /// Error parsing JavaScript/TypeScript
    #[error("Failed to parse script: {0}")]
    ScriptParseError(String),

    /// Error formatting script
    #[error("Failed to format script: {0}")]
    ScriptFormatError(String),

    /// Error parsing template
    #[error("Failed to parse template: {0}")]
    TemplateParseError(String),

    /// Error formatting template
    #[error("Failed to format template: {0}")]
    TemplateFormatError(String),

    /// Error formatting style
    #[error("Failed to format style: {0}")]
    StyleFormatError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<vize_atelier_sfc::SfcError> for FormatError {
    fn from(err: vize_atelier_sfc::SfcError) -> Self {
        FormatError::ParseError(err.message)
    }
}
