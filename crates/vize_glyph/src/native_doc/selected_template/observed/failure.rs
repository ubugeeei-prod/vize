//! A rejected document keeps all original observations made before the exit.

use super::super::input::Observed;
use super::binding::{BindingCustody, ObservedNativeTemplateBindingFailureParts};
use std::{boxed::Box, vec::Vec};
use vize_l0::Span;
use vize_l1::markup::{
    NativeAttributeBindingExpression, NativeAttributeExpression, NativeAttributeExpressionFailure,
    NativeAttributeOperandError,
};
use vize_l1::markup::{
    NativeInterpolationError, NativeInterpolationFailure, NativeInterpolationOperand,
    NativeTemplateComponent,
};

use super::super::NativeTemplateRefusal;

/// Document/Interpolation indices are the next interpolation ordinal.
/// Attribute indices are ordered conditional observations, with authored-file
/// name spans. Binding indices independently count ordered binding observations.
/// Wrapped document refusals retain their existing local/decoded/
/// authored coordinates; interpolation offsets are selected-block relative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedNativeTemplateRefusal {
    Document {
        index: usize,
        refusal: NativeTemplateRefusal,
    },
    Attribute {
        span: Span,
        index: usize,
        kind: NativeAttributeOperandError,
    },
    Binding {
        span: Span,
        index: usize,
        kind: NativeAttributeOperandError,
    },
    Interpolation {
        offset: usize,
        index: usize,
        kind: NativeInterpolationError,
    },
}

/// Existing six-part transfer schema for interpolation/conditional custody.
/// Added binding custody is released; use `into_binding_parts` to retain it.
pub type ObservedNativeTemplateFailureParts<'p, 'a> = (
    &'p NativeTemplateComponent<'a>,
    Vec<NativeInterpolationOperand<'a>>,
    Vec<NativeAttributeExpression<'a>>,
    ObservedNativeTemplateRefusal,
    Option<NativeInterpolationFailure<'a>>,
    Option<NativeAttributeExpressionFailure<'a>>,
);

/// No partial Doc is exposed. Successfully minted rejected operands remain in
/// this owned prefix; an actual observer failure is retained separately.
pub struct ObservedNativeTemplateFailure<'p, 'a> {
    original: &'p NativeTemplateComponent<'a>,
    operands: Vec<NativeInterpolationOperand<'a>>,
    refusal: ObservedNativeTemplateRefusal,
    interpolation_failure: Option<NativeInterpolationFailure<'a>>,
    attributes: Vec<NativeAttributeExpression<'a>>,
    attribute_failure: Option<Box<(Span, usize, NativeAttributeExpressionFailure<'a>)>>,
    binding_custody: Option<Box<BindingCustody<'a>>>,
}

impl core::fmt::Debug for ObservedNativeTemplateFailure<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ObservedNativeTemplateFailure")
            .field("original", &self.original)
            .field("operand_count", &self.operands.len())
            .field("attribute_count", &self.attributes.len())
            .field("attribute_failure", &self.attribute_failure)
            .field("binding_count", &self.binding_operands().len())
            .field("binding_failure", &self.binding_failure())
            .field("refusal", &self.refusal)
            .field("interpolation_failure", &self.interpolation_failure)
            .finish()
    }
}

impl<'p, 'a> ObservedNativeTemplateFailure<'p, 'a> {
    pub(super) fn new(
        original: &'p NativeTemplateComponent<'a>,
        observations: Observed<'a>,
        refusal: ObservedNativeTemplateRefusal,
    ) -> Self {
        Self {
            original,
            operands: observations.operands,
            refusal,
            interpolation_failure: observations.failure,
            attributes: observations.attributes,
            attribute_failure: observations.attribute_failure.map(Box::new),
            binding_custody: BindingCustody::pack(
                observations.bindings,
                observations.binding_failure,
            ),
        }
    }
    pub fn original(&self) -> &'p NativeTemplateComponent<'a> {
        self.original
    }
    pub fn operands(&self) -> &[NativeInterpolationOperand<'a>] {
        &self.operands
    }
    pub fn attribute_operands(&self) -> &[NativeAttributeExpression<'a>] {
        &self.attributes
    }
    pub fn attribute_failure(&self) -> Option<&NativeAttributeExpressionFailure<'a>> {
        self.attribute_failure
            .as_deref()
            .map(|(_, _, failure)| failure)
    }
    pub fn binding_operands(&self) -> &[NativeAttributeBindingExpression<'a>] {
        self.binding_custody
            .as_deref()
            .map(|custody| custody.bindings.as_slice())
            .unwrap_or(&[])
    }
    pub fn binding_failure(&self) -> Option<&NativeAttributeExpressionFailure<'a>> {
        self.binding_custody
            .as_deref()
            .and_then(|custody| custody.failure.as_ref())
            .map(|(_, _, failure)| failure)
    }
    pub(in crate::native_doc) fn into_observations(
        self,
    ) -> (Observed<'a>, ObservedNativeTemplateRefusal) {
        let custody = BindingCustody::unpack(self.binding_custody);
        (
            Observed {
                operands: self.operands,
                failure: self.interpolation_failure,
                attributes: self.attributes,
                attribute_failure: self.attribute_failure.map(|failure| *failure),
                bindings: custody.bindings,
                binding_failure: custody.failure,
            },
            self.refusal,
        )
    }
    pub fn refusal(&self) -> ObservedNativeTemplateRefusal {
        self.refusal
    }
    pub fn interpolation_failure(&self) -> Option<&NativeInterpolationFailure<'a>> {
        self.interpolation_failure.as_ref()
    }
    /// Transfer interpolation/conditional prefixes and both actual failure owners.
    /// The existing six-part schema releases added binding custody; retain this
    /// owner or use `into_binding_parts` to transfer all three families.
    /// This grants no completed template or SFC admission.
    pub fn into_full_parts(self) -> ObservedNativeTemplateFailureParts<'p, 'a> {
        (
            self.original,
            self.operands,
            self.attributes,
            self.refusal,
            self.interpolation_failure,
            self.attribute_failure.map(|failure| failure.2),
        )
    }
    /// Transfer every genuine observed prefix and each actual failure owner.
    /// This grants no completed template or SFC admission.
    pub fn into_binding_parts(self) -> ObservedNativeTemplateBindingFailureParts<'p, 'a> {
        let custody = BindingCustody::unpack(self.binding_custody);
        ObservedNativeTemplateBindingFailureParts {
            original: self.original,
            operands: self.operands,
            attributes: self.attributes,
            bindings: custody.bindings,
            refusal: self.refusal,
            interpolation_failure: self.interpolation_failure,
            attribute_failure: self.attribute_failure.map(|failure| failure.2),
            binding_failure: custody.failure.map(|(_, _, failure)| failure),
        }
    }
    /// Preserve the original interpolation-only transfer contract.
    /// This releases conditional/binding observations and their actual failures;
    /// use `into_binding_parts` or retain this complete owner to preserve them.
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
