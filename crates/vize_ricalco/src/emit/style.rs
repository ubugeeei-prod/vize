//! Static style-attribute serialization used by dynamic `:style` merges.

use oxc_ast::ast::Expression;
use vize_disegno::expr::JsExpr;
use vize_disegno::op::{Attribute, BindOp};

use super::EmitCx;
use super::buf::Buf;
use super::js::escape_js_string;

pub(super) fn emit_style_value(
    cx: &mut EmitCx<'_>,
    static_style: Option<(&Attribute<'_>, &str)>,
    bind: &BindOp<'_>,
    js: &JsExpr<'_>,
    skip_normalize: bool,
) {
    let bare_object_literal =
        static_style.is_none() && matches!(js.ast, Expression::ObjectExpression(_));
    let wrap = !skip_normalize && !bare_object_literal;
    if wrap {
        cx.buf.use_normalize_style();
        cx.buf.push(Buf::normalize_style_alias());
        cx.buf.push("(");
    }
    if let Some((attr, value)) = static_style {
        let before = attr.span.start <= bind.span.start;
        cx.buf.push("[");
        if before {
            emit_static_style_object(cx, value);
            cx.buf.push(", ");
            cx.buf.push(js.source);
        } else {
            cx.buf.push(js.source);
            cx.buf.push(", ");
            emit_static_style_object(cx, value);
        }
        cx.buf.push("]");
    } else {
        cx.buf.push(js.source);
    }
    if wrap {
        cx.buf.push(")");
    }
}

fn emit_static_style_object(cx: &mut EmitCx<'_>, value: &str) {
    cx.buf.push("{");
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut emitted = 0usize;
    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            ';' if depth == 0 => {
                emit_declaration(cx, &value[start..i], &mut emitted);
                start = i + ch.len_utf8();
            }
            _ => {}
        }
    }
    emit_declaration(cx, &value[start..], &mut emitted);
    cx.buf.push("}");
}

fn emit_declaration(cx: &mut EmitCx<'_>, declaration: &str, emitted: &mut usize) {
    let declaration = declaration.trim();
    if declaration.is_empty() {
        return;
    }
    let Some((key, value)) = declaration.split_once(':') else {
        return;
    };
    if *emitted > 0 {
        cx.buf.push(",");
    }
    *emitted += 1;
    cx.buf.push("\"");
    cx.buf.push(escape_js_string(key.trim()).as_str());
    cx.buf.push("\":\"");
    cx.buf.push(escape_js_string(value.trim()).as_str());
    cx.buf.push("\"");
}
