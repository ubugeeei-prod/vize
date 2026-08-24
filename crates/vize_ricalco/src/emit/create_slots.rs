//! `createSlots` for `v-if` / `v-for` slot templates (`{ _: 2 }` base
//! plus `{ name, fn }` entries — ternaries, `_renderList`, static named).
//! A `v-slots` spread lands in the base object (`...expr`) before `_: 2`.

use alloc::vec::Vec as StdVec;

use vize_carton::{String, ToCompactString};
use vize_disegno::op::{BindingOp, DynamicName, ElementOp, ForOp, IfOp, Op, Region, SlotContentOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::js::escape_js_string;
use super::slots::{
    capture, capture_child, emit_template_pieces, is_slot_template, is_whitespace_text,
};
use super::vfor;

pub(super) fn needs_create_slots(children: &Region<'_>) -> bool {
    children.ops.iter().any(|op| match op {
        Op::If(if_op) => is_slot_if(if_op),
        Op::For(for_op) => is_slot_for(for_op),
        _ => false,
    })
}

pub(super) fn emit_create_slots(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    spread: Option<&str>,
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
            Op::If(if_op) if is_slot_if(if_op) => {
                entries.push(capture(cx, |cx| emit_if_entry(cx, if_op))?);
            }
            Op::For(for_op) if is_slot_for(for_op) => {
                entries.push(capture(cx, |cx| emit_for_entry(cx, for_op))?);
            }
            Op::Element(element) if is_slot_template(element) => {
                entries.push(capture(cx, |cx| emit_slot_object(cx, element, None))?);
            }
            _ => {
                cx.buf.indent();
                defaults.push(capture_child(cx, op)?);
                cx.buf.deindent();
            }
        }
    }
    Ok((defaults, entries))
}

fn emit_base(cx: &mut EmitCx<'_>, defaults: &[String], spread: Option<&str>) {
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
        cx.buf.push(spread);
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
            cx.buf.push(vfor::js_source(condition)?);
            cx.buf.push(")");
            cx.buf.indent();
            cx.buf.newline();
            cx.buf.push("? ");
        }
        match first_slot_template(&branch.region) {
            Some((idx, element)) => {
                skip_ops(cx, &branch.region.ops[..idx]);
                emit_slot_object(cx, element, Some(i as u32))?;
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

fn emit_for_entry(cx: &mut EmitCx<'_>, for_op: &ForOp<'_>) -> Result<(), EmitError> {
    let source = vfor::js_source(&for_op.binding.source)?;
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
    let body = match first_slot_template(&for_op.region) {
        Some((idx, element)) => {
            skip_ops(cx, &for_op.region.ops[..idx]);
            let result = emit_slot_object(cx, element, None);
            skip_ops(cx, &for_op.region.ops[idx + 1..]);
            result
        }
        None => {
            skip_ops(cx, &for_op.region.ops);
            Err(EmitError::Unsupported)
        }
    };
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
    key: Option<u32>,
) -> Result<(), EmitError> {
    let _id = cx.walk.mint();
    cx.walk.skip(element.bindings.len());
    let content = slot_content(element).ok_or(EmitError::Unsupported)?;
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
        Some(DynamicName::Dynamic(expr)) => cx.buf.push(expr.source()),
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
            cx.buf.push(expr.source());
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

fn slot_content<'a>(element: &'a ElementOp<'a>) -> Option<&'a SlotContentOp<'a>> {
    element.bindings.iter().find_map(|binding| match binding {
        BindingOp::SlotContent(content) => Some(&**content),
        _ => None,
    })
}

fn first_slot_template<'a>(region: &'a Region<'a>) -> Option<(usize, &'a ElementOp<'a>)> {
    region.ops.iter().enumerate().find_map(|(i, op)| match op {
        Op::Element(element) if is_slot_template(element) => Some((i, &**element)),
        _ => None,
    })
}

fn is_slot_if(if_op: &IfOp<'_>) -> bool {
    if_op
        .branches
        .iter()
        .any(|branch| first_slot_template(&branch.region).is_some())
}

fn is_slot_for(for_op: &ForOp<'_>) -> bool {
    first_slot_template(&for_op.region).is_some()
}

fn skip_ops(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) {
    for op in ops {
        skip_op(cx, op);
    }
}

fn skip_op(cx: &mut EmitCx<'_>, op: &Op<'_>) {
    let _id = cx.walk.mint();
    match op {
        Op::Element(element) => {
            cx.walk.skip(element.bindings.len());
            skip_ops(cx, &element.children.ops);
        }
        Op::Component(component) => {
            cx.walk.skip(component.bindings.len());
            skip_ops(cx, &component.children.ops);
        }
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                skip_ops(cx, &branch.region.ops);
            }
        }
        Op::For(for_op) => skip_ops(cx, &for_op.region.ops),
        Op::Slot(slot) => {
            cx.walk.skip(slot.bindings.len());
            skip_ops(cx, &slot.fallback.ops);
        }
        Op::Text(_) | Op::Interpolation(_) => {}
    }
}
