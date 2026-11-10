//! Small semantic predicates shared by the core-rule visitor.

use oxc_ast::ast::{BinaryExpression, ChainElement, Expression, PropertyKey};
use oxc_syntax::number::ToJsString;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};
use vize_l0::FxHashSet;

pub(super) fn is_underscored_identifier(name: &str) -> bool {
    let body = name.trim_matches('_');
    body.contains('_') && body != body.to_uppercase()
}

pub(super) fn is_numeric(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::NumericLiteral(_) => true,
        Expression::CallExpression(call) => matches!(
            call.callee.without_parentheses(),
            Expression::Identifier(identifier)
                if matches!(identifier.name.as_str(), "Number" | "parseInt" | "parseFloat")
        ),
        _ => false,
    }
}

fn is_identifier_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
}

/// A literal key that `vue/dot-notation` would rather see as `.key`.
pub(super) fn dot_notation_key<'a>(key: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    let key = key.without_parentheses();
    let name = match key {
        Expression::StringLiteral(literal) => literal.value.as_str(),
        Expression::BooleanLiteral(literal) if literal.value => "true",
        Expression::BooleanLiteral(_) => "false",
        Expression::NullLiteral(_) => "null",
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
            .quasis
            .first()
            .and_then(|quasi| quasi.value.cooked.as_ref())
            .map(|cooked| cooked.as_str())?,
        _ => return None,
    };
    is_identifier_name(name).then_some(key)
}

pub(super) fn can_unquote_property(name: &str) -> bool {
    oxc_syntax::identifier::is_identifier_name(name) || is_canonical_number_property(name)
}

/// One decimal number token that prints back as written (`String(Number(key)) === key`).
fn is_canonical_number_property(name: &str) -> bool {
    let (mantissa, exponent) = match name.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (name, None),
    };
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    let token = !(integer.is_empty() && fraction.is_empty())
        && digits(integer)
        && digits(fraction)
        && exponent.is_none_or(|exponent| {
            let unsigned = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
            !unsigned.is_empty() && digits(unsigned)
        });
    token
        && name
            .parse::<f64>()
            .is_ok_and(|value| value.to_js_string() == name)
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

/// `a * 1` or `a - 0`; a chain such as `2 * a * 1` has no plain non-numeric operand.
pub(super) fn is_implicit_numeric_coercion(binary: &BinaryExpression<'_>) -> bool {
    let plain_non_numeric = |operand: &Expression<'_>| {
        !matches!(
            operand.without_parentheses(),
            Expression::BinaryExpression(_)
        ) && !is_numeric(operand)
    };
    match binary.operator {
        BinaryOperator::Multiplication => {
            (is_number(&binary.left, 1.0) || is_number(&binary.right, 1.0))
                && (plain_non_numeric(&binary.right) || plain_non_numeric(&binary.left))
        }
        BinaryOperator::Subtraction => is_number(&binary.right, 0.0) && !is_numeric(&binary.left),
        _ => false,
    }
}

/// `a * 1` written as the dividend of `a * 1 / b` reads as `a * (1 / b)`.
pub(super) fn multiplies_by_fraction_of_one<'a>(
    binary: &'a BinaryExpression<'a>,
) -> Option<&'a BinaryExpression<'a>> {
    if binary.operator != BinaryOperator::Division {
        return None;
    }
    match &binary.left {
        Expression::BinaryExpression(left)
            if left.operator == BinaryOperator::Multiplication && is_number(&left.right, 1.0) =>
        {
            Some(left)
        }
        _ => None,
    }
}

fn is_number(expression: &Expression<'_>, expected: f64) -> bool {
    matches!(expression.without_parentheses(), Expression::NumericLiteral(literal) if literal.value == expected)
}

pub(super) fn is_index_of_call(expression: &Expression<'_>) -> bool {
    let call = match expression.without_parentheses() {
        Expression::CallExpression(call) => call,
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => call,
            _ => return false,
        },
        _ => return false,
    };
    let member = match call.callee.without_parentheses() {
        Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
        callee => callee.as_member_expression(),
    };
    member
        .and_then(|member| member.static_property_name())
        .is_some_and(|property| matches!(property, "indexOf" | "lastIndexOf"))
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

pub(super) fn collect_nested_addition_spans(
    expression: &Expression<'_>,
    spans: &mut FxHashSet<(u32, u32)>,
) {
    let Expression::BinaryExpression(binary) = expression.get_inner_expression() else {
        return;
    };
    if binary.operator != BinaryOperator::Addition {
        return;
    }
    spans.insert((binary.span.start, binary.span.end));
    collect_nested_addition_spans(&binary.left, spans);
    collect_nested_addition_spans(&binary.right, spans);
}
