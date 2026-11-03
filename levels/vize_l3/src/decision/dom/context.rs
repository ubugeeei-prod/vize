//! Scoped context semantics from the real complete retained-AST resolver.

use super::DomExpressionFacts;
use vize_l2::{
    expr::JsExpr,
    resolution::{BindingId, ResolutionTable},
};

/// Explicit context identities shared with the consumer's checked access policy.
///
/// This is a bounded per-file declaration, not the unfinished full script binder.
/// Every supplied table is constructed by L2's actual complete resolver. A table
/// for another AST/location/coordinate bridge cannot classify this expression.
/// Constants remain the DOM producer's literal-AST decision, never this registry.
pub struct ContextOnly<'facts, 'arena> {
    expressions: &'facts [ResolutionTable<'arena>],
    context_bindings: &'facts [BindingId],
}

impl<'facts, 'arena> ContextOnly<'facts, 'arena> {
    #[must_use]
    pub const fn new(
        expressions: &'facts [ResolutionTable<'arena>],
        context_bindings: &'facts [BindingId],
    ) -> Self {
        Self {
            expressions,
            context_bindings,
        }
    }
}

impl DomExpressionFacts for ContextOnly<'_, '_> {
    fn is_context_only(&self, expression: &JsExpr<'_>) -> bool {
        self.expressions.iter().any(|table| {
            let retained = table.expression();
            core::ptr::eq(retained.ast, expression.ast)
                && retained.source == expression.source
                && retained.span == expression.span
                && match (retained.coordinates, expression.coordinates) {
                    (None, None) => true,
                    (Some(left), Some(right)) => core::ptr::eq(left, right),
                    _ => false,
                }
                && !table.occurrences().is_empty()
                && table
                    .occurrences()
                    .iter()
                    .all(|occurrence| self.context_bindings.contains(&occurrence.binding))
        })
    }
}
