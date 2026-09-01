use vize_s0::String;
use vize_s2::op::BindOp;

use super::super::EmitError;
use super::super::hoist::push_attr_pair;
use super::super::js::{escape_js_string, is_valid_js_identifier, js_expr_source};
use super::super::props::{Piece, bind_value_is_static_patchless, static_bind_key};
use super::super::props_bind::{StaticBindKey, StaticBindKeyCasing};
use super::super::props_value::{BindValue, bind_value};
use super::legacy_constant::legacy_global_constant_expr;

pub(super) fn static_hoist_prop<'a>(
    out: &mut String,
    piece: &Piece<'a>,
) -> Result<Option<HoistKey<'a>>, EmitError> {
    let Some(key) = hoist_key(piece)? else {
        return Ok(None);
    };
    match piece {
        Piece::Attr(attr) => {
            push_attr_pair(out, attr);
        }
        Piece::Bind(bind) => {
            push_key(out, key.as_str());
            out.push_str(": ");
            if let Some(js) = bind_value(bind)?.js() {
                let source = js_expr_source(js);
                out.push_str(source.as_str());
            }
        }
        _ => return Ok(None),
    }
    Ok(Some(key))
}

pub(super) fn component_hoist_prop<'a>(
    out: &mut String,
    piece: &Piece<'a>,
) -> Result<Option<(HoistKey<'a>, bool)>, EmitError> {
    match piece {
        Piece::Attr(attr) if attr.name != "ref" => {
            push_attr_pair(out, attr);
            Ok(Some((HoistKey::Borrowed(attr.name), false)))
        }
        Piece::Bind(bind) => {
            let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
                return Ok(None);
            };
            let dynamic_value = !bind_value_is_static_patchless(bind);
            if matches!(key.as_str(), "ref" | "class") {
                return Ok(None);
            }
            let value = bind_value(bind)?;
            let Some(js) = value.js() else {
                return Ok(None);
            };
            if dynamic_value && !legacy_global_constant_expr(js.ast, js.source) {
                return Ok(None);
            }
            push_key(out, key.as_str());
            out.push_str(": ");
            let source = js_expr_source(js);
            out.push_str(source.as_str());
            Ok(Some((HoistKey::StaticBind(key), dynamic_value)))
        }
        _ => Ok(None),
    }
}

pub(super) fn has_prior_hoist_key(pieces: &[Piece<'_>], key: &str) -> Result<bool, EmitError> {
    for piece in pieces {
        if hoist_key(piece)?.is_some_and(|prior| prior.as_str() == key) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn has_prior_component_hoist_key(
    pieces: &[Piece<'_>],
    key: &str,
) -> Result<bool, EmitError> {
    for piece in pieces {
        if component_hoist_key(piece)?.is_some_and(|prior| prior.as_str() == key) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn bind_value_is_legacy_static_prop(bind: &BindOp<'_>) -> bool {
    let Ok(value) = bind_value(bind) else {
        return false;
    };
    let Some(js) = value.js() else {
        return false;
    };
    bind_value_is_static_patchless(bind)
        || js.source.trim() == "undefined"
        || legacy_static_style_prop(bind, &value)
}

pub(super) fn multiline_props_object(props: &[String], line_indent: usize) -> String {
    let mut out = String::from("{");
    for (index, prop) in props.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('\n');
        push_spaces(&mut out, line_indent + 2);
        out.push_str(prop.as_str());
    }
    out.push('\n');
    push_spaces(&mut out, line_indent);
    out.push('}');
    out
}

fn hoist_key<'a>(piece: &Piece<'a>) -> Result<Option<HoistKey<'a>>, EmitError> {
    match piece {
        Piece::Attr(attr) if attr.name != "ref" => Ok(Some(HoistKey::Borrowed(attr.name))),
        Piece::Bind(bind) if bind_value_is_legacy_static_prop(bind) => {
            let key = static_bind_key(bind, StaticBindKeyCasing::Preserve)?;
            if matches!(key.as_str(), "ref" | "class") {
                return Ok(None);
            }
            Ok(Some(HoistKey::StaticBind(key)))
        }
        _ => Ok(None),
    }
}

fn component_hoist_key<'a>(piece: &Piece<'a>) -> Result<Option<HoistKey<'a>>, EmitError> {
    match piece {
        Piece::Attr(attr) if attr.name != "ref" => Ok(Some(HoistKey::Borrowed(attr.name))),
        Piece::Bind(bind) => {
            let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
                return Ok(None);
            };
            if key.as_str() == "ref"
                || (key.as_str() == "class" && !bind_value_is_static_patchless(bind))
            {
                return Ok(None);
            }
            Ok(Some(HoistKey::StaticBind(key)))
        }
        _ => Ok(None),
    }
}

fn legacy_static_style_prop(bind: &BindOp<'_>, value: &BindValue<'_>) -> bool {
    let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
        return false;
    };
    key.as_str() == "style" && super::super::style::legacy_static_style_object(value)
}

pub(super) enum HoistKey<'a> {
    Borrowed(&'a str),
    StaticBind(StaticBindKey<'a>),
}

impl HoistKey<'_> {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::StaticBind(key) => key.as_str(),
        }
    }
}

fn push_key(out: &mut String, key: &str) {
    if !is_valid_js_identifier(key) {
        out.push('"');
        out.push_str(escape_js_string(key).as_str());
        out.push('"');
        return;
    }
    out.push_str(key);
}

fn push_spaces(out: &mut String, width: usize) {
    out.extend(core::iter::repeat_n(' ', width));
}
