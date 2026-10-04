//! Close only the actual factory frame after its normal callback.

use super::{Builder, Frame, Owner, element::ElementAllocation};
use crate::op::{ComponentOp, ElementOp, Op, Region};
use vize_l0::Box;

pub(super) struct ClosedFrame<'a> {
    pub(super) original_for: Option<core::ptr::NonNull<crate::op::OriginalForOp<'a>>>,
    pub(super) element: Option<ElementAllocation<'a>>,
}

impl<'a> Builder<'a> {
    pub(super) fn close(&mut self, frame: Frame<'a>) -> ClosedFrame<'a> {
        let children = Region { ops: frame.ops };
        let bindings = frame.bindings;
        let span = frame.span;
        let mut original = None;
        let mut element = None;
        let op = match frame.owner {
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
            original_for: original,
            element,
        }
    }
}
