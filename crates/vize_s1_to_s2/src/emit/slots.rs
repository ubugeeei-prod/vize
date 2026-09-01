//! Slot objects (`withCtx` / `_: 1|2`) from [`SlotFacts`].
//!
//! Implicit default, named `<template>` groups, and component-root `v-slot`.
//! Conditional / looped slot templates go through [`super::create_slots`].
//! Outlets live in [`super::outlet`]. A `v-slots` spread is the children
//! argument when it is the only slot source, or `...expr` closes a slot object.

mod capture;
mod group;
mod params;

use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::expr::ExprRef;
use vize_s2::op::{BindingOp, DynamicName, Op, Region};
use vize_s2::scope::ScopeOrigin;

pub(super) use self::capture::{
    capture, capture_child, emit_template_pieces, is_slot_template, is_whitespace_text,
};
use self::params::{emit_slot_key, emit_slot_params, group_slot_content};
use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::js::{RawJs, expr_source};
use crate::pass::{SlotFacts, SlotName};

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

pub(super) fn filler_default_needs_props_placeholder(children: &Region<'_>) -> bool {
    let mut needs_placeholder = false;
    for op in children.ops.iter() {
        let Op::Text(text) = op else {
            return false;
        };
        if !crate::lower::legacy_slot_filler_text(text.content) {
            return false;
        }
        needs_placeholder |= crate::lower::legacy_slot_filler_needs_props_placeholder(text.content);
    }
    needs_placeholder
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
    component_bindings: &[BindingOp<'_>],
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
    let start_walk = cx.walk.clone();
    group::collect_pieces(cx, children, facts, &mut buckets)?;
    cx.buf.deindent();
    cx.buf.deindent();
    cx.buf.push("{");
    cx.buf.indent();
    for (i, group) in facts.groups.iter().enumerate() {
        cx.buf.newline();
        emit_slot_key(cx, &group.name);
        cx.buf.push(": ");
        cx.buf.push(Buf::with_ctx_alias());
        cx.buf.push("(");
        let content = group_slot_content(&start_walk, children, component_bindings, group.carrier);
        emit_slot_params(cx, &group.params, content);
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
        if forwarded && cx.slot_param_depth == 0 && !cx.in_v_for {
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
