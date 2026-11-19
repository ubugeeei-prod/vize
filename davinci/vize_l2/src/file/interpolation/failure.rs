//! Retain the whole preparation failure, including any returned stock syntax.

use crate::file::{FileArtifact, RejectedFile};
use vize_l0::Span;
use vize_l1::markup::InterpolationFailure;

/// Only the private original-child receiver constructs these normally owned rows.
///
/// ```compile_fail
/// use vize_l2::file::FileInterpolationFailure;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<FileInterpolationFailure<'static>>();
/// ```
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l1::markup::InterpolationFailure;
/// use vize_l2::file::FileInterpolationFailure;
/// fn forge<'a>(failure: InterpolationFailure<'a>) {
///     let _ = FileInterpolationFailure { span: Span::new(0, 1), failure };
/// }
/// ```
pub struct FileInterpolationFailure<'a> {
    pub(crate) span: Span,
    pub(crate) failure: InterpolationFailure<'a>,
}

impl<'a> FileInterpolationFailure<'a> {
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    #[must_use]
    pub const fn failure(&self) -> &InterpolationFailure<'a> {
        &self.failure
    }
}

impl<'a> FileArtifact<'a> {
    #[must_use]
    pub fn file_interpolation_failures(&self) -> &[FileInterpolationFailure<'a>] {
        &self.facts.file_interpolation_failures
    }
}

impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn file_interpolation_failures(&self) -> &[FileInterpolationFailure<'a>] {
        &self.facts.file_interpolation_failures
    }
}
