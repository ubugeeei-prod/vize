//! Implicit default-slot objects (`withCtx` / `_: 1|2`).
//!
//! Text / interpolation children, native HTML, nested components, and
//! `ui.if` / `ui.for` inside the default slot. Static element subtrees
//! hoist as `/*#__PURE__*/ _createElementVNode(...)`. Named / scoped
//! slots, `<template>`, slot outlets, `v-slots`, and `createSlots` stay
//! [`EmitError::Unsupported`].

use vize_disegno::op::{Namespace, Op, Region, TextOp};

use super::buf::Buf;
use super::children::emit_slot_text_child;
use super::hoist::{emit_hoisted_element, is_hoistable};
use super::vnode::emit_array_child;
use super::EmitCx;
use super::EmitError;

pub(super) fn has_implicit_default(children: &Region<'_>) -> bool {
    children.ops.iter().any(|op| !is_whitespace_text(op))
}

pub(super) fn admit_default(children: &Region<'_>) -> Result<(), EmitError> {
    walk_admit(children)
}

fn walk_admit(region: &Region<'_>) -> Result<(), EmitError> {
    for op in region.ops.iter() {
        match op {
            Op::Text(_) | Op::Interpolation(_) => {}
            Op::Element(element) => {
                if element.tag == "template" || element.namespace != Namespace::Html {
                    return Err(EmitError::Unsupported);
                }
                walk_admit(&element.children)?;
            }
            Op::Component(component) => walk_admit(&component.children)?,
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    walk_admit(&branch.region)?;
                }
            }
            Op::For(for_op) => walk_admit(&for_op.region)?,
            Op::Slot(_) => return Err(EmitError::Unsupported),
        }
    }
    Ok(())
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
    let skip_ws = children
        .ops
        .iter()
        .any(|op| !matches!(op, Op::Text(_) | Op::Interpolation(_)));
    let mut first = true;
    for op in children.ops.iter() {
        if skip_ws && is_whitespace_text(op) {
            let _id = cx.walk.mint();
            continue;
        }
        if !first {
            cx.buf.push(",");
        }
        cx.buf.newline();
        first = false;
        emit_slot_child(cx, op)?;
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

fn emit_slot_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    match op {
        Op::Text(_) | Op::Interpolation(_) => emit_slot_text_child(cx, op),
        Op::Element(element) if is_hoistable(element) => emit_hoisted_element(cx, element),
        Op::Element(_) | Op::Component(_) | Op::If(_) | Op::For(_) => emit_array_child(cx, op),
        Op::Slot(_) => Err(EmitError::Unsupported),
    }
}

fn is_whitespace_text(op: &Op<'_>) -> bool {
    matches!(op, Op::Text(text) if is_whitespace(text))
}

fn is_whitespace(text: &TextOp<'_>) -> bool {
    text.content.chars().all(char::is_whitespace)
}
