//! Restricted primitive proofs from existing AST nodes, never source strings.

use super::super::ScriptParseResult;
use crate::reactivity::ReactiveKind;
use crate::scope::ScopeKind;
use crate::types::builtin_reactive::{BuiltinReactiveType, Primitive};
use oxc_ast::ast::{
    BinaryOperator, BindingPattern, Expression, Statement, VariableDeclarationKind,
    VariableDeclarator,
};
use vize_carton::CompactString;

#[cfg(test)]
mod tests;

pub(super) fn record(
    result: &mut ScriptParseResult,
    declarator: &VariableDeclarator<'_>,
    kind: VariableDeclarationKind,
) {
    if result.scopes.current_scope().kind != ScopeKind::ScriptSetup {
        return;
    }
    let BindingPattern::BindingIdentifier(id) = &declarator.id else {
        return;
    };
    let name = id.name.as_str();
    result.types.clear_builtin_reactive_type(name);
    if kind != VariableDeclarationKind::Const || declarator.type_annotation.is_some() {
        return;
    }
    let Some(Expression::CallExpression(call)) = &declarator.init else {
        return;
    };
    if call.optional || call.type_arguments.is_some() || call.arguments.len() != 1 {
        return;
    }
    let Expression::Identifier(callee) = &call.callee else {
        return;
    };
    let callee = callee.name.as_str();
    if !is_vue_import(result, callee) {
        return;
    }
    let Some(argument) = call.arguments.first().and_then(|arg| arg.as_expression()) else {
        return;
    };
    let (kind, value) = match result.types.definitions().imported_type_export(callee) {
        Some("ref" | "shallowRef") => {
            let Some(value) = primitive_literal(argument) else {
                return;
            };
            (ReactiveKind::Ref, value)
        }
        Some("computed") => {
            let Expression::ArrowFunctionExpression(arrow) = argument else {
                return;
            };
            if arrow.r#async
                || !arrow.expression
                || !arrow.params.items.is_empty()
                || arrow.params.rest.is_some()
                || arrow.return_type.is_some()
                || arrow.type_parameters.is_some()
                || arrow.body.statements.len() != 1
            {
                return;
            }
            let Some(Statement::ExpressionStatement(body)) = arrow.body.statements.first() else {
                return;
            };
            if !numeric_expression(result, &body.expression) {
                return;
            }
            (ReactiveKind::Computed, Primitive::Number)
        }
        _ => return,
    };
    result
        .types
        .record_builtin_reactive_type(BuiltinReactiveType {
            name: CompactString::new(name),
            span: (id.span.start, id.span.end),
            kind,
            value,
        });
}

fn is_vue_import(result: &ScriptParseResult, name: &str) -> bool {
    result
        .import_sources
        .get(name)
        .is_some_and(|source| source.as_str() == "vue")
        && result.binding_spans.get(name).is_some_and(|(start, end)| {
            result
                .import_statements
                .iter()
                .any(|import| *start >= import.start && *end <= import.end)
        })
}

fn primitive_literal(expression: &Expression<'_>) -> Option<Primitive> {
    match expression {
        Expression::BooleanLiteral(_) => Some(Primitive::Boolean),
        Expression::NumericLiteral(_) => Some(Primitive::Number),
        Expression::StringLiteral(_) => Some(Primitive::String),
        _ => None,
    }
}

fn numeric_expression(result: &ScriptParseResult, expression: &Expression<'_>) -> bool {
    match expression {
        Expression::NumericLiteral(_) => true,
        Expression::StaticMemberExpression(member)
            if !member.optional && member.property.name == "value" =>
        {
            owned_primitive_value(result, &member.object) == Some(Primitive::Number)
        }
        Expression::StaticMemberExpression(member)
            if !member.optional && member.property.name == "length" =>
        {
            let Expression::StaticMemberExpression(value) = &member.object else {
                return false;
            };
            !value.optional
                && value.property.name == "value"
                && owned_primitive_value(result, &value.object) == Some(Primitive::String)
        }
        Expression::BinaryExpression(binary)
            if matches!(
                binary.operator,
                BinaryOperator::Addition
                    | BinaryOperator::Subtraction
                    | BinaryOperator::Multiplication
                    | BinaryOperator::Division
                    | BinaryOperator::Remainder
                    | BinaryOperator::Exponential
            ) =>
        {
            numeric_expression(result, &binary.left) && numeric_expression(result, &binary.right)
        }
        _ => false,
    }
}

fn owned_primitive_value(result: &ScriptParseResult, object: &Expression<'_>) -> Option<Primitive> {
    let Expression::Identifier(object) = object else {
        return None;
    };
    let fact = result.types.builtin_reactive_type(object.name.as_str())?;
    (result.binding_spans.get(object.name.as_str()) == Some(&fact.span)).then_some(fact.value)
}
