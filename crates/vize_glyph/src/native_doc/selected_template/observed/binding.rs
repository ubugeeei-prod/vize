//! Complete normal-owned transfers and compact rejected binding custody.

use std::{boxed::Box, vec::Vec};
use vize_l0::Span;
use vize_l1::markup::{
    AttributeBindingExpression, AttributeExpression, AttributeExpressionFailure,
    InterpolationFailure, InterpolationOperand, SelectedTemplateComponent,
};

use super::Doc;
use super::failure::ObservedNativeTemplateRefusal;

/// Complete selected-template transfer, retaining all three original families.
/// The selection borrow remains required; this grants no enclosing-SFC admission.
pub struct ObservedNativeTemplateBindingParts<'p, 'a> {
    /// Original selected template that authorized the observations.
    pub original: &'p SelectedTemplateComponent<'a>,
    /// Interpolation observations in source order.
    pub operands: Vec<InterpolationOperand<'a>>,
    /// Conditional directive observations in source order.
    pub attributes: Vec<AttributeExpression<'a>>,
    /// Binding directive observations in source order.
    pub bindings: Vec<AttributeBindingExpression<'a>>,
    /// Complete document tree joined from those observations.
    pub document: Doc<'a>,
}

/// Complete refused transfer of each original prefix and actual observer failure.
/// No partial Doc or completed-template/SFC authority is exposed.
pub struct ObservedNativeTemplateBindingFailureParts<'p, 'a> {
    /// Original selected template that authorized the observations.
    pub original: &'p SelectedTemplateComponent<'a>,
    /// Successfully observed interpolation prefix.
    pub operands: Vec<InterpolationOperand<'a>>,
    /// Successfully observed conditional directive prefix.
    pub attributes: Vec<AttributeExpression<'a>>,
    /// Successfully observed binding directive prefix.
    pub bindings: Vec<AttributeBindingExpression<'a>>,
    /// Typed reason the complete document was refused.
    pub refusal: ObservedNativeTemplateRefusal,
    /// Original interpolation observer failure, when that caused refusal.
    pub interpolation_failure: Option<InterpolationFailure<'a>>,
    /// Original conditional observer failure, when that caused refusal.
    pub attribute_failure: Option<AttributeExpressionFailure<'a>>,
    /// Original binding observer failure, when that caused refusal.
    pub binding_failure: Option<AttributeExpressionFailure<'a>>,
}

pub(super) struct BindingCustody<'a> {
    pub(super) bindings: Vec<AttributeBindingExpression<'a>>,
    pub(super) failure: Option<(Span, usize, AttributeExpressionFailure<'a>)>,
}

impl<'a> BindingCustody<'a> {
    /// None allocates nothing; only real observations or a failure need a box.
    pub(super) fn pack(
        bindings: Vec<AttributeBindingExpression<'a>>,
        failure: Option<(Span, usize, AttributeExpressionFailure<'a>)>,
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
