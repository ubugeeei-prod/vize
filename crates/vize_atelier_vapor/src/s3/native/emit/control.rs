//! Conditional chains and loops become the shared `If`/`For` IR. Each branch
//! and loop body is a separate block instantiated from its own template.
//!
//! Node numbering reserves one id per block exactly as upstream's compiler
//! does (and the retained lane mirrors), so the published fixture corpus keeps
//! its output parity. Ids are names only; no runtime behavior depends on them.

use vize_carton::{Box, Vec, ensure_sufficient_stack};

use super::super::{AuthoredSpan, Content, Expr};
use super::{Emitter, take};
use crate::ir::{BlockIRNode, ForIRNode, IfIRNode, NegativeBranch, OperationNode};

/// Parent element, insertion anchor, and whether the control op is the
/// parent's only child.
pub(super) type Placement = (usize, usize, bool);

type Branches<'s, 'a> = &'s [(Option<Expr<'a>>, AuthoredSpan, &'s [usize])];

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
        // Each node is emitted once, so its payload moves out of the artifact.
        let operation = match &mut self.artifact.nodes[index].content {
            Content::If { branches } => {
                let branches = take(self.allocator, branches);
                let branches = Vec::from_iter_in(
                    branches
                        .iter()
                        .map(|branch| (branch.condition, branch.span, branch.roots.as_slice())),
                    &self.allocator,
                );
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
                let members = take(self.allocator, &mut self.artifact.nodes[index].children);
                self.id();
                let render = self.body(&members);
                let spans = body.spans;
                let source_span = self.trimmed(spans.source);
                // The legacy walk keys the loop by its carrier element (`<li`
                // or `<template`), which the S3 op span still records.
                if self.source.is_some() {
                    self.units.insert(source_span.0, body.carrier_start);
                }
                let alias = |value: Option<&'a str>, span: Option<AuthoredSpan>| {
                    value.map(|value| {
                        self.spanned(Expr::plain(value), false, span.map(|s| self.trimmed(s)))
                    })
                };
                let node = ForIRNode {
                    id,
                    source: self.spanned(body.source, false, Some(source_span)),
                    value: alias(Some(body.value), spans.aliases[0]),
                    key: alias(body.key, spans.aliases[1]),
                    index: alias(body.index, spans.aliases[2]),
                    key_prop: body.key_prop.map(|key| {
                        self.spanned(key, false, spans.key_prop.map(|s| self.trimmed(s)))
                    }),
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
            _ => unreachable!("control payload checked by the caller"),
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
        let (condition, span, roots) = branches[0];
        let key = self.trimmed(span);
        if self.source.is_some()
            && let Some(start) = self.branch_anchor(key, roots)
        {
            self.units.insert(key.0, start);
        }
        // A trailing `v-else` keeps the carrier element span, so its `<` is
        // the anchor even when the body is an unwrapped template fragment.
        if let [_, (None, else_span, _), ..] = branches
            && self.source.is_some()
        {
            self.else_units.insert(key.0, else_span.0);
        }
        let condition = self.spanned(
            condition.expect("validated leading condition"),
            false,
            Some(key),
        );
        self.id();
        (condition, self.body(roots))
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
            (None, _, roots) => {
                self.id();
                Some(NegativeBranch::Block(self.body(roots)))
            }
            (Some(_), ..) => {
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

    /// Authored `<` of the branch carrier. The condition sits in that open
    /// tag, so the tag's `<` is the same anchor the legacy branch loc uses
    /// for both an element carrier and an unwrapped `<template>`.
    fn branch_anchor(&self, condition: AuthoredSpan, roots: &[usize]) -> Option<u32> {
        self.carrier_lt(condition.0)
            .or_else(|| self.element_start_of(roots))
    }

    /// `<` of a single element root, when the carrier scan cannot see source.
    fn element_start_of(&self, roots: &[usize]) -> Option<u32> {
        let [root] = roots else {
            return None;
        };
        match self.artifact.nodes[*root].content {
            Content::Element { tag_span, .. } => Some(tag_span.0),
            _ => None,
        }
    }

    /// Start of the open tag that contains the authored byte `inside`.
    fn carrier_lt(&self, inside: u32) -> Option<u32> {
        let bytes = self.source?.as_bytes();
        let mut i = inside as usize;
        if i > bytes.len() {
            return None;
        }
        while i > 0 && bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }
        if i == 0 {
            return None;
        }
        i -= 1;
        if matches!(bytes[i], b'"' | b'\'') {
            if i == 0 {
                return None;
            }
            i -= 1;
        }
        let mut quote = None;
        loop {
            let byte = bytes[i];
            match quote {
                Some(q) if byte == q => quote = None,
                Some(_) => {}
                None if byte == b'"' || byte == b'\'' => quote = Some(byte),
                None if byte == b'<' => return Some(i as u32),
                _ => {}
            }
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }
}
