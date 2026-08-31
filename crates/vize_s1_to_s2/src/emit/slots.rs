//! Slot objects (`withCtx` / `_: 1|2`) from [`SlotFacts`].
//!
//! Implicit default, named `<template>` groups, and component-root `v-slot`.
//! Conditional / looped slot templates go through [`super::create_slots`].
//! Outlets live in [`super::outlet`]. A `v-slots` spread is the children
//! argument when it is the only slot source, or `...expr` closes a slot object.

use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::expr::ExprRef;
use vize_s2::op::{BindingOp, DynamicName, ElementOp, Op, Region, TextOp};
use vize_s2::scope::ScopeOrigin;

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::children::{emit_slot_text_child, emit_slot_text_run};
use super::hoist::{emit_hoisted_element, is_static_element_tree};
use super::js::{RawJs, escape_js_string, expr_source, is_valid_js_identifier};
use super::vnode::emit_array_child;
use crate::pass::{SlotCarrier, SlotFacts, SlotName, SlotParams};

/// First `v-slots="expr"` (no argument) on the component, matching
/// `codegen/slots/detect.rs`. An argument spelling is a different
/// construct and stays unsupported.
pub(super) fn slots_spread<'a>(
    bindings: &'a [BindingOp<'a>],
) -> Result<Option<RawJs<'a>>, EmitError> {
    for binding in bindings.iter() {
        let BindingOp::VueDirective(directive) = binding else {
            continue;
        };
        if directive.name != "slots" {
            continue;
        }
        if directive.argument.is_some() || !directive.modifiers.is_empty() {
            return Err(EmitError::unsupported_at(
                Reason::SlotsSpreadShape,
                directive.span,
            ));
        }
        return match directive.value {
            Some(expr) => expr_source(&expr, false).map(Some).ok_or_else(|| {
                EmitError::unsupported_at(Reason::SlotsSpreadValueNotJs, expr.span())
            }),
            None => Err(EmitError::unsupported_at(
                Reason::SlotsSpreadValueNotJs,
                directive.span,
            )),
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

pub(super) fn has_text_only_implicit_default(children: &Region<'_>) -> bool {
    let mut has_content = false;
    for op in children.ops.iter() {
        if is_whitespace_text(op) {
            continue;
        }
        has_content = true;
        if !matches!(op, Op::Text(_) | Op::Interpolation(_)) {
            return false;
        }
    }
    has_content
}

pub(super) fn has_dynamic_names(facts: &SlotFacts) -> bool {
    facts
        .groups
        .iter()
        .any(|g| matches!(g.name, SlotName::Dynamic { .. }))
}

pub(super) fn admit_default(children: &Region<'_>) -> Result<(), EmitError> {
    walk_admit(children)
}

fn walk_admit(region: &Region<'_>) -> Result<(), EmitError> {
    for op in region.ops.iter() {
        match op {
            Op::Text(_) | Op::Interpolation(_) => {}
            Op::Element(element) if is_slot_template(element) => {
                if element.bindings.iter().any(|b| !is_inert_slot_binding(b)) {
                    return Err(EmitError::unsupported_at(
                        Reason::SlotDefaultShape,
                        element.span,
                    ));
                }
                walk_admit(&element.children)?;
            }
            Op::Element(element) => walk_admit(&element.children)?,
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

fn is_inert_slot_binding(binding: &BindingOp<'_>) -> bool {
    match binding {
        BindingOp::SlotContent(_)
        | BindingOp::VueOnce(_)
        | BindingOp::VueMemo(_)
        | BindingOp::VueCloak(_) => true,
        BindingOp::Bind(bind)
            if matches!(bind.name, Some(DynamicName::Static("key")))
                && bind.modifiers.is_empty()
                && matches!(bind.value, Some(ExprRef::Js(_))) =>
        {
            true
        }
        _ => false,
    }
}

pub(super) fn emit_slots(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    facts: &SlotFacts,
    spread: Option<&RawJs<'_>>,
) -> Result<(), EmitError> {
    if let Some(group) = facts.groups.iter().find(|group| group.name.text() == "_") {
        if let SlotName::Static {
            origin: ScopeOrigin::Authored { span },
            ..
        } = &group.name
        {
            return Err(EmitError::unsupported_at(Reason::SlotNameUnderscore, *span));
        }
        return Err(EmitError::unsupported(Reason::SlotNameUnderscore));
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
        cx.buf.push(spread.as_str());
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
    for op in children.ops.iter() {
        match op {
            Op::Element(element) if is_slot_template(element) => {
                let id = cx.walk.mint();
                cx.walk.skip(element.bindings.len());
                let Some(idx) = facts.groups.iter().position(
                    |group| matches!(group.carrier, SlotCarrier::Template(tid) if tid == id),
                ) else {
                    return Err(EmitError::unsupported_at(
                        Reason::SlotFactsMissingGroup,
                        element.span,
                    ));
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
                    return Err(EmitError::unsupported_op(Reason::SlotFactsMissingGroup, op));
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
            emit_slot_text_run(cx, children.ops.as_slice())
        })?);
        return Ok(());
    }
    for op in children.ops.iter() {
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
    if super::slot_root::emit_transition_child(cx, op)? {
        return Ok(());
    }
    match op {
        Op::Text(_) | Op::Interpolation(_) => emit_slot_text_child(cx, op),
        Op::Element(element) if is_static_element_tree(element) => {
            emit_hoisted_element(cx, element)
        }
        Op::Element(_) | Op::Component(_) | Op::If(_) | Op::For(_) | Op::Slot(_) => {
            emit_array_child(cx, op, false, false)
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
