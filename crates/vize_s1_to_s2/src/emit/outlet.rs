//! `<slot>` outlets: `_renderSlot(_ctx.$slots, name, props?, fallback?)`.
//!
//! Matches `vize_atelier_core::codegen/slots/outlet.rs` plus the if/for
//! branch shapes. Static prop keys are camelized.

use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{BindingOp, DynamicName, InterpolationOp, Op, Region, SlotOp};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::children::{
    emit_create_text_vnode, emit_dynamic_part, emit_interpolation, emit_plain_text_vnode,
    emit_raw_interpolation_or_refuse, emit_to_display_string, is_empty_interpolation,
};
use super::hoist::{emit_hoisted_element, is_hoistable};
use super::js::escape_js_string;
use super::outlet_props::emit_props;
use super::prefix::Site;
use super::slots::is_whitespace_text;
use super::vnode::emit_array_child;

pub(super) fn has_forwarded_outlet(region: &Region<'_>) -> bool {
    region.ops.iter().any(op_forwards)
}

fn op_forwards(op: &Op<'_>) -> bool {
    match op {
        Op::Slot(_) => true,
        Op::Element(element) => has_forwarded_outlet(&element.children),
        Op::Component(component) => has_forwarded_outlet(&component.children),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| has_forwarded_outlet(&branch.region)),
        Op::For(for_op) => has_forwarded_outlet(&for_op.region),
        Op::Text(_) | Op::Interpolation(_) => false,
    }
}

/// Emit `write` inside a slot function: `params` is the authored slot
/// props text when the slot is scoped (`None` for a bare slot). The
/// depth tracks `has_slot_params`; the prefix scope tracks the transform's
/// `enter_v_slot_scope` names and the codegen's `add_slot_params`.
pub(super) fn with_slot_params<T>(
    cx: &mut EmitCx<'_>,
    params: Option<&str>,
    write: impl FnOnce(&mut EmitCx<'_>) -> Result<T, EmitError>,
) -> Result<T, EmitError> {
    let scoped = params.is_some();
    if scoped {
        cx.slot_param_depth = cx.slot_param_depth.saturating_add(1);
    }
    let mark = cx.enter_slot_scope(params);
    let result = write(cx);
    cx.leave_scope(mark);
    if scoped {
        cx.slot_param_depth = cx.slot_param_depth.saturating_sub(1);
    }
    result
}

pub(super) fn emit_outlet(
    cx: &mut EmitCx<'_>,
    slot: &SlotOp<'_>,
    key: Option<&str>,
    compact_fallback: bool,
) -> Result<(), EmitError> {
    cx.buf.use_render_slot();
    cx.buf.push(Buf::render_slot_alias());
    cx.buf.push("(_ctx.$slots, ");
    emit_name(cx, slot)?;
    let fallback = meaningful_fallback(&slot.fallback);
    let props = has_props(slot) || key.is_some();
    if fallback {
        cx.buf.push(", ");
        if props {
            emit_props(cx, slot, key)?;
        } else {
            cx.buf.push("{}");
        }
        cx.buf.push(", () => [");
        emit_fallback(cx, &slot.fallback, compact_fallback)?;
        cx.buf.push("])");
    } else if props {
        skip_region(cx, &slot.fallback);
        cx.buf.push(", ");
        emit_props(cx, slot, key)?;
        cx.buf.push(")");
    } else {
        skip_region(cx, &slot.fallback);
        cx.buf.push(")");
    }
    Ok(())
}

fn emit_name(cx: &mut EmitCx<'_>, slot: &SlotOp<'_>) -> Result<(), EmitError> {
    match &slot.name {
        DynamicName::Static(name) => {
            cx.buf.push("\"");
            cx.buf.push(escape_js_string(name).as_str());
            cx.buf.push("\"");
            Ok(())
        }
        DynamicName::Dynamic(expr) => {
            if super::js::expr_source(expr, false).is_none() {
                return Err(EmitError::unsupported_at(
                    Reason::SlotOutletNameNotJs,
                    expr.span(),
                ));
            }
            if cx.prefixing() {
                return cx.push_prefixed_expr(expr, Site::Expression);
            }
            if let Some(source) = super::js::expr_source(expr, false) {
                cx.buf.push(source.as_str());
            }
            Ok(())
        }
    }
}

fn has_props(slot: &SlotOp<'_>) -> bool {
    !slot.attributes.is_empty()
        || slot
            .bindings
            .iter()
            .any(|binding| !matches!(binding, BindingOp::SlotContent(_) | BindingOp::VueCloak(_)))
}

fn meaningful_fallback(region: &Region<'_>) -> bool {
    region.ops.iter().any(|op| !is_whitespace_text(op))
}

pub(super) fn emit_fallback(
    cx: &mut EmitCx<'_>,
    region: &Region<'_>,
    compact: bool,
) -> Result<(), EmitError> {
    if !compact {
        cx.buf.indent();
    }
    let mut first = true;
    for op in region.ops.iter() {
        if is_whitespace_text(op) {
            let _id = cx.walk.mint();
            continue;
        }
        emit_fallback_units(cx, op, compact, &mut first)?;
    }
    if !compact {
        cx.buf.deindent();
        cx.buf.newline();
    }
    Ok(())
}

fn start_fallback_item(cx: &mut EmitCx<'_>, compact: bool, first: &mut bool) {
    if !*first {
        cx.buf.push(",");
    }
    *first = false;
    if !compact {
        cx.buf.newline();
    }
}

fn emit_fallback_units(
    cx: &mut EmitCx<'_>,
    op: &Op<'_>,
    compact: bool,
    first: &mut bool,
) -> Result<(), EmitError> {
    match op {
        Op::Text(_) => {
            start_fallback_item(cx, compact, first);
            emit_create_text_vnode(cx, core::slice::from_ref(op))
        }
        Op::Interpolation(interp) => emit_fallback_interp(cx, interp, compact, first),
        Op::Element(element) if is_hoistable(element) => {
            start_fallback_item(cx, compact, first);
            emit_hoisted_element(cx, element)
        }
        Op::Element(_) => {
            start_fallback_item(cx, compact, first);
            emit_fallback_element(cx, op)
        }
        Op::Component(_) | Op::If(_) | Op::For(_) | Op::Slot(_) => {
            start_fallback_item(cx, compact, first);
            emit_array_child(cx, op, false, false)
        }
    }
}

fn emit_fallback_element(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    let previous = cx.static_cache;
    cx.static_cache = true;
    let result = emit_array_child(cx, op, false, false);
    cx.static_cache = previous;
    result
}

/// Slot fallback walks each S1 child (`generate_node`), so a merged
/// compound expands back into separate array entries: static parts as
/// `_createTextVNode`, interpolations as bare `_toDisplayString`.
fn emit_fallback_interp(
    cx: &mut EmitCx<'_>,
    interp: &InterpolationOp<'_>,
    compact: bool,
    first: &mut bool,
) -> Result<(), EmitError> {
    let id = cx.walk.mint();
    match interp.expression {
        ExprRef::Js(_) => {
            start_fallback_item(cx, compact, first);
            emit_interpolation(cx, interp, id)
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
                start_fallback_item(cx, compact, first);
                if part.dynamic {
                    emit_dynamic_part(cx, part.text.as_str(), Site::Expression)?;
                } else {
                    emit_plain_text_vnode(cx, part.text.as_str());
                }
            }
            Ok(())
        }
        ExprRef::Opaque(opaque) if is_empty_interpolation(opaque) => {
            start_fallback_item(cx, compact, first);
            emit_to_display_string(cx, "");
            Ok(())
        }
        _ => {
            start_fallback_item(cx, compact, first);
            emit_raw_interpolation_or_refuse(cx, interp.expression)
        }
    }
}

fn skip_region(cx: &mut EmitCx<'_>, region: &Region<'_>) {
    for op in region.ops.iter() {
        skip_op(cx, op);
    }
}

fn skip_op(cx: &mut EmitCx<'_>, op: &Op<'_>) {
    let _id = cx.walk.mint();
    match op {
        Op::Element(element) => {
            cx.walk.skip(element.bindings.len());
            skip_region(cx, &element.children);
        }
        Op::Component(component) => {
            cx.walk.skip(component.bindings.len());
            skip_region(cx, &component.children);
        }
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                skip_region(cx, &branch.region);
            }
        }
        Op::For(for_op) => skip_region(cx, &for_op.region),
        Op::Slot(slot) => {
            cx.walk.skip(slot.bindings.len());
            skip_region(cx, &slot.fallback);
        }
        Op::Text(_) | Op::Interpolation(_) => {}
    }
}
