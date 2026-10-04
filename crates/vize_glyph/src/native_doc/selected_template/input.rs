//! Two inputs to the same original-child document traversal.

use std::vec::Vec;
use vize_l1::markup::{
    NativeChild, NativeInterpolationFailure, NativeInterpolationOperand, NativeTemplateComponent,
};

use super::NativeTemplateRefusal;

pub(super) trait Input<'p, 'a> {
    fn operand(
        &mut self,
        selected: &'p NativeTemplateComponent<'a>,
        child: &NativeChild<'p, 'a>,
        index: usize,
        offset: usize,
    ) -> Result<&NativeInterpolationOperand<'a>, NativeTemplateRefusal>;
}

pub(super) struct Borrowed<'p, 'a>(pub(super) &'p [&'p NativeInterpolationOperand<'a>]);

impl<'p, 'a> Input<'p, 'a> for Borrowed<'p, 'a> {
    fn operand(
        &mut self,
        _selected: &'p NativeTemplateComponent<'a>,
        _child: &NativeChild<'p, 'a>,
        index: usize,
        offset: usize,
    ) -> Result<&NativeInterpolationOperand<'a>, NativeTemplateRefusal> {
        self.0
            .get(index)
            .copied()
            .ok_or(NativeTemplateRefusal::MissingOperand { offset, index })
    }
}

pub(super) struct Observed<'a> {
    pub(super) operands: Vec<NativeInterpolationOperand<'a>>,
    pub(super) failure: Option<NativeInterpolationFailure<'a>>,
}

impl<'a> Observed<'a> {
    pub(super) fn new() -> Self {
        Self {
            operands: Vec::new(),
            failure: None,
        }
    }
}

impl<'p, 'a> Input<'p, 'a> for Observed<'a> {
    fn operand(
        &mut self,
        selected: &'p NativeTemplateComponent<'a>,
        child: &NativeChild<'p, 'a>,
        index: usize,
        offset: usize,
    ) -> Result<&NativeInterpolationOperand<'a>, NativeTemplateRefusal> {
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
}
