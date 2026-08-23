//! Static attrs plus static-name `ui.bind` props / patch flags.

use alloc::vec::Vec as StdVec;

use vize_disegno::expr::{ExprRef, JsExpr};
use vize_disegno::op::{Attribute, BindOp, BindingOp, DynamicName, ElementOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::js::{escape_js_string, is_valid_js_identifier};

pub(super) struct Patch<'a> {
    pub flag: i32,
    pub dynamic_props: StdVec<&'a str>,
}

pub(super) fn admit_bindings(element: &ElementOp<'_>) -> Result<(), EmitError> {
    let mut class = false;
    let mut style = false;
    for binding in element.bindings.iter() {
        let BindingOp::Bind(bind) = binding else {
            return Err(EmitError::Unsupported);
        };
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
    if style && has_attr(element, "style") {
        // Static+dynamic style merge parses CSS declarations; next installment.
        return Err(EmitError::Unsupported);
    }
    Ok(())
}

pub(super) fn bind_patch<'a>(element: &'a ElementOp<'a>) -> Patch<'a> {
    let mut flag = 0i32;
    let mut dynamic_props = StdVec::new();
    for binding in element.bindings.iter() {
        let BindingOp::Bind(bind) = binding else {
            continue;
        };
        let Ok(name) = static_bind_name(bind) else {
            continue;
        };
        match name {
            "class" => flag |= 2,
            "style" => flag |= 4,
            "key" => {}
            _ => {
                flag |= 8;
                if !dynamic_props.contains(&name) {
                    dynamic_props.push(name);
                }
            }
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
) -> Result<(), EmitError> {
    let pieces = pieces(element)?;
    let skip_class = has_bind_named(element, "class");
    let visible: StdVec<&Piece<'_>> = pieces
        .iter()
        .filter(|piece| !matches!(piece, Piece::Attr(attr) if skip_class && attr.name == "class"))
        .collect();
    let multiline =
        visible.len() > 1 || has_bind_named(element, "class") || has_bind_named(element, "style");
    if multiline {
        cx.buf.push("{");
        cx.buf.indent();
    } else {
        cx.buf.push("{ ");
    }
    for (i, piece) in visible.iter().enumerate() {
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
        }
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

pub(super) fn patch_flag_comment(flag: i32) -> &'static str {
    match flag {
        1 => "TEXT",
        2 => "CLASS",
        3 => "TEXT, CLASS",
        4 => "STYLE",
        5 => "TEXT, STYLE",
        6 => "CLASS, STYLE",
        7 => "TEXT, CLASS, STYLE",
        8 => "PROPS",
        9 => "TEXT, PROPS",
        10 => "CLASS, PROPS",
        11 => "TEXT, CLASS, PROPS",
        12 => "STYLE, PROPS",
        13 => "TEXT, STYLE, PROPS",
        14 => "CLASS, STYLE, PROPS",
        15 => "TEXT, CLASS, STYLE, PROPS",
        _ => "UNKNOWN",
    }
}

enum Piece<'a> {
    Attr(&'a Attribute<'a>),
    Bind(&'a BindOp<'a>),
}

fn pieces<'a>(element: &'a ElementOp<'a>) -> Result<StdVec<Piece<'a>>, EmitError> {
    let mut out = StdVec::new();
    for attr in element.attributes.iter() {
        out.push(Piece::Attr(attr));
    }
    for binding in element.bindings.iter() {
        let BindingOp::Bind(bind) = binding else {
            return Err(EmitError::Unsupported);
        };
        out.push(Piece::Bind(bind));
    }
    out.sort_by_key(|piece| match piece {
        Piece::Attr(attr) => attr.span.start,
        Piece::Bind(bind) => bind.span.start,
    });
    Ok(out)
}

fn static_bind_name<'a>(bind: &'a BindOp<'a>) -> Result<&'a str, EmitError> {
    match bind.name {
        Some(DynamicName::Static(name)) => Ok(name),
        Some(DynamicName::Dynamic(_)) | None => Err(EmitError::Unsupported),
    }
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
