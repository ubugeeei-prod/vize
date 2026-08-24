//! Text and interpolation children. Compounds compile from [`TextFacts`],
//! never from the opaque rebuilt source (pessimal law 5).
//!
//! [`TextFacts`]: crate::pass::TextFacts

use vize_davinci::id::NodeId;
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{InterpolationOp, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::js::escape_js_string;
use crate::lower::TextPart;

pub(super) fn emit_text_like(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    for (i, op) in ops.iter().enumerate() {
        let id = cx.walk.mint();
        if i > 0 {
            cx.buf.push(" + ");
        }
        match op {
            Op::Text(text) => emit_quoted_text(cx, text.content),
            Op::Interpolation(interp) => emit_interpolation(cx, interp, id)?,
            Op::Element(_) | Op::Component(_) | Op::If(_) | Op::For(_) | Op::Slot(_) => {
                return Err(EmitError::Unsupported);
            }
        }
    }
    Ok(())
}

pub(super) fn children_need_text_flag(children: &Region<'_>) -> bool {
    let ops = &children.ops;
    if ops.is_empty() {
        return false;
    }
    let mut any_interp = false;
    for op in ops.iter() {
        match op {
            Op::Interpolation(_) => any_interp = true,
            Op::Text(_) => {}
            _ => return false,
        }
    }
    any_interp
}

pub(super) fn emit_interpolation(
    cx: &mut EmitCx<'_>,
    interp: &InterpolationOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    match interp.expression {
        ExprRef::Js(js) => {
            emit_to_display_string(cx, js.source);
            Ok(())
        }
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            let id = id.ok_or(EmitError::Unsupported)?;
            let facts = cx.facts.text_facts.get(id).ok_or(EmitError::Unsupported)?;
            emit_compound_parts(cx, &facts.parts)
        }
        ExprRef::Foreign(_) | ExprRef::Filter(_) | ExprRef::Opaque(_) => {
            Err(EmitError::Unsupported)
        }
    }
}

fn emit_compound_parts(cx: &mut EmitCx<'_>, parts: &[TextPart]) -> Result<(), EmitError> {
    if parts.is_empty() {
        return Err(EmitError::Unsupported);
    }
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            cx.buf.push(" + ");
        }
        if part.dynamic {
            emit_to_display_string(cx, part.text.as_str());
        } else {
            emit_quoted_text(cx, part.text.as_str());
        }
    }
    Ok(())
}

fn emit_quoted_text(cx: &mut EmitCx<'_>, content: &str) {
    cx.buf.push("\"");
    cx.buf.push(escape_js_string(content).as_str());
    cx.buf.push("\"");
}

pub(super) fn emit_to_display_string(cx: &mut EmitCx<'_>, source: &str) {
    cx.buf.use_to_display_string();
    cx.buf.push(Buf::to_display_string_alias());
    cx.buf.push("(");
    cx.buf.push(source);
    cx.buf.push(")");
}

/// Static `_createTextVNode("…")` with no walk mint (compound fallback
/// parts already minted their interpolation op).
pub(super) fn emit_plain_text_vnode(cx: &mut EmitCx<'_>, content: &str) {
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    if content == " " {
        cx.buf.push("()");
        return;
    }
    cx.buf.push("(\"");
    cx.buf.push(escape_js_string(content).as_str());
    cx.buf.push("\")");
}

/// Array-form text run: `_createTextVNode(...)`, matching
/// `codegen/children.rs`'s consecutive-run grouping.
pub(super) fn emit_create_text_vnode(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    if ops.is_empty() {
        return Err(EmitError::Unsupported);
    }
    let has_interp = ops.iter().any(|op| matches!(op, Op::Interpolation(_)));
    let is_single_space =
        !has_interp && ops.len() == 1 && matches!(&ops[0], Op::Text(text) if text.content == " ");
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    if is_single_space {
        let _id = cx.walk.mint();
        cx.buf.push("()");
        return Ok(());
    }
    cx.buf.push("(");
    emit_text_like(cx, ops)?;
    if has_interp {
        cx.buf.push(", 1 /* TEXT */");
    }
    cx.buf.push(")");
    Ok(())
}

/// Slot default children: each S2 text/interp op is one or more
/// `_createTextVNode`s. A compound interpolation expands to one vnode
/// per [`TextPart`] so the object matches Vue's separate text + interp
/// children, not the element-children ` + ` concat.
pub(super) fn emit_slot_text_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    let Op::Interpolation(interp) = op else {
        return emit_create_text_vnode(cx, core::slice::from_ref(op));
    };
    match interp.expression {
        ExprRef::Js(_) => emit_create_text_vnode(cx, core::slice::from_ref(op)),
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            let id = cx.walk.mint().ok_or(EmitError::Unsupported)?;
            let parts = cx
                .facts
                .text_facts
                .get(id)
                .ok_or(EmitError::Unsupported)?
                .parts
                .clone();
            emit_slot_compound_parts(cx, &parts)
        }
        ExprRef::Foreign(_) | ExprRef::Filter(_) | ExprRef::Opaque(_) => {
            Err(EmitError::Unsupported)
        }
    }
}

fn emit_slot_compound_parts(cx: &mut EmitCx<'_>, parts: &[TextPart]) -> Result<(), EmitError> {
    if parts.is_empty() {
        return Err(EmitError::Unsupported);
    }
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
            cx.buf.newline();
        }
        cx.buf.use_create_text();
        cx.buf.push(Buf::create_text_alias());
        cx.buf.push("(");
        if part.dynamic {
            emit_to_display_string(cx, part.text.as_str());
            cx.buf.push(", 1 /* TEXT */");
        } else {
            emit_quoted_text(cx, part.text.as_str());
        }
        cx.buf.push(")");
    }
    Ok(())
}
