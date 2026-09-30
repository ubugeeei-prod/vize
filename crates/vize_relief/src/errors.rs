//! Compiler error types and codes.
mod compatibility;
mod diagnostic;
pub use diagnostic::COMPILER_ERROR;
pub mod recovery;
mod render;
use crate::SourceLocation;
pub use render::CompilerErrorWithSource;
use thiserror::Error;
use vize_l0::{CompactString, ToCompactString};
/// Compiler error
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct CompilerError {
    pub code: ErrorCode,
    pub message: CompactString,
    pub loc: Option<SourceLocation>,
}

impl CompilerError {
    pub fn new(code: ErrorCode, loc: Option<SourceLocation>) -> Self {
        Self {
            message: code.message().to_compact_string(),
            code,
            loc,
        }
    }

    pub fn with_message(
        code: ErrorCode,
        message: impl Into<CompactString>,
        loc: Option<SourceLocation>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            loc,
        }
    }

    /// Returns true when this diagnostic is a parser-level warning that
    /// downstream codegen can recover from. Mirrors `@vue/compiler-sfc`'s
    /// classification: a duplicate attribute is reported but does not
    /// gate render emission (#958).
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(self.code, ErrorCode::DuplicateAttribute) || self.is_compatibility_notice()
    }
}

pub use vize_l0::compiler_error::ErrorCode;

/// Result type for compiler operations
pub type CompilerResult<T> = Result<T, CompilerError>;

#[cfg(test)]
mod tests;
