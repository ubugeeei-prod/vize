//! Slot functions and checked structural slot carriers.

use super::BlockIRNode;
use vize_atelier_core::SimpleExpressionNode;
use vize_carton::Box;

/// A slot function, optionally selected by a conditional or a slot loop.
#[derive(Debug)]
pub struct IRSlot<'a> {
    pub name: Box<'a, SimpleExpressionNode<'a>>,
    pub fn_exp: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub block: BlockIRNode<'a>,
    pub control: Option<IRSlotControl<'a>>,
}

#[derive(Debug)]
pub enum IRSlotControl<'a> {
    If {
        condition: Box<'a, SimpleExpressionNode<'a>>,
        negative: Option<Box<'a, IRSlot<'a>>>,
    },
    For(IRSlotLoop<'a>),
}

#[derive(Debug)]
pub struct IRSlotLoop<'a> {
    pub source: Box<'a, SimpleExpressionNode<'a>>,
    pub value: Box<'a, SimpleExpressionNode<'a>>,
    pub key: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub index: Option<Box<'a, SimpleExpressionNode<'a>>>,
    pub key_prop: Option<Box<'a, SimpleExpressionNode<'a>>>,
}

impl<'a> IRSlot<'a> {
    pub(crate) fn dynamic(&self) -> bool {
        !self.name.is_static || self.control.is_some()
    }

    /// All branch bodies, including the final unconditional branch.
    pub(crate) fn blocks(&self) -> impl Iterator<Item = &BlockIRNode<'a>> {
        std::iter::successors(Some(self), |slot| match &slot.control {
            Some(IRSlotControl::If {
                negative: Some(next),
                ..
            }) => Some(next.as_ref()),
            _ => None,
        })
        .map(|slot| &slot.block)
    }
}
