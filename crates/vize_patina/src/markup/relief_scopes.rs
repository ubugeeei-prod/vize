//! Structured control flow over a raw Relief template parse.
//!
//! A raw parse keeps `v-if` / `v-else-if` / `v-else` / `v-for` as directive
//! attributes on sibling elements. The facade's contract is S2's: control
//! flow is scopes, not attributes. This module groups a raw sibling list the
//! way the S1→S2 lowering does (`vize_s1_to_s2::lower::structural`) — the
//! same chain scan, the same gap rule (whitespace-only text and comments
//! between branches are consumed; kept whitespace re-enters *after* the
//! scope), the same `v-for` admission (a blank value builds no list) — so a
//! rule observes one document whichever backend projected it.

use super::loc_to_range;
use super::node::MarkupText;
use crate::ir::ByteRange;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, TemplateChildNode};

/// Whether a Relief directive is structural (consumed into a scope).
pub(super) fn is_structural_directive(directive: &DirectiveNode<'_>) -> bool {
    matches!(directive.name, "if" | "else-if" | "else" | "for")
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BranchKind {
    If,
    ElseIf,
    Else,
}

/// The first branch directive an element carries (the lowering's
/// `analyze`: the first of `v-if` / `v-else-if` / `v-else` wins).
pub(super) fn branch_of<'a>(
    element: &'a ElementNode<'a>,
) -> Option<(BranchKind, &'a DirectiveNode<'a>)> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(directive) => match directive.name {
            "if" => Some((BranchKind::If, &**directive)),
            "else-if" => Some((BranchKind::ElseIf, &**directive)),
            "else" => Some((BranchKind::Else, &**directive)),
            _ => None,
        },
        PropNode::Attribute(_) => None,
    })
}

/// The element's first `v-for`, when its value is non-blank (the lowering
/// builds no `ui.for` for a missing or blank value).
pub(super) fn list_of<'a>(element: &'a ElementNode<'a>) -> Option<&'a DirectiveNode<'a>> {
    let directive = element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(directive) if directive.name == "for" => Some(&**directive),
        _ => None,
    })?;
    match directive.exp.as_ref() {
        Some(ExpressionNode::Simple(value)) if !value.content.trim().is_empty() => Some(directive),
        Some(ExpressionNode::Compound(_)) => Some(directive),
        _ => None,
    }
}

fn is_gap(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Comment(_) => true,
        TemplateChildNode::Text(text) => text.content.trim().is_empty(),
        _ => false,
    }
}

/// A `v-if` chain synthesized over raw siblings: `children[start..end]`
/// holds its branch elements and the gaps they consume.
#[derive(Clone, Copy)]
pub(super) struct ReliefChain<'a> {
    pub(super) children: &'a [TemplateChildNode<'a>],
    pub(super) start: usize,
    pub(super) end: usize,
}

impl<'a> ReliefChain<'a> {
    /// Scan the chain opened by the `v-if` element at `start`.
    pub(super) fn scan(children: &'a [TemplateChildNode<'a>], start: usize) -> Self {
        let mut index = start + 1;
        let mut pending = 0usize;
        while let Some(child) = children.get(index) {
            if is_gap(child) {
                pending += 1;
                index += 1;
                continue;
            }
            let TemplateChildNode::Element(element) = child else {
                break;
            };
            match branch_of(element) {
                Some((BranchKind::ElseIf, _)) => {
                    pending = 0;
                    index += 1;
                }
                Some((BranchKind::Else, _)) => {
                    pending = 0;
                    index += 1;
                    break;
                }
                _ => break,
            }
        }
        Self {
            children,
            start,
            end: index - pending,
        }
    }

    /// The branch elements, in authored order.
    pub(super) fn walk_branches(&self, visitor: &mut impl FnMut(&'a ElementNode<'a>)) {
        for child in self.children.get(self.start..self.end).unwrap_or_default() {
            if let TemplateChildNode::Element(element) = child
                && branch_of(element).is_some()
            {
                visitor(element);
            }
        }
    }

    /// Whitespace the chain consumed as gaps but whitespace condensing kept;
    /// the lowering re-emits it after the scope.
    pub(super) fn walk_kept_gaps(&self, visitor: &mut impl FnMut(MarkupText<'a>)) {
        for child in self.children.get(self.start..self.end).unwrap_or_default() {
            if let TemplateChildNode::Text(text) = child {
                visitor(MarkupText::from_relief(text));
            }
        }
    }

    pub(super) fn branch_count(&self) -> usize {
        let mut count = 0;
        self.walk_branches(&mut |_| count += 1);
        count
    }

    /// Whether some branch is an unconditional `v-else` (no non-blank value).
    pub(super) fn has_else(&self) -> bool {
        let mut found = false;
        self.walk_branches(&mut |element| {
            if let Some((BranchKind::Else, directive)) = branch_of(element) {
                let blank = match directive.exp.as_ref() {
                    Some(ExpressionNode::Simple(value)) => value.content.trim().is_empty(),
                    Some(ExpressionNode::Compound(_)) => false,
                    None => true,
                };
                found |= blank;
            }
        });
        found
    }

    pub(super) fn range(&self) -> ByteRange {
        let mut start = None;
        let mut end = 0;
        self.walk_branches(&mut |element| {
            start.get_or_insert(element.loc.span.start);
            end = element.loc.span.end;
        });
        ByteRange::new(start.unwrap_or(end), end)
    }
}

/// An element's reported range: its location, widened to cover a slot
/// spelling that lies outside it. Only a synthesized slot carrier has one — a
/// JSX `v-slots` entry lowers to a `<template>` whose `v-slot` is the entry
/// key, ahead of the rendered body — and S2 widens that op the same way
/// (a binding stays inside its owner, `S2V002`). An authored template's
/// `v-slot` sits in its own opening tag, so the location is unchanged.
pub(super) fn carrier_range(element: &ElementNode<'_>) -> ByteRange {
    let mut range = loc_to_range(&element.loc);
    for prop in &element.props {
        if let PropNode::Directive(directive) = prop
            && directive.name == "slot"
        {
            range.start = range.start.min(directive.loc.span.start);
            range.end = range.end.max(directive.loc.span.end);
        }
    }
    range
}
