//! Handler shape decisions (`steps::expression::shape_checks`), retained-AST
//! first with the dialect gate, a complete expression parse otherwise.

use oxc_ast::ast::{ChainElement, Expression};

use super::compat::js_module_compatible;
use super::rewrite::{Retained, with_whole_expression};

pub(super) fn is_handler_reference_shape(expr: &Expression<'_>) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => true,
        Expression::ChainExpression(chain) => matches!(
            chain.expression,
            ChainElement::StaticMemberExpression(_) | ChainElement::ComputedMemberExpression(_)
        ),
        _ => false,
    }
}

pub(super) fn is_function_shape(expr: &Expression<'_>) -> bool {
    matches!(
        expr.get_inner_expression(),
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

/// `is_event_handler_reference_expression` (string entry).
pub(super) fn is_event_handler_reference_expression(content: &str) -> bool {
    with_whole_expression(content, is_handler_reference_shape).unwrap_or(false)
}

/// `is_function_expression` (string entry).
pub(super) fn is_function_expression(content: &str) -> bool {
    with_whole_expression(content, is_function_shape).unwrap_or(false)
}

/// `is_event_handler_reference_node`: the retained decision when the
/// gate holds, the string decision otherwise.
pub(super) fn is_event_handler_reference_node(
    content: &str,
    retained: Option<Retained<'_, '_>>,
) -> bool {
    match retained {
        Some(retained) if js_module_compatible(retained.ast, retained.source) => {
            is_handler_reference_shape(retained.ast)
        }
        _ => is_event_handler_reference_expression(content),
    }
}

/// `is_function_expression_node`, same gating.
pub(super) fn is_function_expression_node(
    content: &str,
    retained: Option<Retained<'_, '_>>,
) -> bool {
    match retained {
        Some(retained) if js_module_compatible(retained.ast, retained.source) => {
            is_function_shape(retained.ast)
        }
        _ => is_function_expression(content),
    }
}
