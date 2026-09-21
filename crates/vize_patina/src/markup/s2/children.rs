//! Region ops projected as [`MarkupNode`] children.
//!
//! A merged text/interpolation run is split back into its recorded parts, so
//! the child view matches the parsed child list.

use super::{S2ElementOp, S2Markup};
use crate::ir::ByteRange;
use crate::markup::element::MarkupElement;
use crate::markup::node::{MarkupNode, MarkupText};
use crate::markup::s2_range;
use vize_s2::op::{IfOp, InterpolationOp, Op, TextOp};

/// Visit the child nodes a region contributes, in authored order.
pub(in crate::markup) fn walk_nodes<'a>(
    doc: &'a S2Markup<'a>,
    ops: &'a [Op<'a>],
    visitor: &mut impl FnMut(MarkupNode<'a>),
) {
    for op in ops {
        match op {
            Op::Element(_) | Op::Component(_) | Op::Slot(_) => {
                let Some(element) = S2ElementOp::from_op(op) else {
                    continue;
                };
                visitor(MarkupNode::Element(MarkupElement::from_s2(element, doc)));
            }
            Op::Text(text) => visitor(MarkupNode::Text(text_node(doc, text))),
            Op::Interpolation(interpolation) => walk_interpolation(doc, interpolation, visitor),
            Op::Comment(comment) => visitor(MarkupNode::Comment(s2_range(comment.span))),
            Op::If(if_op) => visitor(MarkupNode::If(if_range(doc, if_op))),
            Op::For(for_op) => visitor(MarkupNode::For(doc.open_tag_range(for_op.span))),
        }
    }
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
