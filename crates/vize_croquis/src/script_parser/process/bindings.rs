//! Binding pattern helpers and expression classification.
//!
//! Provides utility functions for working with binding patterns
//! and classifying expressions as literals or functions.

use oxc_ast::ast::{BindingPattern, Expression, VariableDeclarationKind};
use vize_relief::BindingType;

use super::super::extract::get_binding_type_from_kind;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Get binding name from binding pattern kind
pub(in crate::script_parser) fn get_binding_pattern_name(
    pattern: &BindingPattern<'_>,
) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.to_compact_string()),
        BindingPattern::AssignmentPattern(assign) => get_binding_pattern_name(&assign.left),
        _ => None,
    }
}

/// Infer binding type for destructured variables, matching the non-destructured inference logic.
/// For `const { x } = useComposable()`, returns SetupMaybeRef since the properties may be refs.
pub(in crate::script_parser) fn infer_destructure_binding_type(
    kind: VariableDeclarationKind,
    init: Option<&Expression<'_>>,
) -> BindingType {
    if kind == VariableDeclarationKind::Const {
        if let Some(init) = init {
            if is_function_expression(init) {
                BindingType::SetupConst
            } else {
                BindingType::SetupMaybeRef
            }
        } else {
            BindingType::SetupConst
        }
    } else {
        get_binding_type_from_kind(kind)
    }
}

/// Check if an expression is a literal value (number, string, boolean, null, template literal
/// without expressions, or unary minus on a numeric literal)
pub(in crate::script_parser) fn is_literal_expression(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_) => true,
        Expression::TemplateLiteral(tpl) => tpl.expressions.is_empty(),
        Expression::UnaryExpression(unary) => {
            unary.operator == oxc_ast::ast::UnaryOperator::UnaryNegation
                && matches!(unary.argument, Expression::NumericLiteral(_))
        }
        _ => false,
    }
}

/// Check if an expression is a function expression (arrow function or function expression)
pub(in crate::script_parser) fn is_function_expression(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}
