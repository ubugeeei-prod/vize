//! Static attrs plus static-name `ui.bind` / `ui.on` props / patch flags.

use alloc::vec::Vec as StdVec;

use vize_carton::String;
use vize_disegno::expr::{ExprRef, JsExpr};
use vize_disegno::op::{Attribute, BindOp, BindingOp, DynamicName, OnOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::js::{escape_js_string, is_valid_js_identifier};
use super::on::{
    admit_on, emit_on_pair, event_key_for, is_inline_handler_source, needs_hydration, wraps_on,
};

pub(super) struct Patch {
    pub flag: i32,
    pub dynamic_props: StdVec<String>,
}

pub(super) fn admit_bindings(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<(), EmitError> {
    let mut class = false;
    let mut style = false;
    let mut events = StdVec::new();
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) if bind.name.is_none() => {
                super::merge::admit_object(bind)?;
            }
            BindingOp::On(on) if on.name.is_none() => {
                super::merge::admit_object_on(on)?;
            }
            BindingOp::Bind(bind) => {
                let name = static_bind_name(bind)?;
                if name == "ref" || !bind.modifiers.is_empty() {
                    return Err(EmitError::Unsupported);
                }
                let ExprRef::Js(_) = bind.value.ok_or(EmitError::Unsupported)? else {
                    return Err(EmitError::Unsupported);
                };
                match name {
                    "class" if class => return Err(EmitError::Unsupported),
                    "class" => class = true,
                    "style" if style => return Err(EmitError::Unsupported),
                    "style" => style = true,
                    _ => {}
                }
            }
            BindingOp::On(on) => admit_on(on, &mut events)?,
            BindingOp::SlotContent(_) => {}
            _ => return Err(EmitError::Unsupported),
        }
    }
    if style && has_attr(attributes, "style") {
        // Static+dynamic style merge parses CSS declarations; next installment.
        return Err(EmitError::Unsupported);
    }
    Ok(())
}

pub(super) fn bind_patch(bindings: &[BindingOp<'_>], is_component: bool) -> Patch {
    if super::merge::has_object_spread(bindings) {
        return super::merge::object_patch(bindings, is_component);
    }
    let mut flag = 0i32;
    let mut dynamic_props = StdVec::new();
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) => {
                let Ok(name) = static_bind_name(bind) else {
                    continue;
                };
                match name {
                    "class" if !is_component => flag |= 2,
                    "style" if !is_component => flag |= 4,
                    "key" => {}
                    _ => {
                        flag |= 8;
                        let owned = String::from(name);
                        if !dynamic_props.contains(&owned) {
                            dynamic_props.push(owned);
                        }
                    }
                }
            }
            BindingOp::On(on) => {
                let Ok(key) = event_key_for(on) else {
                    continue;
                };
                flag |= 8;
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

pub(super) fn emit_bind_props(
    cx: &mut EmitCx<'_>,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    if_key: Option<&str>,
) -> Result<(), EmitError> {
    if super::merge::has_object_spread(bindings) {
        return super::merge::emit_spread_props(cx, attributes, bindings, if_key);
    }
    let pieces = pieces(attributes, bindings)?;
    emit_props_object(cx, &pieces, if_key, false)
}

pub(super) fn emit_props_object(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    if_key: Option<&str>,
    skip_normalize: bool,
) -> Result<(), EmitError> {
    let skip_class = pieces_have_named(pieces, "class");
    let skip_key = if_key.is_some();
    let visible: StdVec<&Piece<'_>> = pieces
        .iter()
        .filter(|piece| {
            !matches!(
                piece,
                Piece::Attr(attr) if (skip_class && attr.name == "class")
                    || (skip_key && attr.name == "key")
            )
        })
        .collect();
    if let Some(key) = if_key
        && visible.is_empty()
    {
        cx.buf.push("{ key: ");
        cx.buf.push(key);
        cx.buf.push(" }");
        return Ok(());
    }
    let extra = usize::from(if_key.is_some());
    let multiline = visible.len() + extra > 1
        || pieces_have_named(pieces, "class")
        || pieces_have_named(pieces, "style")
        || pieces_have_inline_on(pieces);
    if multiline {
        cx.buf.push("{");
        cx.buf.indent();
    } else {
        cx.buf.push("{ ");
    }
    let mut i = 0;
    if let Some(key) = if_key {
        if multiline {
            cx.buf.newline();
        }
        cx.buf.push("key: ");
        cx.buf.push(key);
        i = 1;
    }
    for piece in visible.iter() {
        if i > 0 {
            cx.buf.push(",");
        }
        if multiline {
            cx.buf.newline();
        } else if i > 0 {
            cx.buf.push(" ");
        }
        match piece {
            Piece::Attr(attr) => emit_static_pair(cx, attr),
            Piece::Bind(bind) => emit_bind_pair(cx, pieces, bind, skip_normalize)?,
            Piece::On(on) => emit_on_pair(cx, on)?,
        }
        i += 1;
    }
    if multiline {
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}");
    } else {
        cx.buf.push(" }");
    }
    Ok(())
}

pub(super) enum Piece<'a> {
    Attr(&'a Attribute<'a>),
    Bind(&'a BindOp<'a>),
    On(&'a OnOp<'a>),
}

pub(super) fn pieces<'a>(
    attributes: &'a [Attribute<'a>],
    bindings: &'a [BindingOp<'a>],
) -> Result<StdVec<Piece<'a>>, EmitError> {
    let mut out = StdVec::new();
    for attr in attributes.iter() {
        out.push(Piece::Attr(attr));
    }
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) => out.push(Piece::Bind(bind)),
            BindingOp::On(on) => out.push(Piece::On(on)),
            BindingOp::SlotContent(_) => {}
            _ => return Err(EmitError::Unsupported),
        }
    }
    out.sort_by_key(|piece| match piece {
        Piece::Attr(attr) => attr.span.start,
        Piece::Bind(bind) => bind.span.start,
        Piece::On(on) => on.span.start,
    });
    Ok(out)
}

pub(super) fn static_bind_name<'a>(bind: &'a BindOp<'a>) -> Result<&'a str, EmitError> {
    match bind.name {
        Some(DynamicName::Static(name)) => Ok(name),
        Some(DynamicName::Dynamic(_)) | None => Err(EmitError::Unsupported),
    }
}

fn has_attr(attributes: &[Attribute<'_>], name: &str) -> bool {
    attributes.iter().any(|attr| attr.name == name)
}

fn pieces_have_named(pieces: &[Piece<'_>], name: &str) -> bool {
    pieces.iter().any(|piece| {
        matches!(piece, Piece::Bind(bind) if matches!(bind.name, Some(DynamicName::Static(n)) if n == name))
    })
}

fn pieces_have_inline_on(pieces: &[Piece<'_>]) -> bool {
    pieces.iter().any(|piece| match piece {
        Piece::On(on) => {
            wraps_on(on)
                || match on.handler {
                    Some(ExprRef::Js(js)) => is_inline_handler_source(js.source),
                    _ => false,
                }
        }
        _ => false,
    })
}

fn emit_static_pair(cx: &mut EmitCx<'_>, attr: &Attribute<'_>) {
    push_key(cx, attr.name);
    cx.buf.push(": \"");
    if let Some(value) = attr.value {
        cx.buf.push(escape_js_string(value).as_str());
    }
    cx.buf.push("\"");
}

fn emit_bind_pair(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    bind: &BindOp<'_>,
    skip_normalize: bool,
) -> Result<(), EmitError> {
    let name = static_bind_name(bind)?;
    let js = js_value(bind)?;
    push_key(cx, name);
    cx.buf.push(": ");
    match name {
        "class" => emit_class_value(cx, pieces, bind, js, skip_normalize),
        "style" => emit_style_value(cx, js, skip_normalize),
        _ => cx.buf.push(js.source),
    }
    Ok(())
}

fn emit_class_value(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    bind: &BindOp<'_>,
    js: &JsExpr<'_>,
    skip_normalize: bool,
) {
    if !skip_normalize {
        cx.buf.use_normalize_class();
        cx.buf.push(Buf::normalize_class_alias());
        cx.buf.push("(");
    }
    if let Some(static_class) = pieces.iter().find_map(|piece| match piece {
        Piece::Attr(attr) if attr.name == "class" => Some(*attr),
        _ => None,
    }) {
        let before = static_class.span.start <= bind.span.start;
        cx.buf.push("[");
        if before {
            cx.buf.push("\"");
            cx.buf
                .push(escape_js_string(static_class.value.unwrap_or("")).as_str());
            cx.buf.push("\", ");
            cx.buf.push(js.source);
        } else {
            cx.buf.push(js.source);
            cx.buf.push(", \"");
            cx.buf
                .push(escape_js_string(static_class.value.unwrap_or("")).as_str());
            cx.buf.push("\"");
        }
        cx.buf.push("]");
    } else {
        cx.buf.push(js.source);
    }
    if !skip_normalize {
        cx.buf.push(")");
    }
}

fn emit_style_value(cx: &mut EmitCx<'_>, js: &JsExpr<'_>, skip_normalize: bool) {
    let object_literal = js.source.trim_start().starts_with('{');
    let wrap = !skip_normalize && !object_literal;
    if wrap {
        cx.buf.use_normalize_style();
        cx.buf.push(Buf::normalize_style_alias());
        cx.buf.push("(");
    }
    cx.buf.push(js.source);
    if wrap {
        cx.buf.push(")");
    }
}

pub(super) fn js_value<'a>(bind: &'a BindOp<'a>) -> Result<&'a JsExpr<'a>, EmitError> {
    match bind.value {
        Some(ExprRef::Js(js)) => Ok(js),
        _ => Err(EmitError::Unsupported),
    }
}

fn push_key(cx: &mut EmitCx<'_>, name: &str) {
    if !is_valid_js_identifier(name) {
        cx.buf.push("\"");
        cx.buf.push(name);
        cx.buf.push("\"");
    } else {
        cx.buf.push(name);
    }
}
