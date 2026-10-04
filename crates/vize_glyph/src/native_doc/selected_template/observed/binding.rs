//! Complete normal-owned transfers and compact rejected binding custody.

use std::{boxed::Box, vec::Vec};
use vize_l0::Span;
use vize_l1::markup::{
    NativeAttributeBindingExpression, NativeAttributeExpression, NativeAttributeExpressionFailure,
    NativeInterpolationFailure, NativeInterpolationOperand, NativeTemplateComponent,
};

use super::Doc;
use super::failure::ObservedNativeTemplateRefusal;

/// Complete selected-template transfer, retaining all three original families.
/// The selection borrow remains required; this grants no enclosing-SFC admission.
pub struct ObservedNativeTemplateBindingParts<'p, 'a> {
    pub original: &'p NativeTemplateComponent<'a>,
    pub operands: Vec<NativeInterpolationOperand<'a>>,
    pub attributes: Vec<NativeAttributeExpression<'a>>,
    pub bindings: Vec<NativeAttributeBindingExpression<'a>>,
    pub document: Doc<'a>,
}

/// Complete refused transfer of each original prefix and actual observer failure.
/// No partial Doc or completed-template/SFC authority is exposed.
pub struct ObservedNativeTemplateBindingFailureParts<'p, 'a> {
    pub original: &'p NativeTemplateComponent<'a>,
    pub operands: Vec<NativeInterpolationOperand<'a>>,
    pub attributes: Vec<NativeAttributeExpression<'a>>,
    pub bindings: Vec<NativeAttributeBindingExpression<'a>>,
    pub refusal: ObservedNativeTemplateRefusal,
    pub interpolation_failure: Option<NativeInterpolationFailure<'a>>,
    pub attribute_failure: Option<NativeAttributeExpressionFailure<'a>>,
    pub binding_failure: Option<NativeAttributeExpressionFailure<'a>>,
}

pub(super) struct BindingCustody<'a> {
    pub(super) bindings: Vec<NativeAttributeBindingExpression<'a>>,
    pub(super) failure: Option<(Span, usize, NativeAttributeExpressionFailure<'a>)>,
}

impl<'a> BindingCustody<'a> {
    /// None allocates nothing; only real observations or a failure need a box.
    pub(super) fn pack(
        bindings: Vec<NativeAttributeBindingExpression<'a>>,
        failure: Option<(Span, usize, NativeAttributeExpressionFailure<'a>)>,
    ) -> Option<Box<Self>> {
        if bindings.is_empty() && failure.is_none() {
            None
        } else {
            Some(Box::new(Self { bindings, failure }))
        }
    }

    pub(super) fn unpack(custody: Option<Box<Self>>) -> Self {
        custody.map_or_else(
            || Self {
                bindings: Vec::new(),
                failure: None,
            },
            |custody| *custody,
        )
    }
}
