//! Retained-vs-legacy expression shape checks (Davinci P1-7).
//!
//! The boolean shape decisions (`is a handler reference`, `is a function
//! expression`) shared between the legacy string entries in
//! `steps/expression.rs` and the node-aware retained entries here.

use oxc_ast::ast::{ChainElement, Expression};
#[cfg(any(test, feature = "davinci-differential"))]
use oxc_parser::Parser;
#[cfg(any(test, feature = "davinci-differential"))]
use oxc_span::SourceType;
use vize_relief::SimpleExpressionNode;

use super::{is_event_handler_reference_expression, is_function_expression};

/// The handler-reference shape decision, shared by the string and retained
/// entries (and the P1-7 differential comparator).
pub(super) fn is_handler_reference_shape(expr: &Expression<'_>) -> bool {
    match expr {
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

/// The function-expression shape decision, shared like the above.
pub(super) fn is_function_shape(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    )
}

/// Node-aware [`is_event_handler_reference_expression`] (P1-7): reads the
/// retained AST when it still describes the node's bytes and the dialect
/// gate holds; falls back to the legacy string parse otherwise. The legacy
/// entry is a prefix parse (no completeness check), but a retained AST is
/// complete by construction, so on gated nodes both parses see the whole
/// text and the shape decision is the same decision.
pub fn is_event_handler_reference_node(node: &SimpleExpressionNode<'_>) -> bool {
    match crate::retained::retained_whole_expression(node) {
        Some(js) if crate::retained::js_module_compatible(js) => {
            let result = is_handler_reference_shape(js.ast);
            #[cfg(any(test, feature = "davinci-differential"))]
            differential_shape_check(js.raw, result, is_handler_reference_shape);
            result
        }
        _ => is_event_handler_reference_expression(node.content),
    }
}

/// Node-aware [`is_function_expression`] (P1-7); same gating as above.
pub fn is_function_expression_node(node: &SimpleExpressionNode<'_>) -> bool {
    match crate::retained::retained_whole_expression(node) {
        Some(js) if crate::retained::js_module_compatible(js) => {
            let result = is_function_shape(js.ast);
            #[cfg(any(test, feature = "davinci-differential"))]
            differential_shape_check(js.raw, result, is_function_shape);
            result
        }
        _ => is_function_expression(node.content),
    }
}

/// Davinci P1-7 differential lane: a retained shape decision must match the
/// legacy string parse's decision (uncounted arena — lane-only work stays
/// off the production re-parse floor). Divergence panics, never averages.
#[cfg(any(test, feature = "davinci-differential"))]
fn differential_shape_check(raw: &str, retained_result: bool, shape: fn(&Expression<'_>) -> bool) {
    let allocator = oxc_allocator::Allocator::default();
    let legacy = Parser::new(&allocator, raw, SourceType::default().with_module(true))
        .parse_expression()
        .as_ref()
        .map(shape)
        .unwrap_or(false);
    assert_eq!(
        retained_result, legacy,
        "davinci-differential (P1-7): retained shape check diverged from the legacy parse for expression {raw:?}"
    );
    crate::retained::differential::record_shape_comparison();
}
