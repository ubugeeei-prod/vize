//! The immutable numbered tree surface; enter/leave share one id derivation.

use vize_l0::{Span, ensure_sufficient_stack, id::NodeId};

use super::PageWalk;
use crate::op::{BindingOp, Op};

mod surface;

/// A numbered node borrowed from an L2 artifact.
#[derive(Debug, Clone, Copy)]
pub enum NodeRef<'s, 'a> {
    /// A region op.
    Op(&'s Op<'a>),
    /// An attached binding, numbered immediately after its owner.
    Binding(&'s BindingOp<'a>),
}

/// An unsealed tree has more nodes than the stage-local id space admits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeLimit;

/// One event in a nested, page-ordered traversal.
#[derive(Debug, Clone, Copy)]
pub enum NodeEvent<'s, 'a> {
    /// The node before its attached bindings and child regions.
    Enter {
        /// Its artifact-local page id.
        id: NodeId,
        /// The borrowed node.
        node: NodeRef<'s, 'a>,
        /// The region owner, or the owner of an attached binding.
        parent: Option<NodeId>,
        /// The immediate source owner (an if branch has no node id).
        owner_span: Option<Span>,
    },
    /// The same node after all of its numbered descendants.
    Leave {
        /// The same id as its enter event; never minted twice.
        id: NodeId,
        /// The same borrowed node.
        node: NodeRef<'s, 'a>,
        /// The same owner as its enter event.
        parent: Option<NodeId>,
    },
}

/// Visit both region ops and attached bindings in dense page preorder.
///
/// Unsealed trees may exhaust the id space; the error preserves the
/// successfully visited prefix without publishing an unnumbered node.
pub fn visit_nodes<'s, 'a>(
    walk: &mut PageWalk,
    ops: &'s [Op<'a>],
    visit: &mut impl FnMut(NodeId, NodeRef<'s, 'a>),
) -> Result<(), NodeLimit> {
    visit_events(walk, ops, &mut |event| {
        if let NodeEvent::Enter { id, node, .. } = event {
            visit(id, node);
        }
    })
}

/// Visit a nested tree once, retaining each enter id for its leave event.
///
/// Attached bindings enter/leave before owned regions; if branches follow
/// authored order. Attribute and branch rows have no ids. No expressions
/// are parsed and no owned dump is materialized.
pub fn visit_events<'s, 'a>(
    walk: &mut PageWalk,
    ops: &'s [Op<'a>],
    visit: &mut impl FnMut(NodeEvent<'s, 'a>),
) -> Result<(), NodeLimit> {
    regions(walk, ops, None, None, visit)
}

fn regions<'s, 'a>(
    walk: &mut PageWalk,
    ops: &'s [Op<'a>],
    parent: Option<NodeId>,
    owner_span: Option<Span>,
    visit: &mut impl FnMut(NodeEvent<'s, 'a>),
) -> Result<(), NodeLimit> {
    ensure_sufficient_stack(|| {
        for op in ops {
            let id = walk.mint().ok_or(NodeLimit)?;
            let node = NodeRef::Op(op);
            visit(NodeEvent::Enter {
                id,
                node,
                parent,
                owner_span,
            });
            let span = node.span();
            match op {
                Op::Element(element) => {
                    bindings(walk, &element.bindings, id, span, visit)?;
                    regions(walk, &element.children.ops, Some(id), Some(span), visit)?;
                }
                Op::Component(component) => {
                    bindings(walk, &component.bindings, id, span, visit)?;
                    regions(walk, &component.children.ops, Some(id), Some(span), visit)?;
                }
                Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => {}
                Op::If(if_op) => {
                    for branch in &if_op.branches {
                        regions(walk, &branch.region.ops, Some(id), Some(branch.span), visit)?;
                    }
                }
                Op::For(for_op) => regions(walk, &for_op.region.ops, Some(id), Some(span), visit)?,
                Op::OriginalFor(for_op) => {
                    regions(walk, &for_op.region.ops, Some(id), Some(span), visit)?
                }
                Op::Slot(slot) => {
                    bindings(walk, &slot.bindings, id, span, visit)?;
                    regions(walk, &slot.fallback.ops, Some(id), Some(span), visit)?;
                }
            }
            visit(NodeEvent::Leave { id, node, parent });
        }
        Ok(())
    })
}

fn bindings<'s, 'a>(
    walk: &mut PageWalk,
    bindings: &'s [BindingOp<'a>],
    parent: NodeId,
    owner_span: Span,
    visit: &mut impl FnMut(NodeEvent<'s, 'a>),
) -> Result<(), NodeLimit> {
    for binding in bindings {
        let id = walk.mint_attached().ok_or(NodeLimit)?;
        let node = NodeRef::Binding(binding);
        visit(NodeEvent::Enter {
            id,
            node,
            parent: Some(parent),
            owner_span: Some(owner_span),
        });
        visit(NodeEvent::Leave {
            id,
            node,
            parent: Some(parent),
        });
    }
    Ok(())
}
