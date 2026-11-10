//! Small semantic predicates shared by the core-rule visitor.

use oxc_ast::ast::{BinaryExpression, Expression, NumericLiteral, PropertyKey};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

pub(super) fn is_underscored_identifier(name: &str) -> bool {
    let body = name.trim_matches('_');
    body.contains('_') && body != body.to_uppercase()
}

pub(super) fn is_numeric(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::NumericLiteral(_) => true,
        Expression::CallExpression(call) => matches!(
            call.callee.get_inner_expression(),
            Expression::Identifier(identifier)
                if matches!(identifier.name.as_str(), "Number" | "parseInt" | "parseFloat")
        ),
        _ => false,
    }
}

pub(super) fn is_identifier_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
}

pub(super) fn can_unquote_property(name: &str) -> bool {
    is_identifier_name(name) || is_canonical_integer_property(name)
}

fn is_canonical_integer_property(name: &str) -> bool {
    const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

    if name == "0" {
        return true;
    }
    name.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_digit() && *byte != b'0')
        && name.bytes().all(|byte| byte.is_ascii_digit())
        && name
            .parse::<u64>()
            .is_ok_and(|value| value <= MAX_SAFE_INTEGER)
}

pub(super) fn is_empty_string_like(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::StringLiteral(literal) => literal.value.is_empty(),
        Expression::TemplateLiteral(template) => {
            template.expressions.is_empty()
                && template.quasis.first().is_some_and(|quasi| {
                    quasi
                        .value
                        .cooked
                        .as_ref()
                        .is_some_and(|text| text.is_empty())
                })
        }
        _ => false,
    }
}

pub(super) fn is_string_like(expression: &Expression<'_>) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
    )
}

pub(super) fn is_known_string(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::CallExpression(call) => matches!(
            call.callee.get_inner_expression(),
            Expression::Identifier(identifier) if identifier.name == "String"
        ),
        _ => false,
    }
}

pub(super) fn is_implicit_numeric_coercion(binary: &BinaryExpression<'_>) -> bool {
    match binary.operator {
        BinaryOperator::Multiplication => {
            is_number(&binary.left, 1.0) && !is_numeric(&binary.right)
                || is_number(&binary.right, 1.0) && !is_numeric(&binary.left)
        }
        BinaryOperator::Subtraction => is_number(&binary.right, 0.0) && !is_numeric(&binary.left),
        _ => false,
    }
}

fn is_number(expression: &Expression<'_>, expected: f64) -> bool {
    matches!(expression.get_inner_expression(), Expression::NumericLiteral(literal) if literal.value == expected)
}

pub(super) fn is_index_of_call(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return false;
    };
    call.callee
        .get_inner_expression()
        .as_member_expression()
        .and_then(|member| member.static_property_name())
        .is_some_and(|property| property == "indexOf")
}

pub(super) fn leftmost_addition_operand<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression.get_inner_expression() {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            leftmost_addition_operand(&binary.left)
        }
        expression => expression,
    }
}

pub(super) fn rightmost_addition_operand<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression.get_inner_expression() {
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            rightmost_addition_operand(&binary.right)
        }
        expression => expression,
    }
}

pub(super) fn addition_has_string_and_non_string(binary: &BinaryExpression<'_>) -> bool {
    fn collect(expression: &Expression<'_>, has_string: &mut bool, has_other: &mut bool) {
        match expression.get_inner_expression() {
            Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
                collect(&binary.left, has_string, has_other);
                collect(&binary.right, has_string, has_other);
            }
            expression if is_string_like(expression) => *has_string = true,
            _ => *has_other = true,
        }
    }
    let (mut has_string, mut has_other) = (false, false);
    collect(&binary.left, &mut has_string, &mut has_other);
    collect(&binary.right, &mut has_string, &mut has_other);
    has_string && has_other
}

pub(super) fn is_constant(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::ArrayExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.iter().all(is_constant),
        Expression::UnaryExpression(unary) => is_constant(&unary.argument),
        Expression::LogicalExpression(logical) => match logical.operator {
            oxc_syntax::operator::LogicalOperator::Or => {
                is_constant(&logical.left) && is_constant(&logical.right)
                    || is_truthy_constant(&logical.left)
                    || is_truthy_constant(&logical.right)
            }
            oxc_syntax::operator::LogicalOperator::And => {
                is_constant(&logical.left) && is_constant(&logical.right)
                    || is_falsy_constant(&logical.left)
                    || is_falsy_constant(&logical.right)
            }
            oxc_syntax::operator::LogicalOperator::Coalesce => {
                is_constant(&logical.left)
                    && (!is_nullish_constant(&logical.left) || is_constant(&logical.right))
            }
        },
        _ => false,
    }
}

fn is_truthy_constant(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::BooleanLiteral(value) => value.value,
        Expression::NumericLiteral(value) => value.value != 0.0 && !value.value.is_nan(),
        Expression::StringLiteral(value) => !value.value.is_empty(),
        Expression::TemplateLiteral(template) => {
            template.expressions.is_empty()
                && template.quasis.first().is_some_and(|quasi| {
                    quasi
                        .value
                        .cooked
                        .as_ref()
                        .is_some_and(|text| !text.is_empty())
                })
        }
        Expression::ArrayExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_) => true,
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            is_falsy_constant(&unary.argument)
        }
        _ => false,
    }
}

fn is_falsy_constant(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::BooleanLiteral(value) => !value.value,
        Expression::NullLiteral(_) => true,
        Expression::NumericLiteral(value) => value.value == 0.0 || value.value.is_nan(),
        Expression::StringLiteral(value) => value.value.is_empty(),
        Expression::TemplateLiteral(template) => {
            template.expressions.is_empty()
                && template.quasis.first().is_some_and(|quasi| {
                    quasi
                        .value
                        .cooked
                        .as_ref()
                        .is_some_and(|text| text.is_empty())
                })
        }
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            is_truthy_constant(&unary.argument)
        }
        _ => false,
    }
}

fn is_nullish_constant(expression: &Expression<'_>) -> bool {
    matches!(
        expression.get_inner_expression(),
        Expression::NullLiteral(_)
    )
}

pub(super) fn is_negated(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary) => matches!(
            binary.operator,
            BinaryOperator::Inequality | BinaryOperator::StrictInequality
        ),
        _ => false,
    }
}

pub(super) fn numeric_literal_may_lose_precision(literal: &NumericLiteral<'_>) -> bool {
    let Some(raw) = literal.raw.as_ref().map(|raw| raw.as_str()) else {
        return false;
    };
    if !literal.value.is_finite() {
        return true;
    }
    let normalized = raw.replace('_', "");
    if let Some(hex) = normalized
        .strip_prefix("0x")
        .or_else(|| normalized.strip_prefix("0X"))
    {
        return u128::from_str_radix(hex, 16)
            .ok()
            .is_some_and(|value| literal.value as u128 != value);
    }
    if normalized.bytes().all(|byte| byte.is_ascii_digit()) {
        return normalized
            .parse::<u128>()
            .ok()
            .is_some_and(|value| literal.value as u128 != value);
    }
    false
}

pub(super) fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

pub(super) fn is_irregular_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{0085}' | '\u{1680}' | '\u{180E}' | '\u{2000}'
            ..='\u{200B}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}
