//! `<slot>` outlets: `_renderSlot(_ctx.$slots, name, props?, fallback?)`.
//!
//! Matches `vize_atelier_core::codegen/slots/outlet.rs` plus the if/for
//! branch shapes. Static prop keys are camelized.

use vize_s0::camelize;
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{BindOp, BindingOp, DynamicName, InterpolationOp, OnOp, Op, Region, SlotOp};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::children::{
    emit_create_text_vnode, emit_interpolation, emit_plain_text_vnode, emit_to_display_string,
};
use super::hoist::{emit_hoisted_element, is_hoistable};
use super::js::{escape_js_string, is_valid_js_identifier};
use super::merge;
use super::props::{Piece, emit_dynamic_bind_pair, js_value, pieces};
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

pub(super) fn with_slot_params<T>(
    cx: &mut EmitCx<'_>,
    scoped: bool,
    write: impl FnOnce(&mut EmitCx<'_>) -> Result<T, EmitError>,
) -> Result<T, EmitError> {
    if scoped {
        cx.slot_param_depth = cx.slot_param_depth.saturating_add(1);
    }
    let result = write(cx);
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
        DynamicName::Dynamic(ExprRef::Js(js)) => {
            cx.buf.push(js.source);
            Ok(())
        }
        DynamicName::Dynamic(expr) => Err(EmitError::unsupported_at(
            Reason::SlotOutletNameNotJs,
            expr.span(),
        )),
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

fn emit_props(cx: &mut EmitCx<'_>, slot: &SlotOp<'_>, key: Option<&str>) -> Result<(), EmitError> {
    if merge::has_object_spread(&slot.bindings) {
        return emit_spread_props(cx, slot, key);
    }
    emit_props_object(cx, slot, key)
}

fn emit_spread_props(
    cx: &mut EmitCx<'_>,
    slot: &SlotOp<'_>,
    key: Option<&str>,
) -> Result<(), EmitError> {
    let bind_spread = first_bind_spread(&slot.bindings);
    let on_spread = first_on_spread(&slot.bindings);
    let has_entries = has_entry_props(slot);
    let needs_merge =
        key.is_some() || has_entries || (bind_spread.is_some() && on_spread.is_some());

    if needs_merge {
        cx.buf.use_merge_props();
        cx.buf.push(Buf::merge_props_alias());
        cx.buf.push("(");
        let mut first = true;
        if let Some(bind) = bind_spread {
            push_spread_separator(cx, &mut first);
            emit_bind_spread_expr(cx, bind)?;
        }
        if let Some(on) = on_spread {
            push_spread_separator(cx, &mut first);
            emit_on_spread_expr(cx, on)?;
        }
        if key.is_some() || has_entries {
            push_spread_separator(cx, &mut first);
            emit_props_object(cx, slot, key)?;
        }
        cx.buf.push(")");
    } else if let Some(bind) = bind_spread {
        cx.buf.use_normalize_props();
        cx.buf.use_guard_reactive_props();
        cx.buf.push(Buf::normalize_props_alias());
        cx.buf.push("(");
        cx.buf.push(Buf::guard_reactive_props_alias());
        cx.buf.push("(");
        emit_bind_spread_expr(cx, bind)?;
        cx.buf.push("))");
    } else if let Some(on) = on_spread {
        emit_on_spread_expr(cx, on)?;
    } else {
        emit_props_object(cx, slot, key)?;
    }
    Ok(())
}

fn has_entry_props(slot: &SlotOp<'_>) -> bool {
    !slot.attributes.is_empty()
        || slot.bindings.iter().any(|binding| match binding {
            BindingOp::Bind(bind) if bind.name.is_none() => false,
            BindingOp::On(on) if on.name.is_none() => false,
            BindingOp::SlotContent(_) | BindingOp::VueCloak(_) => false,
            _ => true,
        })
}

fn first_bind_spread<'a>(bindings: &'a [BindingOp<'a>]) -> Option<&'a BindOp<'a>> {
    bindings.iter().find_map(|binding| match binding {
        BindingOp::Bind(bind) if bind.name.is_none() => Some(&**bind),
        _ => None,
    })
}

fn first_on_spread<'a>(bindings: &'a [BindingOp<'a>]) -> Option<&'a OnOp<'a>> {
    bindings.iter().find_map(|binding| match binding {
        BindingOp::On(on) if on.name.is_none() => Some(&**on),
        _ => None,
    })
}

fn push_spread_separator(cx: &mut EmitCx<'_>, first: &mut bool) {
    if !*first {
        cx.buf.push(", ");
    }
    *first = false;
}

fn emit_bind_spread_expr(cx: &mut EmitCx<'_>, bind: &BindOp<'_>) -> Result<(), EmitError> {
    cx.buf.push(js_value(bind)?.source);
    Ok(())
}

fn emit_on_spread_expr(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let js = match on.handler {
        Some(ExprRef::Js(js)) => js,
        Some(expr) => {
            return Err(EmitError::unsupported_at(
                Reason::ObjectOnHandlerNotJs,
                expr.span(),
            ));
        }
        None => {
            return Err(EmitError::unsupported_at(
                Reason::ObjectOnHandlerNotJs,
                on.span,
            ));
        }
    };
    cx.buf.use_to_handlers();
    cx.buf.push(Buf::to_handlers_alias());
    cx.buf.push("(");
    cx.buf.push(js.source);
    cx.buf.push(", true)");
    Ok(())
}

fn emit_props_object(
    cx: &mut EmitCx<'_>,
    slot: &SlotOp<'_>,
    key: Option<&str>,
) -> Result<(), EmitError> {
    let list = pieces(&slot.attributes, &slot.bindings, false)?;
    cx.buf.push("{");
    let mut first = true;
    if let Some(key) = key {
        cx.buf.push(" key: ");
        cx.buf.push(key);
        first = false;
    }
    for piece in list.iter() {
        if is_object_spread_piece(piece) {
            continue;
        }
        push_prop_separator(cx, &mut first);
        match piece {
            Piece::Attr(attr) => {
                push_camel_key(cx, attr.name);
                cx.buf.push(": \"");
                if let Some(value) = attr.value {
                    cx.buf.push(escape_js_string(value).as_str());
                }
                cx.buf.push("\"");
            }
            Piece::Bind(bind) => {
                if !emit_dynamic_bind_pair(cx, bind)? {
                    let key = super::props::static_bind_key(
                        bind,
                        super::props::StaticBindKeyCasing::Camelize,
                    )?;
                    let js = js_value(bind)?;
                    push_key(cx, key.as_str());
                    cx.buf.push(": ");
                    cx.buf.push(js.source);
                }
            }
            Piece::VueHtml(html) => {
                super::html::emit_pair(cx, html)?;
            }
            Piece::VueText(text) => {
                super::vtext::emit_pair(cx, text)?;
            }
            Piece::On(on) => {
                super::on::emit_on_pair(cx, on, false)?;
            }
            Piece::ModelValue { .. } | Piece::ModelUpdate { .. } | Piece::ModelModifiers { .. } => {
                return Err(EmitError::unsupported_at(
                    Reason::SlotOutletPropKind,
                    piece.span(),
                ));
            }
        }
    }
    if !first {
        cx.buf.push(" ");
    }
    cx.buf.push("}");
    Ok(())
}

fn is_object_spread_piece(piece: &Piece<'_>) -> bool {
    match piece {
        Piece::Bind(bind) => bind.name.is_none(),
        Piece::On(on) => on.name.is_none(),
        _ => false,
    }
}

fn push_prop_separator(cx: &mut EmitCx<'_>, first: &mut bool) {
    if !*first {
        cx.buf.push(",");
    }
    cx.buf.push(" ");
    *first = false;
}

fn push_camel_key(cx: &mut EmitCx<'_>, name: &str) {
    let key = camelize(name);
    push_key(cx, key.as_str());
}

fn push_key(cx: &mut EmitCx<'_>, key: &str) {
    if is_valid_js_identifier(key) {
        cx.buf.push(key);
    } else {
        cx.buf.push("\"");
        cx.buf.push(escape_js_string(key).as_str());
        cx.buf.push("\"");
    }
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
        Op::Element(_) | Op::Component(_) | Op::If(_) | Op::For(_) | Op::Slot(_) => {
            start_fallback_item(cx, compact, first);
            emit_array_child(cx, op)
        }
    }
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
                    emit_to_display_string(cx, part.text.as_str());
                } else {
                    emit_plain_text_vnode(cx, part.text.as_str());
                }
            }
            Ok(())
        }
        ExprRef::Foreign(_) | ExprRef::Filter(_) | ExprRef::Opaque(_) => Err(
            EmitError::unsupported_at(Reason::TextExpressionNotEmittable, interp.expression.span()),
        ),
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
