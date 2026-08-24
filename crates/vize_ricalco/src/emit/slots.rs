//! Implicit default-slot objects (`withCtx` / `_: 1|2`).
//!
//! This installment emits **text / interpolation** default slots only.
//! Named / scoped slots, `<template>`, slot outlets, `v-slots`,
//! `createSlots`, and native / component children (they need the
//! static-vnode hoist) stay [`EmitError::Unsupported`].

use vize_disegno::op::{Op, Region, TextOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::children::emit_slot_text_child;

pub(super) fn has_implicit_default(children: &Region<'_>) -> bool {
    children.ops.iter().any(|op| !is_whitespace_text(op))
}

pub(super) fn admit_text_default(children: &Region<'_>) -> Result<(), EmitError> {
    if children
        .ops
        .iter()
        .all(|op| matches!(op, Op::Text(_) | Op::Interpolation(_)))
    {
        Ok(())
    } else {
        Err(EmitError::Unsupported)
    }
}

pub(super) fn emit_default_slots(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
) -> Result<(), EmitError> {
    cx.buf.use_with_ctx();
    cx.buf.push("{");
    cx.buf.indent();
    cx.buf.newline();
    cx.buf.push("default: ");
    cx.buf.push(Buf::with_ctx_alias());
    cx.buf.push("(() => [");
    cx.buf.indent();
    let mut first = true;
    for op in children.ops.iter() {
        if !first {
            cx.buf.push(",");
        }
        cx.buf.newline();
        first = false;
        emit_slot_text_child(cx, op)?;
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]),");
    cx.buf.newline();
    if cx.in_v_for {
        cx.buf.push("_: 2 /* DYNAMIC */");
    } else {
        cx.buf.push("_: 1 /* STABLE */");
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
    Ok(())
}

fn is_whitespace_text(op: &Op<'_>) -> bool {
    matches!(op, Op::Text(text) if is_whitespace(text))
}

fn is_whitespace(text: &TextOp<'_>) -> bool {
    text.content.chars().all(char::is_whitespace)
}
