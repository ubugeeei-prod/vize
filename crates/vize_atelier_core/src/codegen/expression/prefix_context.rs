//! Context-aware identifier prefixing for codegen (split from
//! `helpers.rs`; the P1-7 retained-AST entry lives here).
//!
//! The legacy entry parses `(content)` (identifier spans shifted by the
//! wrapper, hence `offset: 1`); the retained entry walks the parse-once AST
//! with content-relative spans (`offset: 0`). The visitor and the rewrite
//! application are shared, so the two paths differ only in where the AST
//! comes from.

use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_relief::{ExpressionScope, SimpleExpressionNode};
use vize_s0::FxHashSet;
use vize_s0::String;
use vize_s0::ToCompactString;

use super::super::context::CodegenContext;
use super::helpers::rewrite_props_aliases;
use super::prefix_visitor::{IdentifierVisitor, apply_rewrites};

/// Walk one parsed expression and apply the collected rewrites to `content`.
/// `offset` is the span shift of the parse text relative to `content` (1 for
/// the legacy `(content)` wrapper, 0 for the retained AST).
fn prefix_via_expr(
    expr: &oxc_ast::ast::Expression<'_>,
    offset: u32,
    content: &str,
    ctx: &CodegenContext,
    implicit_event: bool,
) -> String {
    let mut rewrites: Vec<(usize, usize, String)> = Vec::new();
    let mut assignment_targets: FxHashSet<usize> = FxHashSet::default();

    let mut visitor = IdentifierVisitor {
        rewrites: &mut rewrites,
        local_scopes: vec![FxHashSet::default()],
        assignment_targets: &mut assignment_targets,
        ctx,
        offset,
    };
    if implicit_event {
        visitor.add_local("$event");
    }
    visitor.visit_expression(expr);

    rewrite_props_aliases(apply_rewrites(content, rewrites), ctx)
}

pub(super) fn prefix_identifiers_in_scope(
    content: &str,
    ctx: &CodegenContext,
    implicit_event: bool,
) -> String {
    let allocator = crate::expr_parse_probe::parse_arena();
    let source_type = SourceType::default().with_module(true);

    // First try: wrap in parentheses to parse as a single expression
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    let parser = Parser::new(&allocator, &wrapped, source_type);
    let parse_result = parser.parse_expression();

    match parse_result {
        Ok(expr) => prefix_via_expr(&expr, 1, content, ctx, implicit_event),
        Err(_) => {
            // Expression parsing failed -- try parsing as a program
            let allocator2 = crate::expr_parse_probe::parse_arena();
            let parser2 = Parser::new(&allocator2, content, source_type);
            let parse_result2 = parser2.parse();
            if parse_result2.diagnostics.is_empty() {
                let mut rewrites: Vec<(usize, usize, String)> = Vec::new();
                let mut assignment_targets: FxHashSet<usize> = FxHashSet::default();

                let mut visitor = IdentifierVisitor {
                    rewrites: &mut rewrites,
                    local_scopes: vec![FxHashSet::default()],
                    assignment_targets: &mut assignment_targets,
                    ctx,
                    offset: 0,
                };
                if implicit_event {
                    visitor.add_local("$event");
                }
                visitor.visit_program(&parse_result2.program);

                rewrite_props_aliases(apply_rewrites(content, rewrites), ctx)
            } else {
                content.to_compact_string()
            }
        }
    }
}

/// Node-aware [`prefix_identifiers_with_context`] (P1-7): reads the retained
/// AST when it still describes the node's bytes and the dialect gate holds;
/// the legacy wrapped re-parse otherwise.
pub(crate) fn prefix_identifiers_with_context_node(
    node: &SimpleExpressionNode<'_>,
    ctx: &CodegenContext,
) -> String {
    prefix_node_in_scope(node, ctx, false)
}

pub(super) fn prefix_node_in_scope(
    node: &SimpleExpressionNode<'_>,
    ctx: &CodegenContext,
    implicit_event: bool,
) -> String {
    if let Some(js) = crate::retained::retained_whole_expression(node)
        && crate::retained::js_module_compatible(js)
    {
        let result = prefix_via_expr(js.ast, 0, js.raw, ctx, implicit_event);
        #[cfg(any(test, feature = "davinci-differential"))]
        #[expect(
            clippy::panic,
            reason = "differential oracle: a divergence must abort the run"
        )]
        {
            // Dual-run against the legacy wrapped parse in an uncounted
            // arena; divergence panics, never averages.
            let allocator = oxc_allocator::Allocator::default();
            let mut wrapped = String::with_capacity(js.raw.len() + 2);
            wrapped.push('(');
            wrapped.push_str(js.raw);
            wrapped.push(')');
            let legacy = Parser::new(&allocator, &wrapped, SourceType::default().with_module(true))
                .parse_expression()
                .map(|expr| prefix_via_expr(&expr, 1, js.raw, ctx, implicit_event))
                .unwrap_or_else(|_| {
                    panic!(
                        "davinci-differential (P1-7): js_module_compatible admitted an expression the legacy JS-module parse rejects: {:?}",
                        js.raw
                    )
                });
            assert_eq!(
                result.as_str(),
                legacy.as_str(),
                "davinci-differential (P1-7): retained codegen prefix walk diverged from the legacy re-parse for expression {:?}",
                js.raw
            );
            crate::retained::differential::record_codegen_rewrite_comparison();
        }
        return result;
    }
    prefix_identifiers_in_scope(node.content, ctx, implicit_event)
}

#[cfg(test)]
mod tests;
