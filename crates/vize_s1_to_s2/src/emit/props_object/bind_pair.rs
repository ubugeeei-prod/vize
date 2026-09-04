//! Static attribute and `ui.bind` pair emission inside a props object.

use vize_s2::op::{Attribute, BindOp};

use super::super::js::{escape_js_string, push_ident_key};
use super::super::props_bind::{self, StaticBindKeyCasing};
use super::super::{EmitCx, EmitError, props_value, style};
use super::Piece;

pub(super) fn emit_static_pair(cx: &mut EmitCx<'_>, attr: &Attribute<'_>) {
    emit_ref_for(cx, attr.name);
    push_ident_key(cx, attr.name);
    cx.buf.push(": \"");
    if let Some(value) = attr.value {
        cx.buf.push(escape_js_string(value).as_str());
    }
    cx.buf.push("\"");
}

pub(super) fn emit_bind_pair(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    bind: &BindOp<'_>,
    skip_normalize: bool,
    is_plain_element: bool,
) -> Result<(), EmitError> {
    if props_bind::emit_dynamic_bind_pair(cx, bind)? {
        return Ok(());
    }
    let raw_name = props_bind::static_bind_name(bind)?;
    let key = props_bind::static_bind_key(bind, StaticBindKeyCasing::Preserve)?;
    let value = props_value::bind_value(bind)?;
    let static_style = static_style_piece(pieces);
    let skip_normalize = skip_normalize
        || style::bind_skips_normalize(
            raw_name,
            is_plain_element,
            static_style.is_some(),
            &value,
            &cx.scope,
        );
    emit_ref_for(cx, key.as_str());
    push_ident_key(cx, key.as_str());
    cx.buf.push(": ");
    match raw_name {
        "class" => match value.js() {
            Some(_) if skip_normalize && !super::pieces_have_static_attr(pieces, "class") => {
                value.emit_authored(cx, bind)?;
            }
            Some(js) => {
                crate::emit::props_class::emit_class_value(cx, pieces, bind, js, skip_normalize)?;
            }
            None => {
                if !skip_normalize {
                    cx.buf.use_normalize_class();
                    cx.buf.push(crate::emit::buf::Buf::normalize_class_alias());
                    cx.buf.push("(");
                }
                value.emit(cx, bind)?;
                if !skip_normalize {
                    cx.buf.push(")");
                }
            }
        },
        "style" => match value.js() {
            Some(_) if skip_normalize && static_style.is_none() => {
                value.emit_authored(cx, bind)?;
            }
            Some(js) => style::emit_style_value(cx, static_style, bind, js, skip_normalize)?,
            None => value.emit(cx, bind)?,
        },
        _ => value.emit_authored(cx, bind)?,
    }
    Ok(())
}

fn emit_ref_for(cx: &mut EmitCx<'_>, name: &str) {
    if cx.in_v_for && name == "ref" {
        cx.buf.push("ref_for: true, ");
    }
}

fn static_style_piece<'a>(pieces: &'a [Piece<'a>]) -> Option<(&'a Attribute<'a>, &'a str)> {
    pieces.iter().find_map(|piece| match piece {
        Piece::Attr(attr) if attr.name == "style" => attr.value.map(|value| (*attr, value)),
        _ => None,
    })
}
