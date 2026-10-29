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
use vize_l0::String;
use vize_relief::JsExpression;

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
    guard_maybe_ref_write: bool,
) -> RewriteResult {
    let mut collector = IdentifierCollector::new(ctx, wrapped, guard_maybe_ref_write);
    collector.visit_expression(expr);

    let has_identifiers = collector.has_identifiers;
    let used_unref = collector.used_unref;
    let used_is_ref = collector.used_is_ref;
    let result = splice_insertions(js_content, collector.rewrites, collector.suffix_rewrites, 1);

    RewriteResult {
        code: rewrite_props_aliases(result, ctx),
        used_unref,
        used_is_ref,
        has_identifiers,
        parse_error: None,
    }
}

/// Rewrite expressions owned by a `v-for` callback parameter without
/// rewriting the parameter declarations themselves. The synthesized arrow
/// gives the existing collector the real parameter scope before it visits
/// default initializers and computed keys.
pub(crate) fn rewrite_for_alias(alias: &str, ctx: &TransformContext<'_>) -> RewriteResult {
    let overflows = super::expression_exceeds_max_depth(alias);
    if overflows || !super::expression_has_balanced_delimiters(alias) {
        return RewriteResult {
            code: String::new(alias),
            used_unref: false,
            used_is_ref: false,
            has_identifiers: false,
            parse_error: (!overflows).then(|| String::new("mismatched expression delimiters")),
        };
    }

    let allocator = crate::expr_parse_probe::parse_arena();
    let mut wrapped = String::with_capacity(alias.len() + 10);
    wrapped.push('(');
    wrapped.push_str(alias);
    wrapped.push_str(") => null");
    let source_type = if ctx.options.is_ts {
        SourceType::ts().with_module(true)
    } else {
        SourceType::default().with_module(true)
    };
    match Parser::new(&allocator, &wrapped, source_type).parse_expression() {
        Ok(Expression::ArrowFunctionExpression(arrow)) => {
            let expression = Expression::ArrowFunctionExpression(arrow);
            rewrite_from_wrapped_expr(&expression, &wrapped, alias, ctx, false)
        }
        Ok(_) => RewriteResult {
            code: String::new(alias),
            used_unref: false,
            used_is_ref: false,
            has_identifiers: false,
            parse_error: Some(String::new("invalid parameters")),
        },
        Err(errors) => RewriteResult {
            code: String::new(alias),
            used_unref: false,
            used_is_ref: false,
            has_identifiers: false,
            parse_error: Some(
                errors
                    .first()
                    .map(|error| String::new(error.message.as_ref()))
                    .unwrap_or_else(|| String::new("invalid parameters")),
            ),
        },
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
    guard_maybe_ref_write: bool,
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
        Ok(expr) => {
            rewrite_from_wrapped_expr(&expr, &wrapped, &js_content, ctx, guard_maybe_ref_write)
        }
        Err(expression_errors) => {
            // Expression parsing failed - try parsing as a program (multi-statement handlers)
            let oxc_allocator2 = crate::expr_parse_probe::parse_arena();
            let parser2 = Parser::new(&oxc_allocator2, &js_content, source_type);
            let parse_result2 = parser2.parse();

            if parse_result2.diagnostics.is_empty() {
                // Successfully parsed as program - walk the AST and collect
                // identifiers. Program spans are content-relative already
                // (no wrapping parens), so the wrapper offset is 0.
                let mut collector =
                    IdentifierCollector::new(ctx, &js_content, guard_maybe_ref_write);
                collector.visit_program(&parse_result2.program);

                let has_identifiers = collector.has_identifiers;
                let used_unref = collector.used_unref;
                let used_is_ref = collector.used_is_ref;
                let result = splice_insertions(
                    &js_content,
                    collector.rewrites,
                    collector.suffix_rewrites,
                    0,
                );

                return RewriteResult {
                    code: rewrite_props_aliases(result, ctx),
                    used_unref,
                    used_is_ref,
                    has_identifiers,
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
                used_is_ref: false,
                has_identifiers: false,
                parse_error,
            }
        }
    }
}
