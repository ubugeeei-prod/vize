//! Two inputs to the same original-child document traversal.

use std::vec::Vec;
use vize_l0::Span;
use vize_l1::markup::{
    AttributeBindingExpression, AttributeExpression, AttributeExpressionFailure, AttributeHead,
    StaticBindingHead,
};
use vize_l1::markup::{
    ChildView, InterpolationFailure, InterpolationOperand, NativeTemplateComponent,
};

use super::NativeTemplateRefusal;

pub(super) trait Input<'p, 'a> {
    fn operand(
        &mut self,
        selected: &'p NativeTemplateComponent<'a>,
        child: &ChildView<'p, 'a>,
        index: usize,
        offset: usize,
    ) -> Result<&InterpolationOperand<'a>, NativeTemplateRefusal>;

    fn attribute(
        &mut self,
        head: &AttributeHead<'_, 'a>,
        offset: usize,
    ) -> Result<(usize, &AttributeExpression<'a>), NativeTemplateRefusal>;

    fn binding(
        &mut self,
        head: StaticBindingHead<'_, '_, 'a>,
        offset: usize,
    ) -> Result<(usize, &AttributeBindingExpression<'a>), NativeTemplateRefusal>;
}

pub(super) struct Borrowed<'p, 'a>(pub(super) &'p [&'p InterpolationOperand<'a>]);

impl<'p, 'a> Input<'p, 'a> for Borrowed<'p, 'a> {
    fn operand(
        &mut self,
        _selected: &'p NativeTemplateComponent<'a>,
        _child: &ChildView<'p, 'a>,
        index: usize,
        offset: usize,
    ) -> Result<&InterpolationOperand<'a>, NativeTemplateRefusal> {
        self.0
            .get(index)
            .copied()
            .ok_or(NativeTemplateRefusal::MissingOperand { offset, index })
    }

    fn attribute(
        &mut self,
        _head: &AttributeHead<'_, 'a>,
        offset: usize,
    ) -> Result<(usize, &AttributeExpression<'a>), NativeTemplateRefusal> {
        Err(NativeTemplateRefusal::MissingAttributeOperand { offset, index: 0 })
    }

    fn binding(
        &mut self,
        _head: StaticBindingHead<'_, '_, 'a>,
        offset: usize,
    ) -> Result<(usize, &AttributeBindingExpression<'a>), NativeTemplateRefusal> {
        Err(NativeTemplateRefusal::MissingBindingOperand { offset, index: 0 })
    }
}

pub(in crate::native_doc) struct Observed<'a> {
    pub(in crate::native_doc) operands: Vec<InterpolationOperand<'a>>,
    pub(in crate::native_doc) failure: Option<InterpolationFailure<'a>>,
    pub(in crate::native_doc) attributes: Vec<AttributeExpression<'a>>,
    pub(in crate::native_doc) attribute_failure:
        Option<(Span, usize, AttributeExpressionFailure<'a>)>,
    pub(in crate::native_doc) bindings: Vec<AttributeBindingExpression<'a>>,
    pub(in crate::native_doc) binding_failure:
        Option<(Span, usize, AttributeExpressionFailure<'a>)>,
}

impl<'a> Observed<'a> {
    pub(super) fn new() -> Self {
        Self {
            operands: Vec::new(),
            failure: None,
            attributes: Vec::new(),
            attribute_failure: None,
            bindings: Vec::new(),
            binding_failure: None,
        }
    }
}

impl<'p, 'a> Input<'p, 'a> for Observed<'a> {
    fn operand(
        &mut self,
        selected: &'p NativeTemplateComponent<'a>,
        child: &ChildView<'p, 'a>,
        index: usize,
        offset: usize,
    ) -> Result<&InterpolationOperand<'a>, NativeTemplateRefusal> {
        match selected.observe_interpolation_expression(child.reborrow()) {
            Ok(operand) => self.operands.push(operand),
            Err(failure) => {
                let hole = failure.syntax().and_then(|syntax| syntax.hole());
                self.failure = Some(failure);
                return Err(NativeTemplateRefusal::OperandRejected {
                    offset,
                    index,
                    hole,
                });
            }
        }
        // Park the exact original before admission, source projection or layout.
        self.operands
            .get(index)
            .ok_or(NativeTemplateRefusal::MissingOperand { offset, index })
    }

    fn attribute(
        &mut self,
        head: &AttributeHead<'_, 'a>,
        offset: usize,
    ) -> Result<(usize, &AttributeExpression<'a>), NativeTemplateRefusal> {
        let index = self.attributes.len();
        match head.observe_expression() {
            Ok(operand) => self.attributes.push(operand),
            Err(failure) => {
                let span = head.name_block().span();
                let kind = failure.kind();
                self.attribute_failure = Some((span, index, failure));
                return Err(NativeTemplateRefusal::AttributeObservation { span, index, kind });
            }
        }
        // Park the normal owning original before framing/admission/Doc failure.
        self.attributes
            .get(index)
            .map(|operand| (index, operand))
            .ok_or(NativeTemplateRefusal::MissingAttributeOperand { offset, index })
    }

    fn binding(
        &mut self,
        head: StaticBindingHead<'_, '_, 'a>,
        offset: usize,
    ) -> Result<(usize, &AttributeBindingExpression<'a>), NativeTemplateRefusal> {
        let index = self.bindings.len();
        let span = head.head().name_block().span();
        match head.observe_expression() {
            Ok(operand) => self.bindings.push(operand),
            Err(failure) => {
                let kind = failure.kind();
                self.binding_failure = Some((span, index, failure));
                return Err(NativeTemplateRefusal::BindingObservation { span, index, kind });
            }
        }
        // Consume the sealed view once and park its normal owner before checks.
        self.bindings
            .get(index)
            .map(|operand| (index, operand))
            .ok_or(NativeTemplateRefusal::MissingBindingOperand { offset, index })
    }
}
