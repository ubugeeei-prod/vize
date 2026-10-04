//! A rejected document keeps all original observations made before the exit.

use std::vec::Vec;
use vize_l1::markup::{
    NativeInterpolationError, NativeInterpolationFailure, NativeInterpolationOperand,
    NativeTemplateComponent,
};

use super::super::NativeTemplateRefusal;

/// `index` is the next interpolation ordinal at the first failure. Document
/// refusals preserve their existing block-local/decoded/authored coordinates.
/// An observation failure offset is relative to the selected template block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedNativeTemplateRefusal {
    Document {
        index: usize,
        refusal: NativeTemplateRefusal,
    },
    Interpolation {
        offset: usize,
        index: usize,
        kind: NativeInterpolationError,
    },
}

/// No partial Doc is exposed. Successfully minted rejected operands remain in
/// this owned prefix; an actual observer failure is retained separately.
pub struct ObservedNativeTemplateFailure<'p, 'a> {
    original: &'p NativeTemplateComponent<'a>,
    operands: Vec<NativeInterpolationOperand<'a>>,
    refusal: ObservedNativeTemplateRefusal,
    interpolation_failure: Option<NativeInterpolationFailure<'a>>,
}

impl core::fmt::Debug for ObservedNativeTemplateFailure<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ObservedNativeTemplateFailure")
            .field("original", &self.original)
            .field("operand_count", &self.operands.len())
            .field("refusal", &self.refusal)
            .field("interpolation_failure", &self.interpolation_failure)
            .finish()
    }
}

impl<'p, 'a> ObservedNativeTemplateFailure<'p, 'a> {
    pub(super) fn new(
        original: &'p NativeTemplateComponent<'a>,
        operands: Vec<NativeInterpolationOperand<'a>>,
        refusal: ObservedNativeTemplateRefusal,
        interpolation_failure: Option<NativeInterpolationFailure<'a>>,
    ) -> Self {
        Self {
            original,
            operands,
            refusal,
            interpolation_failure,
        }
    }
    pub fn original(&self) -> &'p NativeTemplateComponent<'a> {
        self.original
    }
    pub fn operands(&self) -> &[NativeInterpolationOperand<'a>] {
        &self.operands
    }
    pub fn refusal(&self) -> ObservedNativeTemplateRefusal {
        self.refusal
    }
    pub fn interpolation_failure(&self) -> Option<&NativeInterpolationFailure<'a>> {
        self.interpolation_failure.as_ref()
    }
    /// Transfer the same selection borrow, owned observed prefix and exact
    /// refusal/failure. This grants no completed template or SFC admission.
    pub fn into_parts(
        self,
    ) -> (
        &'p NativeTemplateComponent<'a>,
        Vec<NativeInterpolationOperand<'a>>,
        ObservedNativeTemplateRefusal,
        Option<NativeInterpolationFailure<'a>>,
    ) {
        (
            self.original,
            self.operands,
            self.refusal,
            self.interpolation_failure,
        )
    }
}
