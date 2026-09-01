use alloc::vec::Vec as StdVec;

use vize_s0::{Span, String, ToCompactString};
use vize_s2::expr::ExprRef;
use vize_s2::op::{DynamicName, ElementOp, ForOp, IfOp, SlotContentOp};

use crate::emit::buf::Buf;
use crate::emit::create_slots_walk::{first_slot_template, skip_ops};
use crate::emit::js::{escape_js_string, expr_source};
use crate::emit::slots::emit_template_pieces;
use crate::emit::vfor;
use crate::emit::{EmitCx, EmitError};

pub(super) fn emit_if_entry(cx: &mut EmitCx<'_>, if_op: &IfOp<'_>) -> Result<(), EmitError> {
    let _id = cx.walk.mint();
    for (i, branch) in if_op.branches.iter().enumerate() {
        if i > 0 {
            cx.buf.newline();
            cx.buf.push(": ");
        }
        if let Some(condition) = &branch.condition {
            cx.buf.push("(");
            emit_condition(cx, condition, branch.span)?;
            cx.buf.push(")");
            cx.buf.indent();
            cx.buf.newline();
            cx.buf.push("? ");
        }
        match first_slot_template(&branch.region) {
            Some((idx, element, content)) => {
                skip_ops(cx, &branch.region.ops[..idx]);
                emit_slot_object(cx, element, content, Some(i as u32))?;
                skip_ops(cx, &branch.region.ops[idx + 1..]);
            }
            None => {
                skip_ops(cx, &branch.region.ops);
                cx.buf.push("undefined");
            }
        }
        if branch.condition.is_some() {
            cx.buf.deindent();
        }
    }
    if if_op
        .branches
        .last()
        .is_none_or(|branch| branch.condition.is_some())
    {
        cx.buf.newline();
        cx.buf.push(": undefined");
    }
    Ok(())
}

fn emit_condition(
    cx: &mut EmitCx<'_>,
    condition: &ExprRef<'_>,
    branch_span: Span,
) -> Result<(), EmitError> {
    let source = vfor::js_source(condition)?;
    if let Some((leading, trailing)) =
        authored_expr_padding(cx.source, branch_span, source.as_str(), condition.span())
    {
        cx.buf.push(leading);
        cx.buf.push(source.as_str());
        cx.buf.push(trailing);
    } else {
        cx.buf.push(source.as_str());
    }
    Ok(())
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

pub(super) fn emit_for_entry(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    slot_idx: usize,
    slot_element: &ElementOp<'_>,
    slot_content: &SlotContentOp<'_>,
) -> Result<(), EmitError> {
    let source_raw = vfor::js_source(&for_op.binding.source)?;
    let source = source_raw.as_str();
    let value = vfor::value_alias(&for_op.binding.value)?;
    let key = vfor::optional_ident(&for_op.binding.key)?;
    let index = vfor::optional_ident(&for_op.binding.index)?;
    let _id = cx.walk.mint();
    cx.buf.use_render_list();
    cx.buf.push(Buf::render_list_alias());
    cx.buf.push("(");
    cx.buf.push(source);
    cx.buf.push(", (");
    cx.buf.push(value);
    if let Some(alias) = key {
        cx.buf.push(", ");
        cx.buf.push(alias);
    }
    if let Some(alias) = index {
        cx.buf.push(", ");
        cx.buf.push(alias);
    }
    cx.buf.push(") => {");
    cx.buf.indent();
    cx.buf.newline();
    cx.buf.push("return ");
    let prev = cx.in_v_for;
    cx.in_v_for = true;
    skip_ops(cx, &for_op.region.ops[..slot_idx]);
    let body = emit_slot_object(cx, slot_element, slot_content, None);
    skip_ops(cx, &for_op.region.ops[slot_idx + 1..]);
    cx.in_v_for = prev;
    body?;
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("})");
    Ok(())
}

pub(super) fn emit_empty_for_slot_outlet_entry(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
) -> Result<(), EmitError> {
    let source_raw = vfor::js_source(&for_op.binding.source)?;
    let source = source_raw.as_str();
    let value = vfor::value_alias(&for_op.binding.value)?;
    let key = vfor::optional_ident(&for_op.binding.key)?;
    let index = vfor::optional_ident(&for_op.binding.index)?;
    let _id = cx.walk.mint();
    cx.buf.use_render_list();
    cx.buf.push(Buf::render_list_alias());
    cx.buf.push("(");
    cx.buf.push(source);
    cx.buf.push(", (");
    cx.buf.push(value);
    if let Some(alias) = key {
        cx.buf.push(", ");
        cx.buf.push(alias);
    }
    if let Some(alias) = index {
        cx.buf.push(", ");
        cx.buf.push(alias);
    }
    cx.buf.push(") => {");
    cx.buf.indent();
    cx.buf.newline();
    cx.buf.push("return ");
    let prev = cx.in_v_for;
    cx.in_v_for = true;
    skip_ops(cx, &for_op.region.ops);
    cx.in_v_for = prev;
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("})");
    Ok(())
}

pub(super) fn emit_slot_object(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    content: &SlotContentOp<'_>,
    key: Option<u32>,
) -> Result<(), EmitError> {
    let _id = cx.walk.mint();
    cx.walk.skip(element.bindings.len());
    cx.buf.push("{");
    cx.buf.indent();
    cx.buf.newline();
    emit_entry_name(cx, content);
    cx.buf.push(",");
    cx.buf.newline();
    cx.buf.push("fn: ");
    cx.buf.push(Buf::with_ctx_alias());
    cx.buf.push("(");
    emit_params(cx, content);
    cx.buf.push(" => [");
    let mut pieces = StdVec::new();
    cx.buf.indent();
    let scoped = matches!(&content.params, Some(expr) if !expr.source().is_empty());
    crate::emit::outlet::with_slot_params(cx, scoped, |cx| {
        emit_template_pieces(cx, &element.children, &mut pieces)
    })?;
    for (i, piece) in pieces.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        cx.buf.newline();
        cx.buf.push(piece.as_str());
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("])");
    if let Some(key) = key {
        cx.buf.push(",");
        cx.buf.newline();
        cx.buf.push("key: \"");
        cx.buf.push(key.to_compact_string().as_str());
        cx.buf.push("\"");
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
    Ok(())
}

fn emit_entry_name(cx: &mut EmitCx<'_>, content: &SlotContentOp<'_>) {
    cx.buf.push("name: ");
    match &content.name {
        Some(DynamicName::Dynamic(expr)) => {
            if let Some(source) = expr_source(expr, false) {
                cx.buf.push(source.as_str());
            } else {
                cx.buf.push(expr.source());
            }
        }
        Some(DynamicName::Static(base)) => {
            cx.buf.push("\"");
            cx.buf
                .push(escape_js_string(&fold_name(base, &content.modifiers)).as_str());
            cx.buf.push("\"");
        }
        None => {
            cx.buf.push("\"");
            cx.buf
                .push(escape_js_string(&fold_name("default", &content.modifiers)).as_str());
            cx.buf.push("\"");
        }
    }
}

fn emit_params(cx: &mut EmitCx<'_>, content: &SlotContentOp<'_>) {
    match &content.params {
        Some(expr) if !expr.source().is_empty() => {
            cx.buf.push("(");
            if let Some((leading, trailing)) =
                authored_expr_padding(cx.source, content.span, expr.source(), expr.span())
            {
                cx.buf.push(leading);
                cx.buf.push(expr.source());
                cx.buf.push(trailing);
            } else {
                cx.buf.push(expr.source());
            }
            cx.buf.push(")");
        }
        _ => cx.buf.push("()"),
    }
}

fn fold_name(base: &str, modifiers: &[&str]) -> String {
    let mut text = String::from(base);
    for modifier in modifiers {
        text.push('.');
        text.push_str(modifier);
    }
    text
}
