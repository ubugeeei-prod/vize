//! Retain the whole preparation failure, including any returned stock syntax.

use crate::file::{L2Ir, RejectedL2Ir};
use vize_l0::Span;
use vize_l1::markup::InterpolationFailure;

/// Only the private original-child receiver constructs these normally owned rows.
///
/// ```compile_fail
/// use vize_l2::file::NativeL2IrInterpolationFailure;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeL2IrInterpolationFailure<'static>>();
/// ```
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l1::markup::InterpolationFailure;
/// use vize_l2::file::NativeL2IrInterpolationFailure;
/// fn forge<'a>(failure: InterpolationFailure<'a>) {
///     let _ = NativeL2IrInterpolationFailure { span: Span::new(0, 1), failure };
/// }
/// ```
pub struct NativeL2IrInterpolationFailure<'a> {
    pub(crate) span: Span,
    pub(crate) failure: InterpolationFailure<'a>,
}

impl<'a> NativeL2IrInterpolationFailure<'a> {
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    #[must_use]
    pub const fn failure(&self) -> &InterpolationFailure<'a> {
        &self.failure
    }
}

impl<'a> L2Ir<'a> {
    #[must_use]
    pub fn native_interpolation_failures(&self) -> &[NativeL2IrInterpolationFailure<'a>] {
        &self.facts.native_interpolation_failures
    }
}

impl<'a> RejectedL2Ir<'a> {
    #[must_use]
    pub fn native_interpolation_failures(&self) -> &[NativeL2IrInterpolationFailure<'a>] {
        &self.facts.native_interpolation_failures
    }
}
