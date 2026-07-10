//! Binding pattern helpers and expression classification.
//!
//! Provides utility functions for working with binding patterns
//! and classifying expressions as literals or functions.

use oxc_ast::ast::{BindingPattern, Expression, VariableDeclarationKind};
use vize_carton::{CompactString, String, ToCompactString};
use vize_relief::BindingType;

use crate::croquis::BindingMetadata;

use super::super::extract::get_binding_type_from_kind;

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

pub(in crate::script_parser) fn add_binding_pattern_names(
    bindings: &mut BindingMetadata,
    pattern: &BindingPattern<'_>,
    binding_type: BindingType,
) {
    for_each_binding_pattern_name(pattern, &mut |name| bindings.add(name, binding_type));
}

pub(in crate::script_parser) fn push_binding_pattern_names(
    pattern: &BindingPattern<'_>,
    target: &mut Vec<CompactString>,
) {
    for_each_binding_pattern_name(pattern, &mut |name| target.push(CompactString::new(name)));
}

fn for_each_binding_pattern_name(pattern: &BindingPattern<'_>, visit: &mut impl FnMut(&str)) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => visit(id.name.as_str()),
        BindingPattern::ObjectPattern(object) => {
            for property in object.properties.iter() {
                for_each_binding_pattern_name(&property.value, visit);
            }
            if let Some(rest) = &object.rest {
                for_each_binding_pattern_name(&rest.argument, visit);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                for_each_binding_pattern_name(element, visit);
            }
            if let Some(rest) = &array.rest {
                for_each_binding_pattern_name(&rest.argument, visit);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            for_each_binding_pattern_name(&assign.left, visit);
        }
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
