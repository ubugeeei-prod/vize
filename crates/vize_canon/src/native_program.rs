//! Opt-in checking of original, sealed L4 whole-Module projections.
//!
//! This does not replace Vue virtual TS, the default route or the #6879 history
//! gate. The original File remains borrowed beside every backend observation.

use crate::{CorsaBridgeError, LspDiagnostic, LspRange};
use vize_l0::Span;
use vize_l4::targets::ts::{MappingError, ProgramProjection};

mod checker;
mod mapping;
pub use checker::{NativeProgramChecker, NativeProgramError};

/// An original backend diagnostic, including refused generated coordinates.
pub struct NativeProgramDiagnostic {
    backend: LspDiagnostic,
    span: Result<Span, MappingError>,
    original_range: Option<LspRange>,
}

impl NativeProgramDiagnostic {
    /// Complete original message/code/severity/related payload; never filtered.
    #[must_use]
    pub fn backend(&self) -> &LspDiagnostic {
        &self.backend
    }

    /// Exact authored UTF-8 coordinates, or the actual typed mapping refusal.
    pub fn span(&self) -> Result<Span, MappingError> {
        self.span
    }

    /// Authored LSP coordinates exist only for a completely mapped range.
    #[must_use]
    pub fn original_range(&self) -> Option<&LspRange> {
        self.original_range.as_ref()
    }
}

/// Diagnostics retain the genuine projection and its original File/unit borrow.
///
/// Detached diagnostics cannot construct a checked owner:
/// ```compile_fail
/// use vize_canon::native_program::NativeProgramCheck;
/// fn forge() { let _ = NativeProgramCheck { diagnostics: Vec::new() }; }
/// ```
/// A live check prevents discarding the actual projection:
/// ```compile_fail
/// use vize_canon::native_program::NativeProgramChecker;
/// use vize_l4::targets::ts::ProgramProjection;
/// async fn discard<'f, 'a>(mut checker: NativeProgramChecker,
///     projection: ProgramProjection<'f, 'a>) {
///     let check = checker.check(&projection).await.unwrap();
///     drop(projection);
///     let _ = check.diagnostics();
/// }
/// ```
pub struct NativeProgramCheck<'projection, 'file, 'arena> {
    projection: &'projection ProgramProjection<'file, 'arena>,
    diagnostics: Vec<NativeProgramDiagnostic>,
    cleanup_error: Option<CorsaBridgeError>,
    configuration_changed: bool,
}

impl<'projection, 'file, 'arena> NativeProgramCheck<'projection, 'file, 'arena> {
    #[must_use]
    pub fn projection(&self) -> &'projection ProgramProjection<'file, 'arena> {
        self.projection
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[NativeProgramDiagnostic] {
        &self.diagnostics
    }

    /// A cleanup failure retains the complete diagnostic result beside it.
    #[must_use]
    pub fn cleanup_error(&self) -> Option<&CorsaBridgeError> {
        self.cleanup_error.as_ref()
    }

    #[must_use]
    pub fn configuration_changed(&self) -> bool {
        self.configuration_changed
    }

    /// Complete coordinate/configuration observation; type errors remain errors.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.cleanup_error.is_none()
            && !self.configuration_changed
            && self.diagnostics.iter().all(|item| item.span.is_ok())
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|item| item.backend.severity == Some(1))
    }
}

#[cfg(test)]
mod tests;
