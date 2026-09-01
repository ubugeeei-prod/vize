use vize_s0::Span;
use vize_s2::op::{BindingOp, Region, SlotContentOp};

use super::is_slot_template;
use crate::emit::EmitCx;
use crate::emit::create_slots_walk::{advance_after_op, slot_content};
use crate::emit::js::{escape_js_string, is_valid_js_identifier};
use crate::pass::walk::PageWalk;
use crate::pass::{SlotCarrier, SlotName, SlotParams};

pub(super) fn emit_slot_key(cx: &mut EmitCx<'_>, name: &SlotName) {
    match name {
        SlotName::Static { text, .. } if is_valid_js_identifier(text.as_str()) => {
            cx.buf.push(text.as_str());
        }
        SlotName::Static { text, .. } => {
            cx.buf.push("\"");
            cx.buf.push(escape_js_string(text.as_str()).as_str());
            cx.buf.push("\"");
        }
        SlotName::Dynamic { text } => {
            cx.buf.push("[");
            cx.buf.push(text.as_str());
            cx.buf.push("]");
        }
    }
}

pub(super) fn group_slot_content<'a>(
    start_walk: &PageWalk,
    children: &'a Region<'a>,
    component_bindings: &'a [BindingOp<'a>],
    carrier: SlotCarrier,
) -> Option<&'a SlotContentOp<'a>> {
    match carrier {
        SlotCarrier::Component => component_bindings.iter().find_map(|binding| match binding {
            BindingOp::SlotContent(content) => Some(&**content),
            _ => None,
        }),
        SlotCarrier::Template(id) => template_slot_content(start_walk, children, id),
        SlotCarrier::Implicit => None,
    }
}

fn template_slot_content<'a>(
    start_walk: &PageWalk,
    children: &'a Region<'a>,
    id: Option<vize_davinci::id::NodeId>,
) -> Option<&'a SlotContentOp<'a>> {
    let mut walk = start_walk.clone();
    for op in children.ops.iter() {
        let op_id = walk.mint();
        if op_id == id
            && let vize_s2::op::Op::Element(element) = op
            && is_slot_template(element)
        {
            return slot_content(element);
        }
        advance_after_op(&mut walk, op);
    }
    None
}

pub(super) fn emit_slot_params(
    cx: &mut EmitCx<'_>,
    params: &SlotParams,
    content: Option<&SlotContentOp<'_>>,
) {
    match params {
        SlotParams::Absent => cx.buf.push("()"),
        SlotParams::Scoped { text, .. } => {
            cx.buf.push("(");
            if let Some((leading, trailing)) = content
                .and_then(|content| content.params.as_ref().map(|expr| (content, expr)))
                .and_then(|(content, expr)| {
                    authored_expr_padding(cx.source, content.span, text.as_str(), expr.span())
                })
            {
                cx.buf.push(leading);
                cx.buf.push(text.as_str());
                cx.buf.push(trailing);
            } else {
                cx.buf.push(text.as_str());
            }
            cx.buf.push(")");
        }
    }
}

fn authored_expr_padding<'a>(
    source: &'a str,
    owner_span: Span,
    value: &str,
    value_span: Span,
) -> Option<(&'a str, &'a str)> {
    let attr_start = usize::try_from(owner_span.start).ok()?;
    let attr_end = usize::try_from(owner_span.end).ok()?;
    let value_start = usize::try_from(value_span.start).ok()?;
    let value_end = usize::try_from(value_span.end).ok()?;
    if attr_start > value_start
        || value_start > value_end
        || value_end > attr_end
        || attr_end > source.len()
        || source.get(value_start..value_end)? != value
    {
        return None;
    }
    let before = source.get(attr_start..value_start)?;
    let quote_pos = before
        .as_bytes()
        .iter()
        .rposition(|byte| matches!(*byte, b'\'' | b'"'))?;
    let quote = before.as_bytes()[quote_pos];
    let leading = before.get(quote_pos + 1..)?;
    let after = source.get(value_end..attr_end)?;
    let trailing_end = after
        .as_bytes()
        .iter()
        .position(|byte| *byte == quote)
        .unwrap_or(after.len());
    let trailing = after.get(..trailing_end)?;
    if leading.is_empty() && trailing.is_empty() {
        return None;
    }
    (leading.bytes().all(|byte| byte.is_ascii_whitespace())
        && trailing.bytes().all(|byte| byte.is_ascii_whitespace()))
    .then_some((leading, trailing))
}
