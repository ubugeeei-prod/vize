//! Class prop value emission for static + dynamic class pairs.

use vize_s2::expr::JsExpr;
use vize_s2::op::BindOp;

use super::EmitCx;
use super::js::escape_js_string;
use super::props_object::Piece;

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
    if let Some(static_class) = static_class_piece(pieces) {
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

fn static_class_piece<'a>(pieces: &'a [Piece<'a>]) -> Option<&'a vize_s2::op::Attribute<'a>> {
    pieces.iter().find_map(|piece| match piece {
        Piece::Attr(attr) if attr.name == "class" => Some(*attr),
        _ => None,
    })
}
