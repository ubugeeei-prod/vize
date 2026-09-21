//! Conditional chains and loops become the shared `If`/`For` IR. Each branch
//! and loop body is a separate block instantiated from its own template.
//!
//! Node numbering reserves one id per block exactly as upstream's compiler
//! does (and the retained lane mirrors), so the published fixture corpus keeps
//! its output parity. Ids are names only; no runtime behavior depends on them.

use vize_carton::{Box, ensure_sufficient_stack};

use super::super::{Content, Expr};
use super::Emitter;
use crate::ir::{BlockIRNode, ForIRNode, IfIRNode, NegativeBranch, OperationNode};

/// Parent element, insertion anchor, and whether the control op is the
/// parent's only child.
pub(super) type Placement = (usize, usize, bool);

type Branches<'s, 'a> = &'s [(Option<Expr<'a>>, usize)];

impl<'a> Emitter<'a, '_> {
    pub(super) fn control(
        &mut self,
        index: usize,
        placement: Option<Placement>,
        block: &mut BlockIRNode<'a>,
    ) -> usize {
        let id = self.id();
        let (parent, anchor) = placement.map_or((None, None), |(parent, anchor, _)| {
            (Some(parent), Some(anchor))
        });
        let operation = match &self.artifact.nodes[index].content {
            Content::If { branches } => {
                let branches: std::vec::Vec<_> = branches
                    .iter()
                    .map(|branch| {
                        (
                            branch.condition,
                            branch.root.expect("validated branch root"),
                        )
                    })
                    .collect();
                let (condition, positive) = self.branch(&branches);
                let negative = self.remaining(&branches[1..], parent, anchor);
                let node = IfIRNode {
                    id,
                    condition,
                    positive,
                    negative,
                    once: false,
                    parent,
                    anchor,
                };
                OperationNode::If(Box::new_in(node, &self.allocator))
            }
            Content::For(body) => {
                let body = *body;
                let root = self.artifact.nodes[index].children[0];
                self.id();
                let render = self.block(root);
                let alias = |value: Option<&'a str>| {
                    value.map(|value| self.expression(Expr::plain(value), false))
                };
                let node = ForIRNode {
                    id,
                    source: self.expression(body.source, false),
                    value: alias(Some(body.value)),
                    key: alias(body.key),
                    index: alias(body.index),
                    key_prop: body.key_prop.map(|key| self.expression(key, false)),
                    render,
                    once: false,
                    component: false,
                    // Upstream's fast-remove flag clears the whole parent, so it
                    // is sound only when the loop is the parent's sole child.
                    only_child: placement.is_some_and(|(_, _, only)| only),
                    parent,
                    anchor,
                    match_scope: false,
                };
                OperationNode::For(Box::new_in(node, &self.allocator))
            }
            Content::Element { .. } | Content::Text { .. } => {
                unreachable!("control payload checked by the caller")
            }
        };
        block.operation.push(operation);
        id
    }

    /// The leading conditional branch of `branches` and its block.
    fn branch(
        &mut self,
        branches: Branches<'_, 'a>,
    ) -> (
        vize_carton::Box<'a, vize_atelier_core::SimpleExpressionNode<'a>>,
        BlockIRNode<'a>,
    ) {
        let (condition, root) = branches[0];
        let condition = self.expression(condition.expect("validated leading condition"), false);
        self.id();
        (condition, self.block(root))
    }

    /// Chained branches share the chain's placement. An inline `v-else-if`
    /// has no id of its own; the shared generator emits it in place.
    fn remaining(
        &mut self,
        branches: Branches<'_, 'a>,
        parent: Option<usize>,
        anchor: Option<usize>,
    ) -> Option<NegativeBranch<'a>> {
        ensure_sufficient_stack(|| match branches.first()? {
            (None, root) => {
                self.id();
                Some(NegativeBranch::Block(self.block(*root)))
            }
            (Some(_), _) => {
                let (condition, positive) = self.branch(branches);
                let negative = if branches.len() > 1 {
                    self.id();
                    self.remaining(&branches[1..], parent, anchor)
                } else {
                    None
                };
                let node = IfIRNode {
                    id: 0,
                    condition,
                    positive,
                    negative,
                    once: false,
                    parent,
                    anchor,
                };
                Some(NegativeBranch::If(Box::new_in(node, &self.allocator)))
            }
        })
    }
}
