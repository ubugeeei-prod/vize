//! Class prop value emission for static + dynamic class pairs.

use vize_s2::expr::JsExpr;
use vize_s2::op::BindOp;

use super::EmitCx;
use super::js::{escape_js_string, js_expr_source};
use super::props_object::Piece;
use super::props_value::authored_value_padding;

pub(super) fn emit_class_value(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    bind: &BindOp<'_>,
    js: &JsExpr<'_>,
    skip_normalize: bool,
) {
    if !skip_normalize {
        cx.buf.use_normalize_class();
        cx.buf.push(super::buf::Buf::normalize_class_alias());
        cx.buf.push("(");
    }
    if let Some(static_class) = static_class_piece(pieces).filter(|_| cx.once_depth == 0) {
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
            emit_authored_js(cx, source.as_str(), authored);
        } else {
            emit_authored_js(cx, source.as_str(), authored);
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
        emit_authored_js(cx, source.as_str(), authored);
    }
    if !skip_normalize {
        cx.buf.push(")");
    }
}

fn emit_authored_js(cx: &mut EmitCx<'_>, source: &str, authored: (&str, &str)) {
    cx.buf.push(authored.0);
    cx.buf.push(source);
    cx.buf.push(authored.1);
}

fn static_class_piece<'a>(pieces: &'a [Piece<'a>]) -> Option<&'a vize_s2::op::Attribute<'a>> {
    pieces.iter().find_map(|piece| match piece {
        Piece::Attr(attr) if attr.name == "class" => Some(*attr),
        _ => None,
    })
}
