//! Template root fragments: empty → `null`, a unique non-compound
//! root stays a block / leaf, two or more generate_node children
//! (or one compound interpolation, which S2 merged and codegen
//! walks as separate S1 children) wrap in
//! `_createElementBlock(_Fragment, …, 64 /* STABLE_FRAGMENT */)`.

use vize_disegno::expr::{ExprRef, OpaqueReason};
use vize_disegno::op::{InterpolationOp, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::children::{
    emit_create_text_vnode, emit_interpolation, emit_plain_text_vnode, emit_to_display_string,
};
use super::helper::Helper;
use super::slots::is_whitespace_text;
use super::vnode;

pub(super) fn root_needs_fragment(root: &Region<'_>) -> bool {
    let mut count = 0u32;
    let mut compound = false;
    for op in root.ops.iter() {
        if is_whitespace_text(op) {
            continue;
        }
        count = count.saturating_add(1);
        compound |= is_compound(op);
    }
    count > 1 || (count == 1 && compound)
}

pub(super) fn prefer_root_fragment(buf: &mut Buf, root: &Region<'_>) {
    if !root_needs_fragment(root) {
        return;
    }
    buf.prefer(Helper::OpenBlock);
    buf.prefer(Helper::CreateElementBlock);
    buf.prefer(Helper::Fragment);
}

pub(super) fn emit_root(cx: &mut EmitCx<'_>, root: &Region<'_>) -> Result<(), EmitError> {
    if root_needs_fragment(root) {
        return emit_fragment(cx, root);
    }
    let mut found = false;
    for op in root.ops.iter() {
        if is_whitespace_text(op) {
            let _id = cx.walk.mint();
            continue;
        }
        if found {
            return Err(EmitError::Unsupported);
        }
        found = true;
        emit_unique(cx, op)?;
    }
    if found {
        Ok(())
    } else {
        cx.buf.push("null");
        Ok(())
    }
}

fn emit_unique(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    match op {
        Op::Text(_) => emit_create_text_vnode(cx, core::slice::from_ref(op)),
        Op::Interpolation(interp) => {
            let id = cx.walk.mint();
            emit_interpolation(cx, interp, id)
        }
        Op::Element(element) => {
            let _id = cx.walk.mint();
            cx.walk.skip(element.bindings.len());
            vnode::emit_unique_element(cx, element)
        }
        Op::Component(component) => {
            let id = cx.walk.mint();
            cx.walk.skip(component.bindings.len());
            super::component::emit_root(cx, component, id)
        }
        Op::If(if_op) => {
            let id = cx.walk.mint();
            super::emit_if_op(cx, if_op, id)
        }
        Op::For(for_op) => {
            let _id = cx.walk.mint();
            super::emit_for_op(cx, for_op, _id, None)
        }
        Op::Slot(slot) => {
            let _id = cx.walk.mint();
            cx.walk.skip(slot.bindings.len());
            super::outlet::emit_outlet(cx, slot, None, false)
        }
    }
}

fn emit_fragment(cx: &mut EmitCx<'_>, root: &Region<'_>) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_element_block();
    cx.buf.use_fragment();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    cx.buf.push(Buf::create_element_block_alias());
    cx.buf.push("(");
    cx.buf.push(Buf::fragment_alias());
    cx.buf.push(", null, [");
    cx.buf.indent();
    let mut first = true;
    for op in root.ops.iter() {
        if is_whitespace_text(op) {
            let _id = cx.walk.mint();
            continue;
        }
        emit_units(cx, op, &mut first)?;
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("], 64 /* STABLE_FRAGMENT */))");
    Ok(())
}

fn start_item(cx: &mut EmitCx<'_>, first: &mut bool) {
    if !*first {
        cx.buf.push(",");
    }
    *first = false;
    cx.buf.newline();
}

fn emit_units(cx: &mut EmitCx<'_>, op: &Op<'_>, first: &mut bool) -> Result<(), EmitError> {
    match op {
        Op::Text(_) => {
            start_item(cx, first);
            emit_create_text_vnode(cx, core::slice::from_ref(op))
        }
        Op::Interpolation(interp) => emit_interp(cx, interp, first),
        Op::Element(element) => {
            start_item(cx, first);
            let _id = cx.walk.mint();
            cx.walk.skip(element.bindings.len());
            vnode::emit_fragment_element(cx, element)
        }
        Op::Component(component) => {
            start_item(cx, first);
            let id = cx.walk.mint();
            cx.walk.skip(component.bindings.len());
            super::component::emit_nested(cx, component, id)
        }
        Op::If(if_op) => {
            start_item(cx, first);
            let id = cx.walk.mint();
            super::emit_if_op(cx, if_op, id)
        }
        Op::For(for_op) => {
            start_item(cx, first);
            let _id = cx.walk.mint();
            super::emit_for_op(cx, for_op, _id, None)
        }
        Op::Slot(slot) => {
            start_item(cx, first);
            let _id = cx.walk.mint();
            cx.walk.skip(slot.bindings.len());
            super::outlet::emit_outlet(cx, slot, None, false)
        }
    }
}

fn emit_interp(
    cx: &mut EmitCx<'_>,
    interp: &InterpolationOp<'_>,
    first: &mut bool,
) -> Result<(), EmitError> {
    let id = cx.walk.mint();
    match interp.expression {
        ExprRef::Js(_) => {
            start_item(cx, first);
            emit_interpolation(cx, interp, id)
        }
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            let id = id.ok_or(EmitError::Unsupported)?;
            let parts = cx
                .facts
                .text_facts
                .get(id)
                .ok_or(EmitError::Unsupported)?
                .parts
                .clone();
            for part in parts.iter() {
                start_item(cx, first);
                if part.dynamic {
                    emit_to_display_string(cx, part.text.as_str());
                } else {
                    emit_plain_text_vnode(cx, part.text.as_str());
                }
            }
            Ok(())
        }
        ExprRef::Foreign(_) | ExprRef::Filter(_) | ExprRef::Opaque(_) => {
            Err(EmitError::Unsupported)
        }
    }
}

fn is_compound(op: &Op<'_>) -> bool {
    matches!(
        op,
        Op::Interpolation(interp)
            if matches!(
                interp.expression,
                ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound
            )
    )
}
