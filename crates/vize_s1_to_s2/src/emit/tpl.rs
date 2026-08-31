//! Unwrapped `<template v-if>` / `<template v-for>` fragments.
//!
//! The lowering unwraps the wrapper, so emit reads
//! [`WrapperKeys::from_template`] / [`ForWrapper`] to recover the
//! shipped codegen split: a single element unwraps to a block (including
//! static nodes, because a `v-for` item must keep block tracking); text /
//! multi child stays a `STABLE_FRAGMENT`.

use vize_davinci::id::NodeId;
use vize_s0::String;
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{Attribute, BindingOp, IfBranch, Op};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::children::{
    emit_create_text_vnode, emit_interpolation, emit_js_to_display_string, emit_plain_text_vnode,
    emit_raw_interpolation_or_refuse, emit_to_display_string, is_empty_interpolation,
};
use super::hoist::{emit_hoisted_element, is_hoistable};
use super::js::{escape_js_string, is_valid_js_identifier};
use super::props_static::PropHoistPosition;
use super::vnode;
use crate::lower::{WrapperAttr, WrapperClass, WrapperKey};

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
    emit_inner_fragment(
        cx,
        &branch.region.ops,
        Some(key),
        &[],
        None,
        ChildMode::ForceArray,
    )
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
            let id = cx.walk.mint();
            let attributes = &element.attributes;
            let bindings = &element.bindings;
            cx.walk.skip(bindings.len());
            register_unwrapped_if_child_props_hoist(cx, attributes, bindings, id)?;
            let previous = cx.template_if_branch_root;
            cx.template_if_branch_root = true;
            let result = super::emit_if_branch_call(cx, element, key);
            cx.template_if_branch_root = previous;
            result
        }
        [Op::Component(component)] => {
            let id = cx.walk.mint();
            cx.walk.skip(component.bindings.len());
            let previous = cx.template_if_branch_root;
            cx.template_if_branch_root = true;
            let result = super::component::emit_if_branch(cx, component, key, id);
            cx.template_if_branch_root = previous;
            result
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

fn register_unwrapped_if_child_props_hoist(
    cx: &mut EmitCx<'_>,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    if !super::props_static::should_hoist(cx, id, PropHoistPosition::Nested) {
        return Ok(());
    }
    if let Some(props) = super::props_static::root_hoist_props(attributes, bindings)? {
        let _ = cx.buf.push_hoist(props);
    }
    Ok(())
}

pub(super) fn should_unwrap_for(ops: &[Op<'_>]) -> bool {
    matches!(ops, [Op::Element(element)] if !super::slots::is_slot_template(element))
}

pub(super) fn emit_for_template_item(
    cx: &mut EmitCx<'_>,
    ops: &[Op<'_>],
    stable: bool,
    key: Option<&str>,
    attributes: &[WrapperAttr],
    class: Option<&WrapperClass>,
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
        let previous = cx.suppress_template_for_child_key;
        cx.suppress_template_for_child_key = true;
        let result = super::emit_for_item_call(cx, element, id, stable, key);
        cx.suppress_template_for_child_key = previous;
        return result;
    }
    if matches!(ops, [Op::Component(_)]) {
        let previous = cx.template_for_item_single_root;
        cx.template_for_item_single_root = true;
        let result = emit_inner_fragment(cx, ops, key, attributes, class, ChildMode::GenerateNode);
        cx.template_for_item_single_root = previous;
        return result;
    }
    emit_inner_fragment(cx, ops, key, attributes, class, ChildMode::GenerateNode)
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
    attributes: &[WrapperAttr],
    class: Option<&WrapperClass>,
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
    emit_fragment_props(cx, key, attributes, class);
    cx.with_static_vnode_hoist(true, |cx| emit_fragment_children(cx, ops, mode))?;
    cx.buf.push(", 64 /* STABLE_FRAGMENT */))");
    Ok(())
}

fn emit_fragment_props(
    cx: &mut EmitCx<'_>,
    key: Option<&str>,
    attributes: &[WrapperAttr],
    class: Option<&WrapperClass>,
) {
    if key.is_none() && attributes.is_empty() && class.is_none() {
        cx.buf.push(", null, ");
        return;
    }
    let multiline = class.is_some_and(|class| class.dynamic_source.is_some());
    if multiline {
        cx.buf.push(", {");
        cx.buf.indent();
    } else {
        cx.buf.push(", { ");
    }
    let mut first = true;
    if let Some(key) = key {
        start_fragment_prop(cx, &mut first, multiline);
        cx.buf.push("key: ");
        cx.buf.push(key);
    }
    for attr in attributes {
        start_fragment_prop(cx, &mut first, multiline);
        push_static_key(cx, attr.name.as_str());
        cx.buf.push(": \"");
        cx.buf
            .push(escape_js_string(attr.value.as_deref().unwrap_or("")).as_str());
        cx.buf.push("\"");
    }
    if let Some(class) = class {
        start_fragment_prop(cx, &mut first, multiline);
        cx.buf.push("class: ");
        emit_wrapper_class(cx, class);
    }
    if multiline {
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}, ");
    } else {
        cx.buf.push(" }, ");
    }
}

fn start_fragment_prop(cx: &mut EmitCx<'_>, first: &mut bool, multiline: bool) {
    if !*first {
        cx.buf.push(",");
        if !multiline {
            cx.buf.push(" ");
        }
    }
    if multiline {
        cx.buf.newline();
    }
    *first = false;
}

fn emit_wrapper_class(cx: &mut EmitCx<'_>, class: &WrapperClass) {
    match (&class.static_value, &class.dynamic_source) {
        (Some(static_value), Some(dynamic_source)) => {
            cx.buf.use_helper(super::helper::Helper::NormalizeClass);
            cx.buf.push(super::helper::Helper::NormalizeClass.alias());
            cx.buf.push("([\"");
            cx.buf
                .push(escape_js_string(static_value.as_str()).as_str());
            cx.buf.push("\", ");
            cx.buf.push(dynamic_source.as_str());
            cx.buf.push("])");
        }
        (None, Some(dynamic_source)) => {
            cx.buf.use_helper(super::helper::Helper::NormalizeClass);
            cx.buf.push(super::helper::Helper::NormalizeClass.alias());
            cx.buf.push("(");
            cx.buf.push(dynamic_source.as_str());
            cx.buf.push(")");
        }
        (Some(static_value), None) => {
            cx.buf.push("\"");
            cx.buf
                .push(escape_js_string(static_value.as_str()).as_str());
            cx.buf.push("\"");
        }
        (None, None) => cx.buf.push("\"\""),
    }
}

fn push_static_key(cx: &mut EmitCx<'_>, key: &str) {
    if is_valid_js_identifier(key) {
        cx.buf.push(key);
        return;
    }
    cx.buf.push("\"");
    cx.buf.push(escape_js_string(key).as_str());
    cx.buf.push("\"");
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
