//! Static style-attribute serialization used by dynamic `:style` merges.

use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind};
use vize_s2::expr::JsExpr;
use vize_s2::op::{Attribute, BindOp};

use super::EmitCx;
use super::buf::Buf;
use super::js::{escape_js_string, js_expr_source};
use super::props_value::BindValue;

pub(super) fn bind_skips_normalize(
    raw_name: &str,
    is_plain_element: bool,
    has_static_style: bool,
    value: &BindValue<'_>,
) -> bool {
    if raw_name != "style" {
        return false;
    }
    if !is_plain_element {
        return static_bind_value(value);
    }
    !has_static_style && static_bind_value(value)
}

fn static_bind_value(value: &BindValue<'_>) -> bool {
    value.js().is_some_and(|js| static_expression(js.ast))
}

fn static_expression(expr: &Expression<'_>) -> bool {
    match unwrap_static_expression(expr) {
        Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::UnaryExpression(unary) => static_expression(&unary.argument),
        Expression::ArrayExpression(array) => array.elements.iter().all(static_array_element),
        Expression::ObjectExpression(object) => object.properties.iter().all(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            !property.computed && static_expression(&property.value)
        }),
        _ => false,
    }
}

fn static_array_element(element: &ArrayExpressionElement<'_>) -> bool {
    match element {
        ArrayExpressionElement::SpreadElement(_) => false,
        ArrayExpressionElement::Elision(_) => true,
        _ => element.as_expression().is_some_and(static_expression),
    }
}

fn unwrap_static_expression<'a>(mut expr: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        match expr {
            Expression::ParenthesizedExpression(paren) => expr = &paren.expression,
            Expression::TSAsExpression(ts_as) => expr = &ts_as.expression,
            Expression::TSNonNullExpression(ts_non_null) => expr = &ts_non_null.expression,
            Expression::TSSatisfiesExpression(ts_satisfies) => expr = &ts_satisfies.expression,
            _ => return expr,
        }
    }
}

pub(super) fn emit_style_value(
    cx: &mut EmitCx<'_>,
    static_style: Option<(&Attribute<'_>, &str)>,
    bind: &BindOp<'_>,
    js: &JsExpr<'_>,
    skip_normalize: bool,
) {
    let wrap = !skip_normalize;
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
            cx.buf.push(js_expr_source(js).as_str());
        } else {
            cx.buf.push(js_expr_source(js).as_str());
            cx.buf.push(", ");
            emit_static_style_object(cx, value);
        }
        cx.buf.push("]");
    } else {
        cx.buf.push(js_expr_source(js).as_str());
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
