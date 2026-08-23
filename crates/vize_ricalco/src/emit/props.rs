//! Static attrs plus static-name `ui.bind` / `ui.on` props / patch flags.

use alloc::vec::Vec as StdVec;

use vize_carton::{String, ToCompactString};
use vize_disegno::expr::{ExprRef, JsExpr};
use vize_disegno::op::{Attribute, BindOp, BindingOp, DynamicName, ElementOp, OnOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::js::{escape_js_string, is_valid_js_identifier};
use super::on::{
    admit_on, emit_on_pair, event_key, is_inline_handler_source, needs_hydration, static_on_name,
};

pub(super) struct Patch {
    pub flag: i32,
    pub dynamic_props: StdVec<String>,
}

pub(super) fn admit_bindings(element: &ElementOp<'_>) -> Result<(), EmitError> {
    let mut class = false;
    let mut style = false;
    let mut events = StdVec::new();
    for binding in element.bindings.iter() {
        match binding {
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
            _ => return Err(EmitError::Unsupported),
        }
    }
    if style && has_attr(element, "style") {
        // Static+dynamic style merge parses CSS declarations; next installment.
        return Err(EmitError::Unsupported);
    }
    Ok(())
}

pub(super) fn bind_patch(element: &ElementOp<'_>) -> Patch {
    let mut flag = 0i32;
    let mut dynamic_props = StdVec::new();
    for binding in element.bindings.iter() {
        match binding {
            BindingOp::Bind(bind) => {
                let Ok(name) = static_bind_name(bind) else {
                    continue;
                };
                match name {
                    "class" => flag |= 2,
                    "style" => flag |= 4,
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
                let Ok(name) = static_on_name(on) else {
                    continue;
                };
                let key = event_key(name);
                flag |= 8;
                if !dynamic_props.contains(&key) {
                    dynamic_props.push(key);
                }
                if needs_hydration(name) {
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
    element: &ElementOp<'_>,
    if_key: Option<&str>,
) -> Result<(), EmitError> {
    let pieces = pieces(element)?;
    let skip_class = has_bind_named(element, "class");
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
        || has_bind_named(element, "class")
        || has_bind_named(element, "style")
        || has_inline_on(element);
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
            Piece::Bind(bind) => emit_bind_pair(cx, element, bind)?,
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

const PATCH_NAMES: [(i32, &str); 6] = [
    (1, "TEXT"),
    (2, "CLASS"),
    (4, "STYLE"),
    (8, "PROPS"),
    (16, "FULL_PROPS"),
    (32, "NEED_HYDRATION"),
];

pub(super) fn emit_patch_flag(cx: &mut EmitCx<'_>, flag: i32) {
    cx.buf.push(", ");
    cx.buf.push(flag.to_compact_string().as_str());
    cx.buf.push(" /* ");
    let mut first = true;
    for (bit, name) in PATCH_NAMES {
        if flag & bit == 0 {
            continue;
        }
        if !first {
            cx.buf.push(", ");
        }
        first = false;
        cx.buf.push(name);
    }
    if first {
        cx.buf.push("UNKNOWN");
    }
    cx.buf.push(" */");
}

enum Piece<'a> {
    Attr(&'a Attribute<'a>),
    Bind(&'a BindOp<'a>),
    On(&'a OnOp<'a>),
}

fn pieces<'a>(element: &'a ElementOp<'a>) -> Result<StdVec<Piece<'a>>, EmitError> {
    let mut out = StdVec::new();
    for attr in element.attributes.iter() {
        out.push(Piece::Attr(attr));
    }
    for binding in element.bindings.iter() {
        match binding {
            BindingOp::Bind(bind) => out.push(Piece::Bind(bind)),
            BindingOp::On(on) => out.push(Piece::On(on)),
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

fn static_bind_name<'a>(bind: &'a BindOp<'a>) -> Result<&'a str, EmitError> {
    match bind.name {
        Some(DynamicName::Static(name)) => Ok(name),
        Some(DynamicName::Dynamic(_)) | None => Err(EmitError::Unsupported),
    }
}

fn has_inline_on(element: &ElementOp<'_>) -> bool {
    element.bindings.iter().any(|binding| match binding {
        BindingOp::On(on) => match on.handler {
            Some(ExprRef::Js(js)) => is_inline_handler_source(js.source),
            _ => false,
        },
        _ => false,
    })
}

fn has_attr(element: &ElementOp<'_>, name: &str) -> bool {
    element.attributes.iter().any(|attr| attr.name == name)
}

fn has_bind_named(element: &ElementOp<'_>, name: &str) -> bool {
    element.bindings.iter().any(|binding| {
        matches!(binding, BindingOp::Bind(bind) if matches!(bind.name, Some(DynamicName::Static(n)) if n == name))
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
    element: &ElementOp<'_>,
    bind: &BindOp<'_>,
) -> Result<(), EmitError> {
    let name = static_bind_name(bind)?;
    let js = js_value(bind)?;
    push_key(cx, name);
    cx.buf.push(": ");
    match name {
        "class" => emit_class_value(cx, element, js),
        "style" => emit_style_value(cx, js),
        _ => cx.buf.push(js.source),
    }
    Ok(())
}

fn emit_class_value(cx: &mut EmitCx<'_>, element: &ElementOp<'_>, js: &JsExpr<'_>) {
    cx.buf.use_normalize_class();
    cx.buf.push(Buf::normalize_class_alias());
    cx.buf.push("(");
    if let Some(static_class) = element.attributes.iter().find(|attr| attr.name == "class") {
        let before = static_class.span.start
            <= element
                .bindings
                .iter()
                .find_map(|binding| match binding {
                    BindingOp::Bind(bind)
                        if matches!(bind.name, Some(DynamicName::Static("class"))) =>
                    {
                        Some(bind.span.start)
                    }
                    _ => None,
                })
                .unwrap_or(u32::MAX);
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
    cx.buf.push(")");
}

fn emit_style_value(cx: &mut EmitCx<'_>, js: &JsExpr<'_>) {
    let object_literal = js.source.trim_start().starts_with('{');
    if !object_literal {
        cx.buf.use_normalize_style();
        cx.buf.push(Buf::normalize_style_alias());
        cx.buf.push("(");
    }
    cx.buf.push(js.source);
    if !object_literal {
        cx.buf.push(")");
    }
}

fn js_value<'a>(bind: &'a BindOp<'a>) -> Result<&'a JsExpr<'a>, EmitError> {
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
