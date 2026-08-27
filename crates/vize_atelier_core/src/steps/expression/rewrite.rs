//! Expression rewriting with identifier prefixing.
//!
//! Since Davinci P1-9 the rewrite is AST-driven where the admission proof
//! holds: a retained whole-expression AST (P1-5) that still describes the
//! node's exact bytes and passes the dialect gate (P1-7) is walked
//! directly — no guard re-scan, no re-parse — and the output bytes are
//! produced by span splicing into the original text
//! (`retained_rewrite.rs` + `splice.rs`). Everything else — params
//! positions, nodes without a retained AST, dialect-gate rejections, TS
//! strips that rewrote the text — keeps the legacy re-parse chain in
//! `reparse.rs`, with the residual classes recorded in
//! `plan/phase-1.md` P1-9.

use oxc_span::SourceType;
use vize_relief::JsExpression;
use vize_s0::String;

use crate::SourceLocation;
use crate::errors::ErrorCode;
use crate::lane::TransformContext;

use super::{
    parse_checks::parse_as_params, reparse::rewrite_reparsed, retained_rewrite::rewrite_retained,
    typescript::strip_typescript_from_expression,
};

fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

fn prop_access_expression(object: &str, key: &str) -> String {
    if super::prefix::is_simple_identifier(key) {
        let mut out = String::with_capacity(object.len() + key.len() + 1);
        out.push_str(object);
        out.push('.');
        out.push_str(key);
        return out;
    }

    let mut out = String::with_capacity(object.len() + key.len() + 4);
    out.push_str(object);
    out.push('[');
    use std::fmt::Write as _;
    let _ = write!(&mut out, "{:?}", key);
    out.push(']');
    out
}

fn replace_prefixed_alias_access(code: String, object: &str, local: &str, key: &str) -> String {
    let needle = {
        let mut needle = String::with_capacity(object.len() + local.len() + 1);
        needle.push_str(object);
        needle.push('.');
        needle.push_str(local);
        needle
    };
    let replacement = prop_access_expression(object, key);

    let mut result = String::with_capacity(code.len());
    let mut cursor = 0;
    while let Some(rel_pos) = code[cursor..].find(needle.as_str()) {
        let start = cursor + rel_pos;
        let end = start + needle.len();
        let after_ok = code[end..]
            .chars()
            .next()
            .is_none_or(|c| !is_identifier_continue(c));

        result.push_str(&code[cursor..start]);
        if after_ok {
            result.push_str(&replacement);
        } else {
            result.push_str(&code[start..end]);
        }
        cursor = end;
    }
    result.push_str(&code[cursor..]);
    result
}

/// Project prefixed props-alias accesses (`__props.local` / `$props.local`)
/// onto their real keys. A find/replace post-pass over the rewritten text,
/// shared by the retained and legacy paths alike: it is the alias
/// projection, not identifier prefixing, and P1-9 deliberately leaves it
/// byte-for-byte in place on both.
pub(super) fn rewrite_props_aliases(code: String, ctx: &TransformContext<'_>) -> String {
    let Some(bindings) = &ctx.options.binding_metadata else {
        return code;
    };
    if bindings.props_aliases.is_empty() {
        return code;
    }

    let mut rewritten = code;
    for (local, key) in &bindings.props_aliases {
        rewritten = replace_prefixed_alias_access(rewritten, "__props", local, key);
        rewritten = replace_prefixed_alias_access(rewritten, "$props", local, key);
    }
    rewritten
}

/// Result of expression rewriting
pub(crate) struct RewriteResult {
    pub(crate) code: String,
    pub(crate) used_unref: bool,
    /// Set when the expression could not be parsed at all and the raw
    /// content was passed through. Holds the parser's error detail so the
    /// caller can emit a compile diagnostic (mirroring `@vue/compiler-core`'s
    /// `X_INVALID_EXPRESSION`). `None` on every successful rewrite path.
    pub(crate) parse_error: Option<String>,
}

/// Emit an `InvalidExpression` compile diagnostic for an expression that
/// failed to parse, matching `@vue/compiler-core`'s
/// `Error parsing JavaScript expression: <detail>` message format.
pub(super) fn report_invalid_expression(
    ctx: &mut TransformContext<'_>,
    detail: &str,
    loc: &SourceLocation,
) {
    const PREFIX: &str = "Error parsing JavaScript expression: ";
    let mut message = String::with_capacity(PREFIX.len() + detail.len());
    message.push_str(PREFIX);
    message.push_str(detail);
    ctx.on_error_with_message(ErrorCode::InvalidExpression, message, Some(loc.clone()));
}

/// Rewrite an expression string, prefixing identifiers with `_ctx.` where needed
pub(crate) fn rewrite_expression(
    content: &str,
    ctx: &TransformContext<'_>,
    as_params: bool,
    retained: Option<&JsExpression<'_>>,
) -> RewriteResult {
    // Davinci P1-9: the retained AST drives the whole rewrite when its
    // admission proof holds, checked before any byte re-scan. The caller
    // gated `raw == content`, which makes the armature parse's nesting
    // guard a proof for these exact bytes — the same
    // `expression_is_safe_to_parse` the legacy path re-runs below — so the
    // depth/balance scans are skipped along with the re-parse. The dialect
    // gate keeps the retained walk byte-equivalent to the legacy JS-module
    // parse (P1-7).
    if !as_params
        && let Some(js) = retained
        && crate::retained::js_module_compatible(js)
    {
        if !ctx.options.is_ts {
            return rewrite_retained(js, ctx, as_params);
        }
        // TS lanes strip first, always: the stripper's detection scan can
        // false-positive on TS-free text (` as ` inside a string literal)
        // and rewrite bytes through its codegen round-trip. Only the
        // identity outcome keeps the retained byte proof; changed bytes
        // stay on the legacy chain — with the guard scans still skipped,
        // which the same armature proof covers.
        let js_content = strip_typescript_from_expression(content);
        if js_content.as_str() == content {
            return rewrite_retained(js, ctx, as_params);
        }
        #[cfg(any(test, feature = "davinci-differential"))]
        crate::retained::differential::record_transform_rewrite_legacy_ts_strip();
        return rewrite_reparsed(js_content, content, ctx, retained);
    }

    // Legacy string path. Classify the residual for the P1-9 coverage
    // ledger while the differential lane is armed.
    #[cfg(any(test, feature = "davinci-differential"))]
    {
        use crate::retained::differential as lane;
        if as_params {
            lane::record_transform_rewrite_legacy_params();
        } else if retained.is_none() {
            lane::record_transform_rewrite_legacy_unretained();
        } else {
            lane::record_transform_rewrite_legacy_dialect();
        }
    }

    // Pass raw content through instead of aborting: depth overflow keeps the
    // silent passthrough (#956); mismatched delimiters surface a diagnostic.
    let overflows = super::expression_exceeds_max_depth(content);
    if overflows || !super::expression_has_balanced_delimiters(content) {
        return RewriteResult {
            code: String::new(content),
            used_unref: false,
            parse_error: (!overflows).then(|| String::new("mismatched expression delimiters")),
        };
    }
    // First, if this is TypeScript, strip type annotations
    let js_content = if ctx.options.is_ts {
        strip_typescript_from_expression(content)
    } else {
        String::new(content)
    };

    if as_params {
        let js_source_type = SourceType::default().with_module(true);
        if parse_as_params(&js_content, js_source_type).is_ok()
            || (ctx.options.is_ts
                && parse_as_params(content, SourceType::ts().with_module(true)).is_ok())
        {
            return RewriteResult {
                code: js_content,
                used_unref: false,
                parse_error: None,
            };
        }

        let detail = parse_as_params(&js_content, js_source_type)
            .err()
            .unwrap_or_else(|| String::new("invalid parameters"));
        return RewriteResult {
            code: js_content,
            used_unref: false,
            parse_error: Some(detail),
        };
    }

    rewrite_reparsed(js_content, content, ctx, retained)
}
