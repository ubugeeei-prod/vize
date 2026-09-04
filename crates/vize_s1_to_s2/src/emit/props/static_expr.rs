use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BinaryOperator, Expression, ObjectPropertyKind,
};
use vize_s2::op::BindOp;

use super::bind_value;

pub(super) fn bind_value_uses_legacy_patchless_runtime_expr(bind: &BindOp<'_>) -> bool {
    match bind_value(bind) {
        Ok(value) => value
            .js()
            .is_some_and(|js| is_legacy_patchless_runtime_expr(js.ast)),
        Err(_) => false,
    }
}

pub(super) fn is_static_bound_expr(expr: &Expression<'_>) -> bool {
    match unwrap_expr(expr) {
        Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::UnaryExpression(unary) => is_static_bound_expr(&unary.argument),
        Expression::ArrayExpression(array) => array.elements.iter().all(static_array_element),
        Expression::ObjectExpression(object) => object.properties.iter().all(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            !property.computed && is_static_bound_expr(&property.value)
        }),
        Expression::CallExpression(call)
            if matches!(
                &call.callee,
                Expression::Identifier(ident)
                    if matches!(ident.name.as_str(), "_normalizeClass" | "_normalizeStyle")
            ) =>
        {
            call.arguments.iter().all(static_argument)
        }
        _ => false,
    }
}

fn is_legacy_patchless_runtime_expr(expr: &Expression<'_>) -> bool {
    is_legacy_in_conditional(expr)
}

fn is_legacy_in_conditional(expr: &Expression<'_>) -> bool {
    let Expression::ConditionalExpression(conditional) = expr else {
        return false;
    };
    legacy_test_starts_with_in(&conditional.test)
}

fn legacy_test_starts_with_in(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::BinaryExpression(binary) => binary.operator == BinaryOperator::In,
        Expression::LogicalExpression(logical) => {
            matches!(
                &logical.left,
                Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::In
            )
        }
        _ => false,
    }
}

fn static_argument(argument: &Argument<'_>) -> bool {
    match argument {
        Argument::SpreadElement(_) => false,
        _ => argument.as_expression().is_some_and(is_static_bound_expr),
    }
}

fn static_array_element(element: &ArrayExpressionElement<'_>) -> bool {
    match element {
        ArrayExpressionElement::SpreadElement(_) => false,
        ArrayExpressionElement::Elision(_) => true,
        _ => element.as_expression().is_some_and(is_static_bound_expr),
    }
}

fn unwrap_expr<'a>(mut expr: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        match expr {
            Expression::ParenthesizedExpression(paren) => expr = &paren.expression,
            _ => return expr,
        }
    }
}
