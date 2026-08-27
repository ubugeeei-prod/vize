//! The AST-driven prefix rewrite over retained expressions (Davinci P1-9,
//! parse side landed in P1-7).
//!
//! For admitted inputs — the node's parse-once AST still describes the
//! exact bytes being rewritten (`raw == content`, no TS strip applied or a
//! byte-identity strip) and the dialect gate
//! `crate::retained::js_module_compatible` holds — the identifier collector
//! walks the retained AST and the output bytes are produced by
//! [`super::splice`]: the identifier spans' prefix/suffix insertions are
//! spliced into the original text in one forward pass, every other byte
//! verbatim. Spans are content-relative, so no wrapper offset applies, and
//! the collector's `wrapped: false` mode reproduces the one observable
//! artifact of the legacy `(content)` wrapper: an assignment-target paren
//! scan that runs to the end of the content would, on the legacy path, run
//! through the wrapper `)` and be dropped by the splicer's bounds check.
//!
//! Under `cfg(any(test, feature = "davinci-differential"))` every AST-driven
//! result is dual-run against the legacy wrapped re-parse and compared
//! exactly (code bytes and helper usage); divergence panics.

use vize_relief::JsExpression;
#[cfg(any(test, feature = "davinci-differential"))]
use vize_s0::String;

use crate::lane::TransformContext;

use super::collector::IdentifierCollector;
use super::rewrite::RewriteResult;
use super::splice::splice_insertions;

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
    // Content-relative spans: wrapper offset 0 (the legacy path subtracts 1).
    let result = splice_insertions(js.raw, collector.rewrites, collector.suffix_rewrites, 0);

    let result = RewriteResult {
        code: super::rewrite::rewrite_props_aliases(result, ctx),
        used_unref,
        parse_error: None,
    };

    #[cfg(any(test, feature = "davinci-differential"))]
    assert_rewrite_agrees(js, ctx, &result);

    result
}

/// Davinci differential lane (P1-6 pattern): the AST-driven splice must
/// reproduce the full legacy pipeline byte-for-byte — the wrapped re-parse
/// AND the retired end-to-start `insert_str` string rewriter, kept verbatim
/// below as the oracle so the corpus lane compares the new mechanism
/// against the exact bytes the old one produced. Any divergence is a bug in
/// one side — panic, never average. The legacy side runs in its own
/// uncounted arena: lane-only work must not disturb the production
/// re-parse floor.
#[cfg(any(test, feature = "davinci-differential"))]
fn assert_rewrite_agrees(
    js: &JsExpression<'_>,
    ctx: &TransformContext<'_>,
    retained: &RewriteResult,
) {
    use oxc_parser::Parser;
    use oxc_span::SourceType;

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

    // The retired string rewriter, verbatim: stable descending sort, then
    // end-to-start `insert_str` with the grow-aware bounds check and the
    // wrapper `-1` span adjustment.
    let mut collector = IdentifierCollector::new(ctx, &wrapped);
    oxc_ast_visit::Visit::visit_expression(&mut collector, &legacy);
    let legacy_used_unref = collector.used_unref;
    let mut all_rewrites: Vec<(usize, String, String)> = collector
        .rewrites
        .into_iter()
        .map(|(pos, prefix)| (pos, prefix, String::default()))
        .collect();
    for (pos, suffix) in collector.suffix_rewrites {
        all_rewrites.push((pos, String::default(), suffix));
    }
    all_rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));
    let mut legacy_code = String::new(js.raw);
    for (pos, prefix, suffix) in all_rewrites {
        let adjusted_pos = pos.saturating_sub(1);
        if adjusted_pos <= legacy_code.len() {
            if !suffix.is_empty() {
                legacy_code.insert_str(adjusted_pos, &suffix);
            }
            if !prefix.is_empty() {
                legacy_code.insert_str(adjusted_pos, &prefix);
            }
        }
    }
    let legacy_code = super::rewrite::rewrite_props_aliases(legacy_code, ctx);

    assert_eq!(
        (retained.code.as_str(), retained.used_unref),
        (legacy_code.as_str(), legacy_used_unref),
        "davinci-differential (P1-9): the AST-driven splice diverged from the legacy string rewrite for expression {:?}",
        js.raw
    );
    crate::retained::differential::record_transform_rewrite_comparison();
}
