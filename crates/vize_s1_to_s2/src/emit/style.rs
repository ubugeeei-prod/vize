//! Static style-attribute serialization used by dynamic `:style` merges.

use oxc_ast::ast::{
    ArrayExpressionElement, Expression, IdentifierReference, ObjectPropertyKind, PropertyKey,
};
use oxc_ast_visit::Visit;
use vize_s2::expr::JsExpr;
use vize_s2::op::{Attribute, BindOp};

use super::buf::Buf;
use super::js::{escape_js_string, js_expr_source};
use super::props_value::{BindValue, authored_value_padding};
use super::{EmitCx, EmitError};

pub(super) fn bind_skips_normalize(
    raw_name: &str,
    is_plain_element: bool,
    has_static_style: bool,
    value: &BindValue<'_>,
) -> bool {
    if raw_name != "style" {
        return false;
    }
    let static_value = static_bind_value(value) || legacy_static_style_object(value);
    if !is_plain_element {
        return static_value;
    }
    !has_static_style && static_value
}

fn static_bind_value(value: &BindValue<'_>) -> bool {
    value.js().is_some_and(|js| static_expression(js.ast))
}

pub(super) fn legacy_static_style_object(value: &BindValue<'_>) -> bool {
    value
        .js()
        .is_some_and(|js| legacy_static_style_object_expr(js.ast, js.source))
}

fn legacy_static_style_object_expr(expr: &Expression<'_>, source: &str) -> bool {
    let Expression::ObjectExpression(object) = unwrap_static_expression(expr) else {
        return false;
    };
    object.properties.iter().all(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        legacy_static_style_key(&property.key, property.computed)
            && legacy_static_style_value(&property.value, source)
    })
}

fn legacy_static_style_value(expr: &Expression<'_>, source: &str) -> bool {
    static_expression(expr) || legacy_global_constant_expr(expr, source)
}

fn legacy_global_constant_expr(expr: &Expression<'_>, source: &str) -> bool {
    if source.contains("_ctx.")
        || source.contains("$setup.")
        || source.contains("__props.")
        || source.contains("$props.")
    {
        return false;
    }
    let mut walk = LegacyGlobalConstWalk { dynamic: false };
    walk.visit_expression(expr);
    !walk.dynamic
}

struct LegacyGlobalConstWalk {
    dynamic: bool,
}

impl<'a> Visit<'a> for LegacyGlobalConstWalk {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if !super::props_bind::is_global_key_name(ident.name.as_str()) {
            self.dynamic = true;
        }
    }
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
            legacy_static_style_key(&property.key, property.computed)
                && static_expression(&property.value)
        }),
        _ => false,
    }
}

fn legacy_static_style_key(key: &PropertyKey<'_>, computed: bool) -> bool {
    if !computed {
        return true;
    }
    match key {
        PropertyKey::StringLiteral(_) => true,
        PropertyKey::TemplateLiteral(template) => template.expressions.is_empty(),
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
) -> Result<(), EmitError> {
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
            emit_bound_style_source(cx, bind, js)?;
        } else {
            emit_bound_style_source(cx, bind, js)?;
            cx.buf.push(", ");
            emit_static_style_object(cx, value);
        }
        cx.buf.push("]");
    } else {
        emit_bound_style_source(cx, bind, js)?;
    }
    if wrap {
        cx.buf.push(")");
    }
    Ok(())
}

fn emit_bound_style_source(
    cx: &mut EmitCx<'_>,
    bind: &BindOp<'_>,
    js: &JsExpr<'_>,
) -> Result<(), EmitError> {
    if cx.prefixing() {
        let text = cx.prefixed_bind_js(js)?;
        cx.buf.push(text.as_str());
        return Ok(());
    }
    let source = js_expr_source(js);
    if let Some((leading, trailing)) =
        authored_value_padding(cx.source, bind, source.as_str(), js.span)
    {
        cx.buf.push(leading);
        cx.buf.push(source.as_str());
        cx.buf.push(trailing);
    } else {
        cx.buf.push(source.as_str());
    }
    Ok(())
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
