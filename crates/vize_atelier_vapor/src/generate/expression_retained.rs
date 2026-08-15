//! Retained-AST fast path for the vapor expression resolver (Davinci P1-7).
//!
//! The legacy resolver (`expression.rs`) parses `(expr)` with the TS module
//! dialect into a throwaway arena. The retained parse (P1-5) is TS with the
//! unambiguous module goal — the only dialect delta is the module goal, so
//! the gate reuses `vize_atelier_core::retained::js_module_compatible`,
//! which is deliberately over-conservative here (it also rejects TS-syntax
//! expressions; those stay on the vapor fallback parse — recorded in
//! `plan/phase-1.md` P1-7).
//!
//! Span note: the legacy wrapper shifts identifier spans by one, undone via
//! `apply_rewrites(.., offset: 1)`; the retained AST is content-relative, so
//! the same rewrites apply at the leading-whitespace offset (the resolver
//! rewrites the *trimmed* text; tokens never live in the surrounding
//! whitespace, so the shift is exact).

use oxc_ast_visit::Visit;
use vize_atelier_core::SimpleExpressionNode;
use vize_carton::{String, ToCompactString};

use super::context::GenerateContext;
use super::expression::{
    ExpressionRewriteCollector, apply_rewrites, is_literal_expression, is_simple_path_expression,
    resolve_with_oxc,
};

/// Node-aware `resolve_expression`: byte-identical to
/// `expression::resolve_expression(ctx, node.content.as_str())`, minus the
/// throwaway parse when the retained AST applies.
pub(super) fn resolve_expression_node(
    ctx: &GenerateContext<'_>,
    node: &SimpleExpressionNode<'_>,
) -> String {
    let expr = node.content.as_str();
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return String::default();
    }

    if is_literal_expression(trimmed) {
        return trimmed.to_compact_string();
    }

    if is_simple_path_expression(trimmed) {
        return ctx.resolve_simple_reference(trimmed);
    }

    if let Some(js) = vize_atelier_core::retained::retained_whole_expression(node)
        && vize_atelier_core::retained::js_module_compatible(js)
    {
        // Retained spans are content-relative; the resolver rewrites the
        // trimmed text, so shift by the leading whitespace (tokens never
        // live in the surrounding whitespace).
        let lead = expr.len() - expr.trim_start().len();
        let mut collector = ExpressionRewriteCollector::new(ctx);
        collector.visit_expression(js.ast);
        let resolved = apply_rewrites(trimmed, collector.rewrites, lead);
        #[cfg(any(test, feature = "davinci-differential"))]
        assert_resolve_agrees(ctx, trimmed, &resolved);
        return resolved;
    }

    if let Some(resolved) = resolve_with_oxc(ctx, trimmed) {
        return resolved;
    }

    ctx.resolve_complex_expression_fallback(trimmed)
}

/// Davinci P1-7 differential lane: the retained walk must reproduce the
/// legacy `resolve_with_oxc` expression branch byte-for-byte. That branch
/// counts through the P0-3 probe, so the dual-run replicates it in an
/// uncounted arena instead — lane-only work stays off the production
/// re-parse floor. Divergence panics, never averages.
#[cfg(any(test, feature = "davinci-differential"))]
fn assert_resolve_agrees(ctx: &GenerateContext<'_>, expr: &str, retained: &str) {
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = oxc_allocator::Allocator::default();
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);
    let mut wrapped = String::with_capacity(expr.len() + 2);
    wrapped.push('(');
    wrapped.push_str(expr);
    wrapped.push(')');
    let parsed = Parser::new(&allocator, wrapped.as_str(), source_type)
        .parse_expression()
        .unwrap_or_else(|_| {
            panic!(
                "davinci-differential (P1-7): the dialect gate admitted an \
                 expression the legacy vapor parse rejects: {expr:?}"
            )
        });
    let mut collector = ExpressionRewriteCollector::new(ctx);
    collector.visit_expression(&parsed);
    let legacy = apply_rewrites(expr, collector.rewrites, 1);
    assert_eq!(
        retained,
        legacy.as_str(),
        "davinci-differential (P1-7): retained vapor resolve diverged from the legacy re-parse for expression {expr:?}"
    );
    vize_atelier_core::retained::differential::record_vapor_resolve_comparison();
}
