//! Class prop value emission for static + dynamic class pairs.

use vize_l2::expr::JsExpr;
use vize_l2::op::{BindOp, DynamicName};

use super::js::{escape_js_string, js_expr_source};
use super::props_object::Piece;
use super::props_value::authored_value_padding;
use super::{EmitCx, EmitError};

/// A lone JSX `class` is emitted as authored: `createVNode` normalizes only
/// truthy non-strings, so falsy values must reach `patchClass` unchanged.
pub(super) fn is_runtime_jsx_class(cx: &EmitCx<'_>, pieces: &[Piece<'_>]) -> bool {
    cx.jsx_class
        && !super::props_object::pieces_have_static_attr(pieces, "class")
        && pieces.iter().any(|piece| match piece {
            Piece::Bind(bind) => matches!(bind.name, Some(DynamicName::Static("class"))),
            _ => false,
        })
}

pub(super) fn emit_class_value(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    bind: &BindOp<'_>,
    js: &JsExpr<'_>,
    skip_normalize: bool,
) -> Result<(), EmitError> {
    if !skip_normalize {
        cx.buf.use_normalize_class();
        cx.buf.push(super::buf::Buf::normalize_class_alias());
        cx.buf.push("(");
    }
    if let Some(static_class) = static_class_piece(pieces) {
        let before = static_class.span.start <= bind.span.start;
        let source = js_expr_source(js);
        let authored =
            authored_value_padding(cx.source, bind, source.as_str(), js.span).unwrap_or(("", ""));
        cx.buf.push("[");
        if before {
            cx.buf.push("\"");
            cx.buf
                .push(escape_js_string(static_class.value.unwrap_or("")).as_str());
            cx.buf.push("\", ");
            emit_authored_js(cx, js, source.as_str(), authored)?;
        } else {
            emit_authored_js(cx, js, source.as_str(), authored)?;
            cx.buf.push(", \"");
            cx.buf
                .push(escape_js_string(static_class.value.unwrap_or("")).as_str());
            cx.buf.push("\"");
        }
        cx.buf.push("]");
    } else {
        let source = js_expr_source(js);
        let authored =
            authored_value_padding(cx.source, bind, source.as_str(), js.span).unwrap_or(("", ""));
        emit_authored_js(cx, js, source.as_str(), authored)?;
    }
    if !skip_normalize {
        cx.buf.push(")");
    }
    Ok(())
}

fn emit_authored_js(
    cx: &mut EmitCx<'_>,
    js: &JsExpr<'_>,
    source: &str,
    authored: (&str, &str),
) -> Result<(), EmitError> {
    if cx.prefixing() {
        let text = cx.prefixed_bind_js(js)?;
        cx.buf.push(text.as_str());
        return Ok(());
    }
    cx.buf.push(authored.0);
    cx.buf.push(source);
    cx.buf.push(authored.1);
    Ok(())
}

fn static_class_piece<'a>(pieces: &'a [Piece<'a>]) -> Option<&'a vize_l2::op::Attribute<'a>> {
    pieces.iter().find_map(|piece| match piece {
        Piece::Attr(attr) if attr.name == "class" => Some(*attr),
        _ => None,
    })
}
