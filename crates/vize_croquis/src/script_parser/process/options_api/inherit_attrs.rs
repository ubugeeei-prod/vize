use oxc_ast::ast::{Expression, ObjectExpression};

use super::option_expression_property;

pub(super) fn option_bool_property(object: &ObjectExpression<'_>, key_name: &str) -> Option<bool> {
    match option_expression_property(object, key_name)? {
        Expression::BooleanLiteral(literal) => Some(literal.value),
        _ => None,
    }
}
