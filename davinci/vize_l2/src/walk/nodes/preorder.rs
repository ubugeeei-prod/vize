//! Preorder-only traversal of sealed markup, sharing control-flow numbering
//! with the full nested event walk.
use super::{NodeEvent, NodeLimit, NodeRef, PageWalk, visit_events};
use crate::op::{BindingOp, Op};
use vize_l0::{ensure_sufficient_stack, id::NodeId};

pub(crate) fn visit_sealed_nodes<'s, 'a>(
    walk: &mut PageWalk,
    mut ops: &'s [Op<'a>],
    visit: &mut impl FnMut(NodeId, NodeRef<'s, 'a>),
) -> Result<(), NodeLimit> {
    while let Some((op, siblings)) = ops.split_first() {
        let (bindings, children): (&[BindingOp<'a>], &[Op<'a>]) = match op {
            Op::Element(element) => (&element.bindings, &element.children.ops),
            Op::Component(component) => (&component.bindings, &component.children.ops),
            Op::Text(_) | Op::Interpolation(_) | Op::ChildValue(_) | Op::Comment(_) => (&[], &[]),
            // The original event walker owns the shape of branches, loop
            // fallbacks and slots. It mints the wrapper and all its descendants.
            Op::If(_) | Op::And(_) | Op::For(_) | Op::OriginalFor(_) | Op::Slot(_) => {
                visit_control(walk, op, visit)?;
                ops = siblings;
                continue;
            }
        };
        visit(walk.mint().ok_or(NodeLimit)?, NodeRef::Op(op));
        for binding in bindings {
            visit(
                walk.mint_attached().ok_or(NodeLimit)?,
                NodeRef::Binding(binding),
            );
        }
        // A preorder callback has no leave event pending. The last child's
        // region can therefore reuse this frame, even on a deeply nested file.
        if siblings.is_empty() {
            ops = children;
        } else {
            match children {
                [] => {}
                [
                    leaf
                    @ (Op::Text(_) | Op::Interpolation(_) | Op::ChildValue(_) | Op::Comment(_)),
                ] => {
                    visit(walk.mint().ok_or(NodeLimit)?, NodeRef::Op(leaf));
                }
                _ => ensure_sufficient_stack(|| visit_sealed_nodes(walk, children, visit))?,
            }
            ops = siblings;
        }
    }
    Ok(())
}

// One shared structural walker keeps each markup consumer from instantiating
// another complete nested event implementation for the control fallback.
#[inline]
fn visit_control<'s, 'a>(
    walk: &mut PageWalk,
    op: &'s Op<'a>,
    visit: &mut dyn FnMut(NodeId, NodeRef<'s, 'a>),
) -> Result<(), NodeLimit> {
    visit_events(walk, core::slice::from_ref(op), &mut |event| {
        if let NodeEvent::Enter { id, node, .. } = event {
            visit(id, node);
        }
    })
}
