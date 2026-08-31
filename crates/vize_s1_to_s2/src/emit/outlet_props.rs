//! `<slot>` outlet prop emission.
//!
//! Slot outlet spreads intentionally follow the shipped lane's fixed order:
//! first object `v-bind`, then first object `v-on`, then entry props.

use vize_s0::camelize;
use vize_s2::op::{BindOp, BindingOp, OnOp, SlotOp};

use super::buf::Buf;
use super::js::{escape_js_string, is_valid_js_identifier};
use super::props::{
    Piece, StaticBindKeyCasing, bind_value, emit_dynamic_bind_pair, pieces, static_bind_key,
};
use super::{EmitCx, EmitError};
use super::{UnsupportedReason as Reason, merge};

pub(super) fn emit_props(
    cx: &mut EmitCx<'_>,
    slot: &SlotOp<'_>,
    key: Option<&str>,
) -> Result<(), EmitError> {
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
    bind_value(bind)?.emit(cx);
    Ok(())
}

fn emit_on_spread_expr(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let source = match on.handler {
        Some(expr) => super::js::expr_source(&expr, false)
            .ok_or_else(|| EmitError::unsupported_at(Reason::ObjectOnHandlerNotJs, expr.span()))?,
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
    cx.buf.push(source.as_str());
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
                    let key = static_bind_key(bind, StaticBindKeyCasing::Camelize)?;
                    push_key(cx, key.as_str());
                    cx.buf.push(": ");
                    bind_value(bind)?.emit(cx);
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
