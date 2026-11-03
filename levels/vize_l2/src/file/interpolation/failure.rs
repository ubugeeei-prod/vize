//! Retain the whole preparation failure, including any returned stock syntax.

use crate::file::{FileArtifact, RejectedFile};
use vize_l0::Span;
use vize_l1::markup::NativeInterpolationFailure;

/// Only the private original-child receiver constructs these normally owned rows.
///
/// ```compile_fail
/// use vize_l2::file::NativeFileInterpolationFailure;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeFileInterpolationFailure<'static>>();
/// ```
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l1::markup::NativeInterpolationFailure;
/// use vize_l2::file::NativeFileInterpolationFailure;
/// fn forge<'a>(failure: NativeInterpolationFailure<'a>) {
///     let _ = NativeFileInterpolationFailure { span: Span::new(0, 1), failure };
/// }
/// ```
pub struct NativeFileInterpolationFailure<'a> {
    pub(crate) span: Span,
    pub(crate) failure: NativeInterpolationFailure<'a>,
}

impl<'a> NativeFileInterpolationFailure<'a> {
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    #[must_use]
    pub const fn failure(&self) -> &NativeInterpolationFailure<'a> {
        &self.failure
    }
}

impl<'a> FileArtifact<'a> {
    #[must_use]
    pub fn native_interpolation_failures(&self) -> &[NativeFileInterpolationFailure<'a>] {
        &self.facts.native_interpolation_failures
    }
}

impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn native_interpolation_failures(&self) -> &[NativeFileInterpolationFailure<'a>] {
        &self.facts.native_interpolation_failures
    }
}
