//! Close only the actual factory frame after its normal callback.

use super::{Builder, Frame, Owner, element::ElementAllocation};
use crate::op::{ComponentOp, ElementOp, Op, Region};
use vize_l0::Box;

pub(super) struct ClosedFrame<'a> {
    pub(super) native_if: Option<core::ptr::NonNull<crate::op::IfOp<'a>>>,
    pub(super) original_for: Option<core::ptr::NonNull<crate::op::OriginalForOp<'a>>>,
    pub(super) element: Option<ElementAllocation<'a>>,
}

impl<'a> Builder<'a> {
    pub(super) fn close(&mut self, frame: Frame<'a>) -> ClosedFrame<'a> {
        let children = Region { ops: frame.ops };
        let bindings = frame.bindings;
        let span = frame.span;
        let mut original = None;
        let mut native_if = None;
        let mut element = None;
        let op = match frame.owner {
            Owner::NativeIf(condition) => {
                let mut branches = vize_l0::Vec::new_in(&self.allocator);
                branches.push(crate::op::IfBranch {
                    condition: Some(crate::expr::ExprRef::Js(condition)),
                    region: children,
                    span,
                });
                let owner = Box::new_in(crate::op::IfOp { branches, span }, &self.allocator);
                native_if = Some(core::ptr::NonNull::from(owner.as_ref()));
                Op::If(owner)
            }
            Owner::NativeIfElse {
                condition,
                first,
                first_span,
                else_span,
            } => {
                let mut branches = vize_l0::Vec::new_in(&self.allocator);
                branches.push(crate::op::IfBranch {
                    condition: Some(crate::expr::ExprRef::Js(condition)),
                    region: first,
                    span: first_span,
                });
                branches.push(crate::op::IfBranch {
                    condition: None,
                    region: children,
                    span: else_span,
                });
                let owner = Box::new_in(crate::op::IfOp { branches, span }, &self.allocator);
                native_if = Some(core::ptr::NonNull::from(owner.as_ref()));
                Op::If(owner)
            }
            Owner::OriginalFor(id) => {
                let owner = Box::new_in(
                    crate::op::OriginalForOp {
                        id,
                        region: children,
                        span,
                    },
                    &self.allocator,
                );
                original = Some(core::ptr::NonNull::from(owner.as_ref()));
                Op::OriginalFor(owner)
            }
            Owner::Element {
                tag,
                namespace,
                attributes,
            } => {
                let owner = Box::new_in(
                    ElementOp {
                        tag,
                        namespace,
                        attributes,
                        bindings,
                        children,
                        span,
                    },
                    &self.allocator,
                );
                element = Some(ElementAllocation::original(owner.as_ref()));
                Op::Element(owner)
            }
            Owner::Component { name, attributes } => Op::Component(Box::new_in(
                ComponentOp {
                    name,
                    attributes,
                    bindings,
                    children,
                    span,
                },
                &self.allocator,
            )),
        };
        self.push(op);
        ClosedFrame {
            native_if,
            original_for: original,
            element,
        }
    }
}
