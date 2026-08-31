//! Unwrapped `<template v-if>` / `<template v-for>` fragments.
//!
//! The lowering unwraps the wrapper, so emit reads
//! [`WrapperKeys::from_template`] / [`ForWrapper`] to recover the
//! shipped codegen split: a single element unwraps to a block (including
//! static nodes, because a `v-for` item must keep block tracking); text /
//! multi child stays a `STABLE_FRAGMENT`.

use vize_s0::String;
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{IfBranch, Op};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::children::{
    emit_create_text_vnode, emit_interpolation, emit_js_to_display_string, emit_plain_text_vnode,
    emit_raw_interpolation_or_refuse, emit_to_display_string, is_empty_interpolation,
};
use super::hoist::{emit_hoisted_element, is_hoistable};
use super::js::escape_js_string;
use super::vnode;
use crate::lower::WrapperKey;

#[derive(Clone, Copy)]
enum ChildMode {
    /// `generate_children_force_array`: interpolations are
    /// `_createTextVNode(_toDisplayString(…), 1)`.
    ForceArray,
    /// `generate_node` per child: interpolations stay bare
    /// `_toDisplayString`.
    GenerateNode,
}

pub(super) fn wrapper_key_js(key: &WrapperKey) -> Result<String, EmitError> {
    match key {
        WrapperKey::Static { value: None, .. } => Ok(String::from("\"\"")),
        WrapperKey::Static {
            value: Some(value), ..
        } => {
            let mut out = String::from("\"");
            out.push_str(escape_js_string(value.as_str()).as_str());
            out.push('"');
            Ok(out)
        }
        WrapperKey::Dynamic { source, span } if source.is_empty() => Err(
            EmitError::unsupported_at(Reason::TemplateDynamicKeyEmpty, *span),
        ),
        WrapperKey::Dynamic { source, .. } => Ok(source.clone()),
    }
}

pub(super) fn emit_if_template_branch(
    cx: &mut EmitCx<'_>,
    branch: &IfBranch<'_>,
    key: &str,
) -> Result<(), EmitError> {
    if should_unwrap_if(&branch.region.ops) {
        return unwrap_if(cx, branch, key);
    }
    emit_inner_fragment(cx, &branch.region.ops, Some(key), ChildMode::ForceArray)
}

fn should_unwrap_if(ops: &[Op<'_>]) -> bool {
    match ops {
        [Op::Element(element)] => !is_hoistable(element),
        [Op::Component(_)] | [Op::Slot(_)] | [Op::For(_)] => true,
        _ => false,
    }
}

fn unwrap_if(cx: &mut EmitCx<'_>, branch: &IfBranch<'_>, key: &str) -> Result<(), EmitError> {
    match branch.region.ops.as_slice() {
        [Op::Element(element)] => {
            let _id = cx.walk.mint();
            cx.walk.skip(element.bindings.len());
            super::emit_if_branch_call(cx, element, key)
        }
        [Op::Component(component)] => {
            let id = cx.walk.mint();
            cx.walk.skip(component.bindings.len());
            super::component::emit_if_branch(cx, component, key, id)
        }
        [Op::Slot(slot)] => {
            let _id = cx.walk.mint();
            cx.walk.skip(slot.bindings.len());
            super::outlet::emit_outlet(cx, slot, Some(key), true)
        }
        [Op::For(for_op)] => {
            let id = cx.walk.mint();
            super::emit_for_op(cx, for_op, id, Some(key))
        }
        _ => Err(EmitError::unsupported_at(
            Reason::TemplateUnwrapShape,
            branch.span,
        )),
    }
}

pub(super) fn should_unwrap_for(ops: &[Op<'_>]) -> bool {
    matches!(ops, [Op::Element(element)] if !super::slots::is_slot_template(element))
}

pub(super) fn emit_for_template_item(
    cx: &mut EmitCx<'_>,
    ops: &[Op<'_>],
    stable: bool,
    key: Option<&str>,
) -> Result<(), EmitError> {
    if should_unwrap_for(ops) {
        let Op::Element(element) = &ops[0] else {
            return Err(EmitError::unsupported_op(
                Reason::TemplateUnwrapShape,
                &ops[0],
            ));
        };
        let id = cx.walk.mint();
        cx.walk.skip(element.bindings.len());
        return super::emit_for_item_call(cx, element, id, stable, key);
    }
    emit_inner_fragment(cx, ops, key, ChildMode::GenerateNode)
}

pub(super) fn emit_inline(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    match ops {
        [op] => emit_inline_child(cx, op),
        ops => {
            cx.buf.push("[");
            for (i, op) in ops.iter().enumerate() {
                if i > 0 {
                    cx.buf.push(", ");
                }
                emit_inline_child(cx, op)?;
            }
            cx.buf.push("]");
            Ok(())
        }
    }
}

fn emit_inner_fragment(
    cx: &mut EmitCx<'_>,
    ops: &[Op<'_>],
    key: Option<&str>,
    mode: ChildMode,
) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_element_block();
    cx.buf.use_fragment();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    cx.buf.push(Buf::create_element_block_alias());
    cx.buf.push("(");
    cx.buf.push(Buf::fragment_alias());
    if let Some(key) = key {
        cx.buf.push(", { key: ");
        cx.buf.push(key);
        cx.buf.push(" }, ");
    } else {
        cx.buf.push(", null, ");
    }
    cx.with_static_vnode_hoist(true, |cx| emit_fragment_children(cx, ops, mode))?;
    cx.buf.push(", 64 /* STABLE_FRAGMENT */))");
    Ok(())
}

fn emit_fragment_children(
    cx: &mut EmitCx<'_>,
    ops: &[Op<'_>],
    mode: ChildMode,
) -> Result<(), EmitError> {
    if ops.is_empty() {
        cx.buf.push("null");
        return Ok(());
    }
    cx.buf.push("[");
    cx.buf.indent();
    match mode {
        ChildMode::ForceArray => emit_force_array(cx, ops)?,
        ChildMode::GenerateNode => emit_generate_node(cx, ops)?,
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]");
    Ok(())
}

fn emit_force_array(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    let mut i = 0;
    let mut first = true;
    while i < ops.len() {
        if matches!(ops[i], Op::Text(_) | Op::Interpolation(_)) {
            let start = i;
            while i < ops.len() && matches!(ops[i], Op::Text(_) | Op::Interpolation(_)) {
                i += 1;
            }
            start_item(cx, &mut first);
            emit_create_text_vnode(cx, &ops[start..i])?;
            continue;
        }
        start_item(cx, &mut first);
        emit_node_child(cx, &ops[i])?;
        i += 1;
    }
    Ok(())
}

fn emit_generate_node(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    let mut first = true;
    for op in ops {
        match op {
            Op::Text(_) => {
                start_item(cx, &mut first);
                emit_create_text_vnode(cx, core::slice::from_ref(op))?;
            }
            Op::Interpolation(interp) => {
                emit_gen_interp(cx, interp, &mut first)?;
            }
            _ => {
                start_item(cx, &mut first);
                emit_node_child(cx, op)?;
            }
        }
    }
    Ok(())
}

fn emit_gen_interp(
    cx: &mut EmitCx<'_>,
    interp: &vize_s2::op::InterpolationOp<'_>,
    first: &mut bool,
) -> Result<(), EmitError> {
    let id = cx.walk.mint();
    match interp.expression {
        ExprRef::Js(js) => {
            start_item(cx, first);
            emit_js_to_display_string(cx, js);
            Ok(())
        }
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            let id = id.ok_or(EmitError::unsupported_at(
                Reason::WalkIdOverflow,
                interp.span,
            ))?;
            let parts = cx
                .facts
                .text_facts
                .get(id)
                .ok_or(EmitError::unsupported_at_node(
                    Reason::MissingTextFacts,
                    interp.span,
                    id,
                ))?
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
        ExprRef::Opaque(opaque) if is_empty_interpolation(opaque) => {
            start_item(cx, first);
            emit_to_display_string(cx, "");
            Ok(())
        }
        _ => {
            start_item(cx, first);
            emit_raw_interpolation_or_refuse(cx, interp.expression)
        }
    }
}

fn emit_node_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    match op {
        Op::Element(element) if is_hoistable(element) => emit_hoisted_element(cx, element),
        _ => vnode::emit_array_child(cx, op, false, false),
    }
}

fn emit_inline_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    match op {
        Op::Text(_) => emit_create_text_vnode(cx, core::slice::from_ref(op)),
        Op::Interpolation(interp) => {
            let id = cx.walk.mint();
            emit_interpolation(cx, interp, id)
        }
        _ => emit_node_child(cx, op),
    }
}

fn start_item(cx: &mut EmitCx<'_>, first: &mut bool) {
    if !*first {
        cx.buf.push(",");
    }
    *first = false;
    cx.buf.newline();
}
