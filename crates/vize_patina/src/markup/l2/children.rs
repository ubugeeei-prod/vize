//! Region ops projected as the authored [`MarkupNode`] child list.
//!
//! `walk_children` presents what the author wrote under an element. In a
//! template an element that carries `v-if` / `v-for` is itself the child (the
//! scope structure is what the visitor's scope hooks carry), so a `ui.if` is
//! read back as its branch carriers, interleaved with the gap comments and
//! kept whitespace the lowering consumed between them, and a `ui.for` as its
//! carrier. A JSX conditional or list is an expression, so a JSX projection's
//! `ui.if` / `ui.for` stays one [`MarkupNode::If`] / [`MarkupNode::For`]. A
//! merged text/interpolation run is split back into its recorded parts.

use super::surface::{child_start, siblings_with_pre, token_range};
use super::walk::{L2Step, scope_region};
use super::{L2ElementOp, L2Markup};
use crate::ir::ByteRange;
use crate::markup::element::MarkupElement;
use crate::markup::l2_range;
use crate::markup::node::{MarkupNode, MarkupText};
use vize_l0::Span;
use vize_l1::SurfaceChild;
use vize_l2::op::{IfOp, InterpolationOp, Op, TextOp};

/// Visit the child nodes a region contributes, in authored order.
///
/// Keep this reconstruction shared across rule callbacks, including recursive
/// branch walks. The borrowed callback adds no allocation to the traversal.
pub(in crate::markup) fn walk_nodes<'a>(
    doc: &'a L2Markup<'a>,
    ops: &'a [Op<'a>],
    visitor: &mut dyn FnMut(MarkupNode<'a>),
) {
    let mut index = 0;
    while let Some(op) = ops.get(index) {
        match op {
            Op::Element(_) | Op::Component(_) | Op::Slot(_) => {
                if let Some(element) = L2ElementOp::from_op(op) {
                    visitor(MarkupNode::Element(MarkupElement::from_l2(element, doc)));
                }
            }
            Op::Text(text) => visitor(MarkupNode::Text(text_node(doc, text))),
            Op::Interpolation(interpolation) => walk_interpolation(doc, interpolation, visitor),
            Op::Comment(comment) => visitor(MarkupNode::Comment(l2_range(comment.span))),
            Op::If(if_op) if doc.is_template() => {
                walk_chain(doc, if_op, visitor);
            }
            Op::For(for_op) if doc.is_template() => {
                match scope_carrier(doc, for_op.span, &for_op.region.ops) {
                    Some(carrier) => visitor(MarkupNode::Element(carrier)),
                    None => visitor(MarkupNode::For(doc.open_tag_range(for_op.span))),
                }
            }
            Op::OriginalFor(for_op) if doc.is_template() => {
                match scope_carrier(doc, for_op.span, &for_op.region.ops) {
                    Some(carrier) => visitor(MarkupNode::Element(carrier)),
                    None => visitor(MarkupNode::For(doc.open_tag_range(for_op.span))),
                }
            }
            Op::If(if_op) => visitor(MarkupNode::If(if_range(doc, if_op))),
            Op::For(for_op) => visitor(MarkupNode::For(doc.open_tag_range(for_op.span))),
            Op::OriginalFor(for_op) => visitor(MarkupNode::For(doc.open_tag_range(for_op.span))),
        }
        index += 1;
    }
}

/// The element a template scope was written on: the branch or list carrier
/// op, the `v-for` carrier inside a `v-if` branch, or the unwrapped
/// `<template>` read off L1.
pub(in crate::markup) fn scope_carrier<'a>(
    doc: &'a L2Markup<'a>,
    span: Span,
    region: &'a [Op<'a>],
) -> Option<MarkupElement<'a>> {
    match scope_region(doc, span, region) {
        L2Step::Element(element, _) => Some(element),
        L2Step::Region([Op::For(for_op)]) if for_op.span == span => {
            scope_carrier(doc, for_op.span, &for_op.region.ops)
        }
        L2Step::Region([Op::OriginalFor(for_op)]) if for_op.span == span => {
            scope_carrier(doc, for_op.span, &for_op.region.ops)
        }
        L2Step::Region([op]) => L2ElementOp::from_op(op)
            .filter(|element| element.span() == span)
            .map(|element| MarkupElement::from_l2(element, doc)),
        L2Step::Region(_) => None,
    }
}

/// A template `ui.if` as authored: branch carriers, comments and parse-time
/// gap whitespace read off L1. Compiler regions drop gaps; the lint facade
/// retains its original authored view and the lowering's whitespace policy.
fn walk_chain<'a>(
    doc: &'a L2Markup<'a>,
    if_op: &'a IfOp<'a>,
    visitor: &mut dyn FnMut(MarkupNode<'a>),
) {
    let (siblings, in_pre) = doc
        .surface
        .and_then(|tree| siblings_with_pre(tree, if_op.span.start))
        .unwrap_or((&[], false));
    let source = doc.source;
    let mut gaps = siblings
        .iter()
        .filter(|child| matches!(child, SurfaceChild::Comment(_) | SurfaceChild::Text(_)))
        .filter(|child| {
            let start = child_start(source, child);
            start > if_op.span.start && start < if_op.span.end
        })
        .peekable();
    let mut branches = if_op.branches.iter().peekable();
    loop {
        let next_branch = branches.peek().map(|branch| branch.span.start);
        let next_gap = gaps.peek().map(|child| child_start(source, child));
        let Some(first) = [next_branch, next_gap].into_iter().flatten().min() else {
            break;
        };
        if next_branch == Some(first) {
            let Some(branch) = branches.next() else { break };
            match scope_carrier(doc, branch.span, &branch.region.ops) {
                Some(carrier) => visitor(MarkupNode::Element(carrier)),
                None => walk_nodes(doc, &branch.region.ops, visitor),
            }
        } else {
            match gaps.next() {
                Some(SurfaceChild::Comment(token)) => {
                    visitor(MarkupNode::Comment(token_range(source, token)));
                }
                Some(SurfaceChild::Text(token)) => {
                    if let Some(allocator) = doc.allocator
                        && let Some(content) =
                            vize_l1_to_l2::lower::branch_gap_text(allocator, token.text, in_pre)
                    {
                        visitor(MarkupNode::Text(MarkupText::from_static(
                            content,
                            token_range(source, token),
                        )));
                    }
                }
                _ => {}
            }
        }
    }
}

/// Kept authored gap text follows the branch visits, matching the raw facade's
/// text hook order even though the compiler region contains no gap text ops.
pub(in crate::markup) fn walk_kept_gaps<'a>(
    doc: &'a L2Markup<'a>,
    if_op: &'a IfOp<'a>,
    visitor: &mut impl FnMut(MarkupText<'a>),
) {
    let Some(allocator) = doc.allocator else {
        return;
    };
    let Some((siblings, in_pre)) = doc
        .surface
        .and_then(|tree| siblings_with_pre(tree, if_op.span.start))
    else {
        return;
    };
    for child in siblings {
        if let SurfaceChild::Text(token) = child {
            let start = child_start(doc.source, child);
            if start > if_op.span.start
                && start < if_op.span.end
                && let Some(content) =
                    vize_l1_to_l2::lower::branch_gap_text(allocator, token.text, in_pre)
            {
                visitor(MarkupText::from_static(
                    content,
                    token_range(doc.source, token),
                ));
            }
        }
    }
}

/// Split a merged run back into the text and interpolation parts the
/// lowering recorded for it; a lone interpolation is one node.
pub(in crate::markup) fn walk_interpolation<'a>(
    doc: &'a L2Markup<'a>,
    interpolation: &'a InterpolationOp<'a>,
    visitor: &mut (impl FnMut(MarkupNode<'a>) + ?Sized),
) {
    let Some(parts) = doc.text_parts(interpolation.span) else {
        visitor(MarkupNode::Interpolation(l2_range(interpolation.span)));
        return;
    };
    for part in &parts.parts {
        let range = l2_range(part.span);
        if part.dynamic {
            visitor(MarkupNode::Interpolation(range));
        } else {
            visitor(MarkupNode::Text(MarkupText::from_static(
                doc.static_part_text(part.text.as_str()),
                range,
            )));
        }
    }
}

/// A `ui.if`'s reported range (see [`L2Markup::scope_range`]).
pub(in crate::markup) fn if_range<'a>(doc: &'a L2Markup<'a>, op: &'a IfOp<'a>) -> ByteRange {
    let last = op.branches.last().map_or(op.span, |branch| branch.span);
    doc.scope_range(op.span, last)
}

/// A `ui.text` op as rendered text: its content entity-decoded with the L2
/// decoder, at its authored range.
pub(in crate::markup) fn text_node<'a>(
    doc: &'a L2Markup<'a>,
    op: &'a TextOp<'a>,
) -> MarkupText<'a> {
    MarkupText::from_static(doc.static_part_text(op.content), l2_range(op.span))
}
