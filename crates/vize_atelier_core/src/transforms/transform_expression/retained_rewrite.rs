//! Retained-AST fast path for the prefix rewrite (Davinci P1-7).
//!
//! When the node's parse-once AST still describes the exact bytes being
//! rewritten (`raw == content`, no TS stripping applied, and the dialect
//! gate `crate::retained::js_module_compatible` holds), the identifier
//! collector walks the retained AST instead of re-parsing `(content)` into a
//! throwaway arena. Spans are then content-relative, so rewrites apply
//! without the wrapper's `-1` adjustment, and the collector's
//! `wrapped: false` mode reproduces the one observable artifact of the
//! wrapper: an assignment-target paren scan that runs to the end of the
//! content would, on the legacy path, run through the wrapper `)` and be
//! dropped by the apply loop's bounds check.
//!
//! Under `cfg(any(test, feature = "davinci-differential"))` every fast-path
//! result is dual-run against the legacy wrapped parse and compared exactly
//! (code bytes and helper usage); divergence panics.

use vize_carton::String;
use vize_relief::JsExpression;

use crate::lane::TransformContext;

use super::collector::IdentifierCollector;
use super::rewrite::RewriteResult;

/// Rewrite via the retained AST. Caller guarantees `js.raw` equals the text
/// being rewritten and the dialect gate passed.
pub(super) fn rewrite_retained(
    js: &JsExpression<'_>,
    ctx: &TransformContext<'_>,
    as_params: bool,
) -> RewriteResult {
    debug_assert!(!as_params);
    let _ = as_params;

    let mut collector = IdentifierCollector::new_unwrapped(ctx, js.raw);
    oxc_ast_visit::Visit::visit_expression(&mut collector, js.ast);

    let used_unref = collector.used_unref;

    let mut all_rewrites: Vec<(usize, String, String)> = collector
        .rewrites
        .into_iter()
        .map(|(pos, prefix)| (pos, prefix, String::default()))
        .collect();
    for (pos, suffix) in collector.suffix_rewrites {
        all_rewrites.push((pos, String::default(), suffix));
    }
    all_rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

    let mut result = String::new(js.raw);
    for (pos, prefix, suffix) in all_rewrites {
        // Content-relative spans: no wrapper adjustment (legacy subtracts 1).
        if pos <= result.len() {
            if !suffix.is_empty() {
                result.insert_str(pos, &suffix);
            }
            if !prefix.is_empty() {
                result.insert_str(pos, &prefix);
            }
        }
    }

    let result = RewriteResult {
        code: super::rewrite::rewrite_props_aliases(result, ctx),
        used_unref,
        parse_error: None,
    };

    #[cfg(any(test, feature = "davinci-differential"))]
    assert_rewrite_agrees(js, ctx, &result);

    result
}

/// Davinci P1-7 differential lane: the retained walk must reproduce the
/// legacy wrapped re-parse byte-for-byte. Any divergence is a bug in one
/// side — panic, never average. The legacy side runs in its own uncounted
/// arena: lane-only work must not disturb the production re-parse floor.
#[cfg(any(test, feature = "davinci-differential"))]
fn assert_rewrite_agrees(
    js: &JsExpression<'_>,
    ctx: &TransformContext<'_>,
    retained: &RewriteResult,
) {
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use super::rewrite::rewrite_from_wrapped_expr;

    let allocator = oxc_allocator::Allocator::default();
    let mut wrapped = String::with_capacity(js.raw.len() + 2);
    wrapped.push('(');
    wrapped.push_str(js.raw);
    wrapped.push(')');
    let legacy = Parser::new(
        &allocator,
        &wrapped,
        SourceType::default().with_module(true),
    )
    .parse_expression()
    .unwrap_or_else(|_| {
        panic!(
            "davinci-differential (P1-7): js_module_compatible admitted an \
                 expression the legacy JS-module parse rejects: {:?}",
            js.raw
        )
    });
    let legacy = rewrite_from_wrapped_expr(&legacy, &wrapped, js.raw, ctx);
    assert_eq!(
        (retained.code.as_str(), retained.used_unref),
        (legacy.code.as_str(), legacy.used_unref),
        "davinci-differential (P1-7): retained-AST rewrite diverged from the legacy re-parse for expression {:?}",
        js.raw
    );
    crate::retained::differential::record_transform_rewrite_comparison();
}
