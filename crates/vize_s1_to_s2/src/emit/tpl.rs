//! Unwrapped `<template v-if>` / `<template v-for>` fragments.
//!
//! The lowering unwraps the wrapper, so emit reads
//! [`WrapperKeys::from_template`] / [`ForWrapper`] to recover the
//! shipped codegen split: a single element unwraps to a block (including
//! static nodes, because a `v-for` item must keep block tracking); text /
//! multi child stays a `STABLE_FRAGMENT`.

mod fragment;

use vize_davinci::id::NodeId;
use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp, IfBranch, Op};

pub(super) use fragment::emit_inline;
use fragment::{ChildMode, emit_inner_fragment};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::hoist::is_hoistable;
use super::js::escape_js_string;
use super::prefix::Site;
use super::props_static::PropHoistPosition;
use crate::lower::{WrapperAttr, WrapperClass, WrapperKey};

pub(super) fn wrapper_key_js(cx: &EmitCx<'_>, key: &WrapperKey) -> Result<String, EmitError> {
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
        WrapperKey::Dynamic { source, .. } if cx.prefixing() => {
            cx.prefixed_text(source.as_str(), Site::Expression)
        }
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
            let previous = cx.template_if_for_branch_root;
            cx.template_if_for_branch_root = true;
            let result = super::emit_for_op(cx, for_op, id, Some(key));
            cx.template_if_for_branch_root = previous;
            result
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
