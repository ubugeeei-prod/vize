//! Object-spread `v-bind` (`ui.bind` with no name): `normalizeProps` /
//! `guardReactiveProps` when the spread is alone, `mergeProps` when it
//! sits beside other props. `v-on` object form stays unsupported.

use alloc::vec::Vec as StdVec;

use vize_carton::String;
use vize_disegno::op::{Attribute, BindOp, BindingOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::on::{event_key_for, needs_hydration};
use super::props::{Patch, Piece, emit_props_object, js_value, pieces, static_bind_name};

pub(super) fn has_object_bind(bindings: &[BindingOp<'_>]) -> bool {
    bindings
        .iter()
        .any(|binding| matches!(binding, BindingOp::Bind(bind) if bind.name.is_none()))
}

pub(super) fn admit_object(bind: &BindOp<'_>) -> Result<(), EmitError> {
    if !bind.modifiers.is_empty() {
        return Err(EmitError::Unsupported);
    }
    js_value(bind).map(|_| ())
}

pub(super) fn object_patch(bindings: &[BindingOp<'_>], is_component: bool) -> Patch {
    let mut dynamic_props = StdVec::new();
    let mut flag = 16i32;
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) if bind.name.is_none() => {}
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
) -> Result<(), EmitError> {
    let args = merge_args(attributes, bindings, if_key)?;
    let only_spread = args.iter().all(|arg| matches!(arg, Arg::Spread(_)));
    if only_spread {
        let Arg::Spread(bind) = args[0] else {
            return Err(EmitError::Unsupported);
        };
        return emit_normalize_guard(cx, bind);
    }
    cx.buf.use_merge_props();
    cx.buf.push(Buf::merge_props_alias());
    cx.buf.push("(");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            cx.buf.push(", ");
        }
        match arg {
            Arg::Spread(bind) => cx.buf.push(js_value(bind)?.source),
            Arg::Object { if_key, pieces } => {
                emit_props_object(cx, pieces, *if_key, true)?;
            }
        }
    }
    cx.buf.push(")");
    Ok(())
}

enum Arg<'a> {
    Object {
        if_key: Option<&'a str>,
        pieces: StdVec<Piece<'a>>,
    },
    Spread(&'a BindOp<'a>),
}

fn merge_args<'a>(
    attributes: &'a [Attribute<'a>],
    bindings: &'a [BindingOp<'a>],
    if_key: Option<&'a str>,
) -> Result<StdVec<Arg<'a>>, EmitError> {
    let mut args = StdVec::new();
    let mut current = StdVec::new();
    for piece in pieces(attributes, bindings)? {
        if let Piece::Bind(bind) = piece
            && bind.name.is_none()
        {
            if !current.is_empty() {
                args.push(Arg::Object {
                    if_key: None,
                    pieces: current,
                });
                current = StdVec::new();
            }
            args.push(Arg::Spread(bind));
            continue;
        }
        current.push(piece);
    }
    if !current.is_empty() {
        args.push(Arg::Object {
            if_key: None,
            pieces: current,
        });
    }
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
