//! `createSlots` for `v-if` / `v-for` slot templates (`{ _: 2 }` base
//! plus `{ name, fn }` entries — ternaries, `_renderList`, static named).
//! A `v-slots` spread lands in the base object (`...expr`) before `_: 2`.

use alloc::vec::Vec as StdVec;

use vize_s0::{Span, String, ToCompactString};
use vize_s2::expr::ExprRef;
use vize_s2::op::{DynamicName, ElementOp, ForOp, IfOp, Op, Region, SlotContentOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::create_slots_walk::{
    advance_after_op, first_slot_template, is_slot_for, is_slot_if, skip_ops, slot_template_content,
};
use super::js::{RawJs, escape_js_string, expr_source};
use super::slots::{capture, capture_child, emit_template_pieces, is_whitespace_text};
use super::vfor;

pub(super) fn needs_create_slots(cx: &EmitCx<'_>, children: &Region<'_>) -> bool {
    let mut walk = cx.walk.clone();
    children.ops.iter().any(|op| {
        let id = walk.mint();
        let needs = match op {
            Op::If(if_op) => is_slot_if(cx, id, if_op),
            Op::For(for_op) => is_slot_for(cx, id, for_op),
            _ => false,
        };
        advance_after_op(&mut walk, op);
        needs
    })
}

pub(super) fn emit_create_slots(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    spread: Option<&RawJs<'_>>,
) -> Result<(), EmitError> {
    cx.buf.use_create_slots();
    cx.buf.use_with_ctx();
    cx.buf.indent();
    let (defaults, entries) = collect(cx, children)?;
    cx.buf.deindent();
    cx.buf.push(Buf::create_slots_alias());
    cx.buf.push("(");
    emit_base(cx, &defaults, spread);
    cx.buf.push(", [");
    cx.buf.indent();
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        cx.buf.newline();
        cx.buf.push(entry.as_str());
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("])");
    Ok(())
}

fn collect(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
) -> Result<(StdVec<String>, StdVec<String>), EmitError> {
    let mut defaults = StdVec::new();
    let mut entries = StdVec::new();
    let first_branch_key = cx.if_branch_key;
    let mut default_branch_key = first_branch_key;
    let mut entry_branch_key =
        first_branch_key.saturating_add(default_branch_key_count(cx, children));
    let skip_ws = children
        .ops
        .iter()
        .any(|op| !matches!(op, Op::Text(_) | Op::Interpolation(_)));
    for op in children.ops.iter() {
        if skip_ws && is_whitespace_text(op) {
            let _id = cx.walk.mint();
            continue;
        }
        match op {
            Op::If(if_op) => {
                let is_slot = is_slot_if(cx, peek_id(cx), if_op);
                let walk_before = cx.walk.clone();
                let walk_after_default = if !is_slot {
                    with_branch_key(cx, &mut default_branch_key, |cx| {
                        collect_default(cx, &mut defaults, op)
                    })?;
                    Some(cx.walk.clone())
                } else {
                    None
                };
                cx.walk = walk_before;
                entries.push(with_branch_key(cx, &mut entry_branch_key, |cx| {
                    capture(cx, |cx| emit_if_entry(cx, if_op))
                })?);
                if let Some(walk_after) = walk_after_default {
                    cx.walk = walk_after;
                }
            }
            Op::For(for_op) if is_slot_for(cx, peek_id(cx), for_op) => {
                let Some((idx, element, content)) = first_slot_template(&for_op.region) else {
                    return Err(EmitError::unsupported_at(
                        super::UnsupportedReason::CreateSlotsMissingSlotTemplate,
                        for_op.span,
                    ));
                };
                entries.push(with_branch_key(cx, &mut entry_branch_key, |cx| {
                    capture(cx, |cx| emit_for_entry(cx, for_op, idx, element, content))
                })?);
            }
            Op::Element(element) => {
                if let Some(content) = slot_template_content(element) {
                    entries.push(with_branch_key(cx, &mut entry_branch_key, |cx| {
                        capture(cx, |cx| emit_slot_object(cx, element, content, None))
                    })?);
                } else {
                    with_branch_key(cx, &mut default_branch_key, |cx| {
                        collect_default(cx, &mut defaults, op)
                    })?;
                }
            }
            _ => {
                with_branch_key(cx, &mut default_branch_key, |cx| {
                    collect_default(cx, &mut defaults, op)
                })?;
            }
        }
    }
    cx.if_branch_key = entry_branch_key;
    Ok((defaults, entries))
}

fn peek_id(cx: &EmitCx<'_>) -> Option<vize_davinci::id::NodeId> {
    let mut walk = cx.walk.clone();
    walk.mint()
}

fn with_branch_key<T>(
    cx: &mut EmitCx<'_>,
    next_key: &mut u32,
    write: impl FnOnce(&mut EmitCx<'_>) -> Result<T, EmitError>,
) -> Result<T, EmitError> {
    let saved = cx.if_branch_key;
    cx.if_branch_key = *next_key;
    let result = write(cx);
    *next_key = cx.if_branch_key;
    cx.if_branch_key = saved;
    result
}

fn default_branch_key_count(cx: &EmitCx<'_>, children: &Region<'_>) -> u32 {
    let mut walk = cx.walk.clone();
    let skip_ws = children
        .ops
        .iter()
        .any(|op| !matches!(op, Op::Text(_) | Op::Interpolation(_)));
    children.ops.iter().fold(0u32, |count, op| {
        let Some(id) = walk.mint() else {
            return count;
        };
        let is_entry = match op {
            Op::If(if_op) => is_slot_if(cx, Some(id), if_op),
            Op::For(for_op) => is_slot_for(cx, Some(id), for_op),
            Op::Element(element) => slot_template_content(element).is_some(),
            _ => false,
        };
        advance_after_op(&mut walk, op);
        if (skip_ws && is_whitespace_text(op)) || is_entry {
            count
        } else {
            count.saturating_add(op_branch_key_count(op))
        }
    })
}

fn region_branch_key_count(region: &Region<'_>) -> u32 {
    region.ops.iter().fold(0u32, |count, op| {
        count.saturating_add(op_branch_key_count(op))
    })
}

fn op_branch_key_count(op: &Op<'_>) -> u32 {
    match op {
        Op::Element(element) => region_branch_key_count(&element.children),
        Op::Component(component) => region_branch_key_count(&component.children),
        Op::If(if_op) => u32::try_from(if_op.branches.len()).unwrap_or(u32::MAX),
        Op::For(for_op) => region_branch_key_count(&for_op.region),
        Op::Slot(slot) => region_branch_key_count(&slot.fallback),
        Op::Text(_) | Op::Interpolation(_) => 0,
    }
}

fn collect_default(
    cx: &mut EmitCx<'_>,
    defaults: &mut StdVec<String>,
    op: &Op<'_>,
) -> Result<(), EmitError> {
    cx.buf.indent();
    defaults.push(capture_child(cx, op)?);
    cx.buf.deindent();
    Ok(())
}

fn emit_base(cx: &mut EmitCx<'_>, defaults: &[String], spread: Option<&RawJs<'_>>) {
    if defaults.is_empty() && spread.is_none() {
        cx.buf.push("{ _: 2 /* DYNAMIC */ }");
        return;
    }
    cx.buf.push("{");
    cx.buf.indent();
    if !defaults.is_empty() {
        cx.buf.newline();
        cx.buf.push("default: ");
        cx.buf.push(Buf::with_ctx_alias());
        cx.buf.push("(() => [");
        cx.buf.indent();
        for (i, piece) in defaults.iter().enumerate() {
            if i > 0 {
                cx.buf.push(",");
            }
            cx.buf.newline();
            cx.buf.push(piece.as_str());
        }
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("]),");
    }
    if let Some(spread) = spread {
        cx.buf.newline();
        cx.buf.push("...");
        cx.buf.push(spread.as_str());
        cx.buf.push(",");
    }
    cx.buf.newline();
    cx.buf.push("_: 2 /* DYNAMIC */");
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
}

fn emit_if_entry(cx: &mut EmitCx<'_>, if_op: &IfOp<'_>) -> Result<(), EmitError> {
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

fn emit_for_entry(
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

fn emit_slot_object(
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
    super::outlet::with_slot_params(cx, scoped, |cx| {
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
