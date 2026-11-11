//! Call-path and options-object helpers for `vue/no-restricted-call-after-await`.

use oxc_ast::ast::{ExportDefaultDeclarationKind, Expression, ObjectExpression, PropertyKey};

pub(super) fn callee_path<'a>(callee: &'a Expression<'a>) -> Option<(&'a str, Vec<&'a str>)> {
    let mut members = Vec::new();
    let mut current = callee.get_inner_expression();
    loop {
        match current {
            Expression::Identifier(identifier) => {
                members.reverse();
                return Some((identifier.name.as_str(), members));
            }
            Expression::StaticMemberExpression(member) => {
                members.push(member.property.name.as_str());
                current = member.object.get_inner_expression();
            }
            _ => return None,
        }
    }
}

pub(super) fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

pub(super) fn property_key<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

pub(super) fn object_of_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            object_of(&expression.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            object_of(&expression.expression)
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(expression) => {
            object_of(&expression.expression)
        }
        _ => None,
    }
}

fn object_of<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(expression) => object_of(&expression.expression),
        Expression::TSAsExpression(expression) => object_of(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => object_of(&expression.expression),
        _ => None,
    }
}
