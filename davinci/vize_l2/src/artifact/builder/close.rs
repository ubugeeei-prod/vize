//! Close only the actual factory frame after its normal callback.

use super::{Builder, Frame, Owner};
use crate::op::{ComponentOp, ElementOp, Op, Region};
use vize_l0::Box;

impl<'a> Builder<'a> {
    pub(super) fn close(
        &mut self,
        frame: Frame<'a>,
    ) -> Option<core::ptr::NonNull<crate::op::OriginalForOp<'a>>> {
        let children = Region { ops: frame.ops };
        let bindings = frame.bindings;
        let span = frame.span;
        let mut original = None;
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
            } => Op::Element(Box::new_in(
                ElementOp {
                    tag,
                    namespace,
                    attributes,
                    bindings,
                    children,
                    span,
                },
                &self.allocator,
            )),
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
        original
    }
}
