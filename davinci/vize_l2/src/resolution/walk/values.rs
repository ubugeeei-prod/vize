//! Scalar payloads from the already visited original expression.

use super::{Expression, SyntaxKind};

pub(super) fn syntax_value<'a>(expression: &Expression<'a>) -> SyntaxKind<'a> {
    match expression {
        Expression::Identifier(value) => SyntaxKind::IdentifierExpression(value.name.as_str()),
        Expression::NumericLiteral(value) => SyntaxKind::NumberExpression(value.value.to_bits()),
        Expression::StringLiteral(value) => SyntaxKind::StringExpression(value.value.as_str()),
        Expression::BooleanLiteral(value) => SyntaxKind::BooleanExpression(value.value),
        Expression::NullLiteral(_) => SyntaxKind::NullExpression,
        _ => SyntaxKind::Expression,
    }
}
