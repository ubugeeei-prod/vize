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

use super::surface::{child_start, siblings_at, token_range};
use super::walk::{S2Step, scope_region};
use super::{S2ElementOp, S2Markup};
use crate::ir::ByteRange;
use crate::markup::element::MarkupElement;
use crate::markup::node::{MarkupNode, MarkupText};
use crate::markup::s2_range;
use vize_s0::Span;
use vize_s1::SurfaceChild;
use vize_s2::op::{IfOp, InterpolationOp, Op, TextOp};

/// Visit the child nodes a region contributes, in authored order.
pub(in crate::markup) fn walk_nodes<'a>(
    doc: &'a S2Markup<'a>,
    ops: &'a [Op<'a>],
    visitor: &mut impl FnMut(MarkupNode<'a>),
) {
    let mut index = 0;
    while let Some(op) = ops.get(index) {
        match op {
            Op::Element(_) | Op::Component(_) | Op::Slot(_) => {
                if let Some(element) = S2ElementOp::from_op(op) {
                    visitor(MarkupNode::Element(MarkupElement::from_s2(element, doc)));
                }
            }
            Op::Text(text) => visitor(MarkupNode::Text(text_node(doc, text))),
            Op::Interpolation(interpolation) => walk_interpolation(doc, interpolation, visitor),
            Op::Comment(comment) => visitor(MarkupNode::Comment(s2_range(comment.span))),
            Op::If(if_op) if doc.is_template() => {
                index = walk_chain(doc, ops, index, if_op, visitor);
                continue;
            }
            Op::For(for_op) if doc.is_template() => {
                match scope_carrier(doc, for_op.span, &for_op.region.ops) {
                    Some(carrier) => visitor(MarkupNode::Element(carrier)),
                    None => visitor(MarkupNode::For(doc.open_tag_range(for_op.span))),
                }
            }
            Op::If(if_op) => visitor(MarkupNode::If(if_range(doc, if_op))),
            Op::For(for_op) => visitor(MarkupNode::For(doc.open_tag_range(for_op.span))),
        }
        index += 1;
    }
}

/// The element a template scope was written on: the branch or list carrier
/// op, the `v-for` carrier inside a `v-if` branch, or the unwrapped
/// `<template>` read off S1.
pub(in crate::markup) fn scope_carrier<'a>(
    doc: &'a S2Markup<'a>,
    span: Span,
    region: &'a [Op<'a>],
) -> Option<MarkupElement<'a>> {
    match scope_region(doc, span, region) {
        S2Step::Element(element, _) => Some(element),
        S2Step::Region([Op::For(for_op)]) if for_op.span == span => {
            scope_carrier(doc, for_op.span, &for_op.region.ops)
        }
        S2Step::Region([op]) => S2ElementOp::from_op(op)
            .filter(|element| element.span() == span)
            .map(|element| MarkupElement::from_s2(element, doc)),
        S2Step::Region(_) => None,
    }
}

/// A template `ui.if` as authored: each branch carrier, with the gap comments
/// (read off S1) and kept gap whitespace (the text ops the lowering re-emits
/// right after the scope) back in their authored places. Returns the index
/// past the consumed gap texts.
fn walk_chain<'a>(
    doc: &'a S2Markup<'a>,
    ops: &'a [Op<'a>],
    index: usize,
    if_op: &'a IfOp<'a>,
    visitor: &mut impl FnMut(MarkupNode<'a>),
) -> usize {
    let inside = |span: Span| span.start >= if_op.span.start && span.end <= if_op.span.end;
    let texts_end = index
        + 1
        + ops[index + 1..]
            .iter()
            .take_while(|op| matches!(op, Op::Text(text) if inside(text.span)))
            .count();
    let mut texts = ops[index + 1..texts_end].iter().peekable();
    let siblings = doc
        .surface
        .and_then(|tree| siblings_at(tree, if_op.span.start))
        .unwrap_or(&[]);
    let source = doc.source;
    let mut comments = siblings
        .iter()
        .filter(|child| matches!(child, SurfaceChild::Comment(_)))
        .filter(|child| {
            let start = child_start(source, child);
            start > if_op.span.start && start < if_op.span.end
        })
        .peekable();
    let mut branches = if_op.branches.iter().peekable();
    loop {
        let next_branch = branches.peek().map(|branch| branch.span.start);
        let next_comment = comments.peek().map(|child| child_start(source, child));
        let next_text = texts.peek().and_then(|op| match op {
            Op::Text(text) => Some(text.span.start),
            _ => None,
        });
        let Some(first) = [next_branch, next_comment, next_text]
            .into_iter()
            .flatten()
            .min()
        else {
            break;
        };
        if next_branch == Some(first) {
            let Some(branch) = branches.next() else { break };
            match scope_carrier(doc, branch.span, &branch.region.ops) {
                Some(carrier) => visitor(MarkupNode::Element(carrier)),
                None => walk_nodes(doc, &branch.region.ops, visitor),
            }
        } else if next_comment == Some(first) {
            if let Some(SurfaceChild::Comment(token)) = comments.next() {
                visitor(MarkupNode::Comment(token_range(source, token)));
            }
        } else if let Some(Op::Text(text)) = texts.next() {
            visitor(MarkupNode::Text(text_node(doc, text)));
        }
    }
    texts_end
}

/// Split a merged run back into the text and interpolation parts the
/// lowering recorded for it; a lone interpolation is one node.
pub(in crate::markup) fn walk_interpolation<'a>(
    doc: &'a S2Markup<'a>,
    interpolation: &'a InterpolationOp<'a>,
    visitor: &mut impl FnMut(MarkupNode<'a>),
) {
    let Some(parts) = doc.text_parts(interpolation.span) else {
        visitor(MarkupNode::Interpolation(s2_range(interpolation.span)));
        return;
    };
    for part in &parts.parts {
        let range = s2_range(part.span);
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

/// A `ui.if`'s reported range (see [`S2Markup::scope_range`]).
pub(in crate::markup) fn if_range<'a>(doc: &'a S2Markup<'a>, op: &'a IfOp<'a>) -> ByteRange {
    let last = op.branches.last().map_or(op.span, |branch| branch.span);
    doc.scope_range(op.span, last)
}

/// A `ui.text` op as rendered text: its content entity-decoded with the S2
/// decoder, at its authored range.
pub(in crate::markup) fn text_node<'a>(
    doc: &'a S2Markup<'a>,
    op: &'a TextOp<'a>,
) -> MarkupText<'a> {
    MarkupText::from_static(doc.static_part_text(op.content), s2_range(op.span))
}
