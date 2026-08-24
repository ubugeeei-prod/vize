//! Slot objects (`withCtx` / `_: 1|2`) from [`SlotFacts`].
//!
//! Implicit default, static / dynamic named `<template>` groups, and
//! component-root `v-slot` (bare spellings key `default`; named spellings
//! preserve their authored slot name).
//! Conditional / looped slot templates go through [`super::create_slots`].
//! Outlets (`renderSlot`) live in [`super::outlet`]. A `v-slots` spread
//! is the children argument when it is the only slot source, or
//! `...expr` closing an authored slot object (no `_` flag).

use alloc::vec::Vec as StdVec;

use vize_carton::String;
use vize_disegno::expr::ExprRef;
use vize_disegno::op::{BindingOp, ElementOp, Namespace, Op, Region, TextOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::children::{emit_create_text_vnode, emit_slot_text_child};
use super::hoist::{emit_hoisted_element, is_hoistable};
use super::js::{escape_js_string, is_valid_js_identifier};
use super::vnode::emit_array_child;
use crate::pass::{SlotCarrier, SlotFacts, SlotName, SlotParams};

/// First `v-slots="expr"` (no argument) on the component, matching
/// `codegen/slots/detect.rs`. An argument spelling is a different
/// construct and stays unsupported.
pub(super) fn slots_spread<'a>(
    bindings: &'a [BindingOp<'a>],
) -> Result<Option<&'a str>, EmitError> {
    for binding in bindings.iter() {
        let BindingOp::VueDirective(directive) = binding else {
            continue;
        };
        if directive.name != "slots" {
            continue;
        }
        if directive.argument.is_some() || !directive.modifiers.is_empty() {
            return Err(EmitError::Unsupported);
        }
        return match directive.value {
            Some(ExprRef::Js(js)) => Ok(Some(js.source)),
            _ => Err(EmitError::Unsupported),
        };
    }
    Ok(None)
}

pub(super) fn is_slots_spread(binding: &BindingOp<'_>) -> bool {
    matches!(binding, BindingOp::VueDirective(directive) if directive.name == "slots")
}

pub(super) fn has_implicit_default(children: &Region<'_>) -> bool {
    children.ops.iter().any(|op| !is_whitespace_text(op))
}

pub(super) fn has_dynamic_names(facts: &SlotFacts) -> bool {
    facts
        .groups
        .iter()
        .any(|group| matches!(group.name, SlotName::Dynamic { .. }))
}

pub(super) fn admit_default(children: &Region<'_>) -> Result<(), EmitError> {
    walk_admit(children)
}

fn walk_admit(region: &Region<'_>) -> Result<(), EmitError> {
    for op in region.ops.iter() {
        match op {
            Op::Text(_) | Op::Interpolation(_) => {}
            Op::Element(element) if is_slot_template(element) => {
                if element
                    .bindings
                    .iter()
                    .any(|binding| !matches!(binding, BindingOp::SlotContent(_)))
                    || !element.attributes.is_empty()
                {
                    return Err(EmitError::Unsupported);
                }
                walk_admit(&element.children)?;
            }
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
            Op::Slot(slot) => walk_admit(&slot.fallback)?,
        }
    }
    Ok(())
}

pub(super) fn emit_slots(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    facts: &SlotFacts,
    spread: Option<&str>,
) -> Result<(), EmitError> {
    if facts.groups.iter().any(|group| group.name.text() == "_") {
        return Err(EmitError::Unsupported);
    }
    cx.buf.use_with_ctx();
    cx.buf.indent();
    cx.buf.indent();
    let mut buckets: StdVec<StdVec<String>> = facts.groups.iter().map(|_| StdVec::new()).collect();
    collect_pieces(cx, children, facts, &mut buckets)?;
    cx.buf.deindent();
    cx.buf.deindent();
    cx.buf.push("{");
    cx.buf.indent();
    for (i, group) in facts.groups.iter().enumerate() {
        cx.buf.newline();
        emit_slot_key(cx, &group.name)?;
        cx.buf.push(": ");
        cx.buf.push(Buf::with_ctx_alias());
        cx.buf.push("(");
        emit_slot_params(&mut cx.buf, &group.params);
        cx.buf.push(" => [");
        cx.buf.indent();
        for (j, piece) in buckets[i].iter().enumerate() {
            if j > 0 {
                cx.buf.push(",");
            }
            cx.buf.newline();
            cx.buf.push(piece.as_str());
        }
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("]),");
    }
    cx.buf.newline();
    if let Some(spread) = spread {
        cx.buf.push("...");
        cx.buf.push(spread);
    } else {
        let forwarded = super::outlet::has_forwarded_outlet(children);
        if forwarded && cx.slot_param_depth == 0 && !cx.in_v_for && !has_dynamic_names(facts) {
            cx.buf.push("_: 3 /* FORWARDED */");
        } else if cx.in_v_for || has_dynamic_names(facts) || (forwarded && cx.slot_param_depth > 0)
        {
            cx.buf.push("_: 2 /* DYNAMIC */");
        } else {
            cx.buf.push("_: 1 /* STABLE */");
        }
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
    Ok(())
}

fn collect_pieces(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    facts: &SlotFacts,
    buckets: &mut [StdVec<String>],
) -> Result<(), EmitError> {
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
            Op::Element(element) if is_slot_template(element) => {
                let id = cx.walk.mint();
                cx.walk.skip(element.bindings.len());
                let Some(idx) = facts.groups.iter().position(
                    |group| matches!(group.carrier, SlotCarrier::Template(tid) if tid == id),
                ) else {
                    return Err(EmitError::Unsupported);
                };
                let scoped = matches!(facts.groups[idx].params, SlotParams::Scoped { .. });
                super::outlet::with_slot_params(cx, scoped, |cx| {
                    emit_template_pieces(cx, &element.children, &mut buckets[idx])
                })?;
            }
            _ => {
                let idx = facts.groups.iter().position(|group| {
                    matches!(
                        group.carrier,
                        SlotCarrier::Implicit | SlotCarrier::Component
                    )
                });
                let Some(idx) = idx else {
                    return Err(EmitError::Unsupported);
                };
                let scoped = matches!(facts.groups[idx].params, SlotParams::Scoped { .. });
                buckets[idx].push(super::outlet::with_slot_params(cx, scoped, |cx| {
                    capture_child(cx, op)
                })?);
            }
        }
    }
    Ok(())
}

pub(super) fn emit_template_pieces(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    bucket: &mut StdVec<String>,
) -> Result<(), EmitError> {
    if children.ops.iter().all(is_whitespace_text) {
        for op in children.ops.iter() {
            let _id = cx.walk.mint();
            let _ = op;
        }
        return Ok(());
    }
    if !children.ops.is_empty()
        && children
            .ops
            .iter()
            .all(|op| matches!(op, Op::Text(_) | Op::Interpolation(_)))
    {
        bucket.push(capture(cx, |cx| {
            emit_create_text_vnode(cx, children.ops.as_slice())
        })?);
        return Ok(());
    }
    let skip_ws = children
        .ops
        .iter()
        .any(|op| !matches!(op, Op::Text(_) | Op::Interpolation(_)));
    for op in children.ops.iter() {
        if skip_ws && is_whitespace_text(op) {
            let _id = cx.walk.mint();
            continue;
        }
        bucket.push(capture_child(cx, op)?);
    }
    Ok(())
}

fn emit_slot_key(cx: &mut EmitCx<'_>, name: &SlotName) -> Result<(), EmitError> {
    match name {
        SlotName::Static { text, .. } if is_valid_js_identifier(text.as_str()) => {
            cx.buf.push(text.as_str());
        }
        SlotName::Static { text, .. } => {
            cx.buf.push("\"");
            cx.buf.push(escape_js_string(text.as_str()).as_str());
            cx.buf.push("\"");
        }
        SlotName::Dynamic { text } => {
            cx.buf.push("[");
            cx.buf.push(text.as_str());
            cx.buf.push("]");
        }
    }
    Ok(())
}

fn emit_slot_params(buf: &mut Buf, params: &SlotParams) {
    match params {
        SlotParams::Absent => buf.push("()"),
        SlotParams::Scoped { text, .. } => {
            buf.push("(");
            buf.push(text.as_str());
            buf.push(")");
        }
    }
}

pub(super) fn capture_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<String, EmitError> {
    capture(cx, |cx| emit_slot_child(cx, op))
}

pub(super) fn capture(
    cx: &mut EmitCx<'_>,
    write: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<String, EmitError> {
    let start = cx.buf.code.len();
    write(cx)?;
    let piece = String::from(&cx.buf.code.as_str()[start..]);
    cx.buf.code.truncate(start);
    Ok(piece)
}

fn emit_slot_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    match op {
        Op::Text(_) | Op::Interpolation(_) => emit_slot_text_child(cx, op),
        Op::Element(element) if is_hoistable(element) => emit_hoisted_element(cx, element),
        Op::Element(_) | Op::Component(_) | Op::If(_) | Op::For(_) | Op::Slot(_) => {
            emit_array_child(cx, op)
        }
    }
}

pub(super) fn is_slot_template(element: &ElementOp<'_>) -> bool {
    element.tag == "template"
        && element
            .bindings
            .iter()
            .any(|binding| matches!(binding, BindingOp::SlotContent(_)))
}

pub(super) fn is_whitespace_text(op: &Op<'_>) -> bool {
    matches!(op, Op::Text(text) if is_whitespace(text))
}

fn is_whitespace(text: &TextOp<'_>) -> bool {
    text.content.chars().all(char::is_whitespace)
}
