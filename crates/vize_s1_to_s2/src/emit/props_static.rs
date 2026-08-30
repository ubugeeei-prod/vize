//! Inline static props for native element calls.

use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp};

use super::EmitCx;
use super::EmitError;
use super::hoist::{push_attr_pair, unique_attrs};
use super::js::{escape_js_string, is_valid_js_identifier};
use super::props::{Piece, bind_value_is_static_patchless, pieces, static_bind_key};
use super::props_bind::StaticBindKeyCasing;
use super::props_value::bind_value;

pub(super) fn root_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<Option<String>, EmitError> {
    let mut out = String::from("{ ");
    let mut seen: StdVec<String> = StdVec::new();
    let mut emitted = 0usize;
    for piece in pieces(attributes, bindings, false)? {
        let mut prop = String::default();
        let Some(key) = static_hoist_prop(&mut prop, &piece)? else {
            return Ok(None);
        };
        if seen.iter().any(|seen| seen == key.as_str()) {
            continue;
        }
        if emitted > 0 {
            out.push_str(", ");
        }
        out.push_str(prop.as_str());
        seen.push(key);
        emitted += 1;
    }
    if emitted == 0 {
        return Ok(None);
    }
    out.push_str(" }");
    Ok(Some(out))
}

fn static_hoist_prop(out: &mut String, piece: &Piece<'_>) -> Result<Option<String>, EmitError> {
    match piece {
        Piece::Attr(attr) if attr.name != "ref" => {
            push_attr_pair(out, attr);
            Ok(Some(String::from(attr.name)))
        }
        Piece::Bind(bind) if bind_value_is_static_patchless(bind) => {
            let key = static_bind_key(bind, StaticBindKeyCasing::Preserve)?;
            let key = String::from(key.as_str());
            if matches!(key.as_str(), "ref" | "class") {
                return Ok(None);
            }
            push_key(out, key.as_str());
            out.push_str(": ");
            if let Some(js) = bind_value(bind)?.js() {
                out.push_str(js.source);
            }
            Ok(Some(key))
        }
        _ => Ok(None),
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

pub(super) fn emit_inline<'a>(
    cx: &mut EmitCx<'_>,
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
) {
    let unique = unique_attrs(attributes);
    let multiline = unique.len() > 1 && !cx.in_v_for;
    if multiline {
        cx.buf.push("{");
        cx.buf.indent();
    } else {
        cx.buf.push("{ ");
    }
    for (i, attr) in unique.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        if multiline {
            cx.buf.newline();
        } else if i > 0 {
            cx.buf.push(" ");
        }
        if cx.in_v_for && attr.name == "ref" {
            cx.buf.push("ref_for: true, ");
        }
        let mut pair = String::default();
        push_attr_pair(&mut pair, attr);
        cx.buf.push(pair.as_str());
    }
    if multiline {
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}");
    } else {
        cx.buf.push(" }");
    }
}
