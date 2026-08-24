//! Object-spread `v-bind` (`ui.bind` with no name) and object `v-on`
//! (`ui.on` with no name): `normalizeProps` / `guardReactiveProps` when
//! a bind spread is alone, `toHandlers(..., true)` when an on spread is
//! alone, `mergeProps` when a spread sits beside other props or the
//! two spread kinds mix. The `, true` is Vue's `handlerOnly` flag — the
//! shipped `generate_von_object_exp` always emits it.

use alloc::vec::Vec as StdVec;

use vize_carton::String;
use vize_disegno::expr::ExprRef;
use vize_disegno::op::{Attribute, BindOp, BindingOp, OnOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::on::{event_key_for, needs_hydration};
use super::props::{Patch, Piece, emit_props_object, js_value, pieces, static_bind_name};

pub(super) fn has_object_spread(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(|binding| match binding {
        BindingOp::Bind(bind) if bind.name.is_none() => true,
        BindingOp::On(on) if on.name.is_none() => true,
        _ => false,
    })
}

pub(super) fn admit_object(bind: &BindOp<'_>) -> Result<(), EmitError> {
    if !bind.modifiers.is_empty() {
        return Err(EmitError::Unsupported);
    }
    js_value(bind).map(|_| ())
}

pub(super) fn admit_object_on(on: &OnOp<'_>) -> Result<(), EmitError> {
    if !on.modifiers.is_empty() {
        return Err(EmitError::Unsupported);
    }
    match on.handler {
        Some(ExprRef::Js(_)) => Ok(()),
        _ => Err(EmitError::Unsupported),
    }
}

pub(super) fn object_patch(bindings: &[BindingOp<'_>], is_component: bool) -> Patch {
    let mut dynamic_props = StdVec::new();
    let mut flag = 16i32;
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) if bind.name.is_none() => {}
            BindingOp::On(on) if on.name.is_none() => {}
            BindingOp::Bind(bind) => {
                let Ok(name) = static_bind_name(bind) else {
                    continue;
                };
                if matches!(name, "class" | "style" | "key") {
                    continue;
                }
                let owned = String::from(name);
                if !dynamic_props.contains(&owned) {
                    dynamic_props.push(owned);
                }
            }
            BindingOp::On(on) => {
                let Ok(key) = event_key_for(on) else {
                    continue;
                };
                if !dynamic_props.contains(&key) {
                    dynamic_props.push(key.clone());
                }
                if !is_component && needs_hydration(key.as_str(), on) {
                    flag |= 32;
                }
            }
            _ => {}
        }
    }
    Patch {
        flag,
        dynamic_props,
    }
}

pub(super) fn emit_spread_props(
    cx: &mut EmitCx<'_>,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    if_key: Option<&str>,
    skip_is: bool,
) -> Result<(), EmitError> {
    let args = merge_args(attributes, bindings, if_key, skip_is)?;
    if let Some(lone) = lone_kind_spread(&args) {
        return match lone {
            Arg::BindSpread(bind) => emit_normalize_guard(cx, bind),
            Arg::OnSpread(on) => emit_to_handlers(cx, on),
            Arg::Object { .. } => Err(EmitError::Unsupported),
        };
    }
    cx.buf.use_merge_props();
    cx.buf.push(Buf::merge_props_alias());
    cx.buf.push("(");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            cx.buf.push(", ");
        }
        match arg {
            Arg::BindSpread(bind) => cx.buf.push(js_value(bind)?.source),
            Arg::OnSpread(on) => emit_to_handlers(cx, on)?,
            Arg::Object { if_key, pieces } => {
                emit_props_object(cx, pieces, *if_key, true)?;
            }
        }
    }
    cx.buf.push(")");
    Ok(())
}

/// Every arg is the same spread kind (all binds, or all ons). Vue keeps
/// only the first of that kind when nothing else is present.
fn lone_kind_spread<'a>(args: &'a [Arg<'a>]) -> Option<&'a Arg<'a>> {
    if args.is_empty() {
        return None;
    }
    let all_bind = args.iter().all(|arg| matches!(arg, Arg::BindSpread(_)));
    let all_on = args.iter().all(|arg| matches!(arg, Arg::OnSpread(_)));
    if all_bind || all_on {
        args.first()
    } else {
        None
    }
}

enum Arg<'a> {
    Object {
        if_key: Option<&'a str>,
        pieces: StdVec<Piece<'a>>,
    },
    BindSpread(&'a BindOp<'a>),
    OnSpread(&'a OnOp<'a>),
}

fn merge_args<'a>(
    attributes: &'a [Attribute<'a>],
    bindings: &'a [BindingOp<'a>],
    if_key: Option<&'a str>,
    skip_is: bool,
) -> Result<StdVec<Arg<'a>>, EmitError> {
    let mut args = StdVec::new();
    let mut current = StdVec::new();
    for piece in pieces(attributes, bindings, skip_is)? {
        match piece {
            Piece::Bind(bind) if bind.name.is_none() => {
                flush_object(&mut args, &mut current);
                args.push(Arg::BindSpread(bind));
            }
            Piece::On(on) if on.name.is_none() => {
                flush_object(&mut args, &mut current);
                args.push(Arg::OnSpread(on));
            }
            other => current.push(other),
        }
    }
    flush_object(&mut args, &mut current);
    if if_key.is_some() {
        match args.first_mut() {
            Some(Arg::Object { if_key: slot, .. }) => *slot = if_key,
            _ => args.insert(
                0,
                Arg::Object {
                    if_key,
                    pieces: StdVec::new(),
                },
            ),
        }
    }
    Ok(args)
}

fn flush_object<'a>(args: &mut StdVec<Arg<'a>>, current: &mut StdVec<Piece<'a>>) {
    if current.is_empty() {
        return;
    }
    args.push(Arg::Object {
        if_key: None,
        pieces: core::mem::take(current),
    });
}

fn emit_normalize_guard(cx: &mut EmitCx<'_>, bind: &BindOp<'_>) -> Result<(), EmitError> {
    let js = js_value(bind)?;
    cx.buf.use_normalize_props();
    cx.buf.use_guard_reactive_props();
    cx.buf.push(Buf::normalize_props_alias());
    cx.buf.push("(");
    cx.buf.push(Buf::guard_reactive_props_alias());
    cx.buf.push("(");
    cx.buf.push(js.source);
    cx.buf.push("))");
    Ok(())
}

fn emit_to_handlers(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let js = match on.handler {
        Some(ExprRef::Js(js)) => js,
        _ => return Err(EmitError::Unsupported),
    };
    cx.buf.use_to_handlers();
    cx.buf.push(Buf::to_handlers_alias());
    cx.buf.push("(");
    cx.buf.push(js.source);
    cx.buf.push(", true)");
    Ok(())
}
