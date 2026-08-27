//! Legacy re-parse chain for the prefix rewrite (the non-admitted path).
//!
//! Inputs without an admitted retained AST — no AST retained for these
//! bytes, dialect-gate rejection, or a TS strip that rewrote the text —
//! are re-parsed here exactly as before Davinci P1-9: wrapped `(content)`
//! expression parse, then a whole-program parse for multi-statement
//! handlers, then the simple-identifier fallback. Only the emission
//! changed: collected rewrites are applied by [`super::splice`], the same
//! span splicer the retained path uses. Deleting this chain is a follow-up
//! (plan/phase-1.md P1-9 records the residual classes), not this task.

use oxc_ast::ast::Expression;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_relief::JsExpression;
use vize_s0::String;

use crate::lane::TransformContext;

use super::{
    collector::IdentifierCollector,
    parse_checks::parses_as_typescript,
    prefix::{get_identifier_prefix, is_ref_binding_simple, is_simple_identifier},
    rewrite::{RewriteResult, rewrite_props_aliases},
    splice::splice_insertions,
};

/// Rewrite a successfully parsed `(js_content)` expression: walk the AST,
/// collect prefix/suffix rewrites, and splice them into `js_content`
/// (spans are wrapped-text relative, hence the wrapper offset of 1).
pub(super) fn rewrite_from_wrapped_expr(
    expr: &Expression<'_>,
    wrapped: &str,
    js_content: &str,
    ctx: &TransformContext<'_>,
) -> RewriteResult {
    let mut collector = IdentifierCollector::new(ctx, wrapped);
    collector.visit_expression(expr);

    let used_unref = collector.used_unref;
    let result = splice_insertions(js_content, collector.rewrites, collector.suffix_rewrites, 1);

    RewriteResult {
        code: rewrite_props_aliases(result, ctx),
        used_unref,
        parse_error: None,
    }
}

/// The legacy re-parse chain over already-TS-stripped text: wrapped
/// expression parse, program-parse fallback, simple-identifier fallback.
/// `content` is the original (pre-strip) text, consulted only for the
/// TS-acceptance diagnostic check; `retained` short-circuits that check
/// when the dialect-gated AST already proves the original parses as TS.
pub(super) fn rewrite_reparsed(
    js_content: String,
    content: &str,
    ctx: &TransformContext<'_>,
    retained: Option<&JsExpression<'_>>,
) -> RewriteResult {
    let oxc_allocator = crate::expr_parse_probe::parse_arena();
    let source_type = SourceType::default().with_module(true);

    // Wrap in parentheses to make it a valid expression statement
    let mut wrapped = String::with_capacity(js_content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(&js_content);
    wrapped.push(')');
    let parser = Parser::new(&oxc_allocator, &wrapped, source_type);
    let parse_result = parser.parse_expression();

    match parse_result {
        Ok(expr) => rewrite_from_wrapped_expr(&expr, &wrapped, &js_content, ctx),
        Err(expression_errors) => {
            // Expression parsing failed - try parsing as a program (multi-statement handlers)
            let oxc_allocator2 = crate::expr_parse_probe::parse_arena();
            let parser2 = Parser::new(&oxc_allocator2, &js_content, source_type);
            let parse_result2 = parser2.parse();

            if parse_result2.diagnostics.is_empty() {
                // Successfully parsed as program - walk the AST and collect
                // identifiers. Program spans are content-relative already
                // (no wrapping parens), so the wrapper offset is 0.
                let mut collector = IdentifierCollector::new(ctx, &js_content);
                collector.visit_program(&parse_result2.program);

                let used_unref = collector.used_unref;
                let result = splice_insertions(
                    &js_content,
                    collector.rewrites,
                    collector.suffix_rewrites,
                    0,
                );

                return RewriteResult {
                    code: rewrite_props_aliases(result, ctx),
                    used_unref,
                    parse_error: None,
                };
            }

            // Program parsing also failed - fallback to simple identifier check
            let mut parse_error = None;
            let code: String = if is_simple_identifier(&js_content) {
                // Reserved words (`class`, `default`, …) fail to parse as an
                // expression but are still rewritable identifiers. Vue treats
                // them through its simple-identifier fast path without ever
                // parsing, so no diagnostic is emitted here either.
                if let Some(prefix) = get_identifier_prefix(&js_content, ctx) {
                    let mut s = String::with_capacity(prefix.len() + js_content.len());
                    s.push_str(prefix);
                    s.push_str(&js_content);
                    s
                } else if is_ref_binding_simple(&js_content, ctx) {
                    // Add .value for refs in inline mode
                    let mut s = String::with_capacity(js_content.len() + 6);
                    s.push_str(&js_content);
                    s.push_str(".value");
                    s
                } else {
                    js_content
                }
            } else {
                // The raw content is passed through unprefixed. The official
                // compiler reports `X_INVALID_EXPRESSION` here, so surface the
                // parser detail for the caller to emit a diagnostic — unless
                // the original source is valid TypeScript that only vize's
                // TS-stripping fallback failed to lower (the official compiler
                // accepts it, so vize must not reject it). A retained AST that
                // passed the dialect gate is itself proof the original parses
                // as TypeScript (P1-7), so the re-check is skipped.
                let ts_accepts = ctx.options.is_ts
                    && (retained.is_some_and(crate::retained::js_module_compatible)
                        || parses_as_typescript(content));
                if !ts_accepts {
                    parse_error = Some(
                        expression_errors
                            .first()
                            .map(|error| String::new(error.message.as_ref()))
                            .unwrap_or_else(|| String::new("invalid expression")),
                    );
                }
                js_content
            };
            RewriteResult {
                code: rewrite_props_aliases(code, ctx),
                used_unref: false,
                parse_error,
            }
        }
    }
}
