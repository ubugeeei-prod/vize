//! Identifier extraction from Vue template expressions.
//!
//! Provides hybrid extraction strategies:
//! - **Fast path**: String-based scanning for simple expressions
//! - **Slow path**: OXC AST-based extraction for complex expressions
//!   (object literals, type assertions, arrow functions)
//!
//! Only "root" identifiers are extracted -- property accesses like
//! `item.name` yield only `"item"`, not `"name"`.

mod comments;
mod fast;
mod slow;

#[cfg(test)]
mod tests;

pub use comments::strip_js_comments;

use vize_carton::{CompactString, profile};
use vize_relief::JsExpression;

use fast::{extract_identifier_refs_fast, extract_identifiers_fast};
use slow::{
    extract_identifier_refs_oxc_slow, extract_identifiers_oxc_slow,
    extract_identifiers_retained_slow,
};

/// Root identifier reference extracted from a template expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentifierRef {
    pub name: CompactString,
    /// Byte offset relative to the expression source.
    pub offset: u32,
}

impl IdentifierRef {
    #[inline]
    pub(super) fn new(name: &str, offset: u32) -> Self {
        Self {
            name: CompactString::new(name),
            offset,
        }
    }
}

/// Dispatch heuristic shared by every hybrid entry point: constructs a string
/// scanner cannot handle move to the OXC AST path.
///
/// - Object literals: `{ }`
/// - Type assertions: `as Type`
/// - Arrow functions: `() =>`
/// - Regex literals and division: `/`
/// - Statement bodies: `;`
#[inline]
fn needs_ast_extraction(expr: &str) -> bool {
    !expr.is_ascii()
        || expr.contains('{')
        || expr.contains(" as ")
        || expr.contains("=>")
        || expr.contains('/')
        || expr.contains(';')
}

/// Hybrid identifier extraction - fast path for simple expressions, OXC for complex ones.
/// Only extracts "root" identifiers - identifiers that are references, not:
/// - Property accesses (item.name -> only "item" extracted)
/// - Object literal keys ({ active: value } -> only "value" extracted)
/// - String literals, computed property names, etc.
#[inline]
pub fn extract_identifiers_oxc(expr: &str) -> Vec<CompactString> {
    let stripped = strip_js_comments(expr);
    let expr = stripped.as_ref();

    if needs_ast_extraction(expr) {
        return profile!(
            "croquis.helpers.identifiers.slow",
            extract_identifiers_oxc_slow(expr)
        );
    }

    profile!(
        "croquis.helpers.identifiers.fast",
        extract_identifiers_fast(expr)
    )
}

/// Node-aware hybrid identifier extraction over the parse-once retained AST
/// (Davinci P1-6).
///
/// Identical dispatch and results to [`extract_identifiers_oxc`], but when
/// the expression's node carries the retained AST (P1-5,
/// `SimpleExpressionNode::js_ast`) **and** [`strip_js_comments`] left the
/// text unchanged, the slow branch walks it instead of re-parsing the text
/// into a throwaway arena. `retained` must be the node's own `js_ast` for
/// `expr` — by the P1-5 contract `js.raw` string-equals the node content, so
/// on the comment-free path the legacy parse and the retained AST describe
/// the **exact same bytes** and equality is a parser-determinism fact, not a
/// comment-semantics argument.
///
/// Fallback classes keep the legacy re-parse unchanged until P1-8 deletes
/// the split: nodes without a retained AST (v-for sub-expressions, v-on
/// statement bodies, guard-refused or invalid text, compound expressions)
/// and text `strip_js_comments` rewrites. The latter is deliberate:
/// the stripper is not regex-aware, so text like `/[/*]/.test(x)` is
/// mangled before the legacy parse, and reading the retained AST there
/// would *fix* that scanner bug — a behavior change P1-8's waiver-reviewed
/// differential owns, not this migration.
///
/// Under `cfg(any(test, feature = "davinci-differential"))` every retained
/// walk is dual-run against the legacy re-parse and divergence panics — the
/// P1-6 differential lane.
#[inline]
pub fn extract_identifiers_retained(
    expr: &str,
    retained: Option<&JsExpression<'_>>,
) -> Vec<CompactString> {
    let stripped = strip_js_comments(expr);
    let comment_free = matches!(stripped, std::borrow::Cow::Borrowed(_));
    let stripped = stripped.as_ref();

    if needs_ast_extraction(stripped) {
        let result = match retained {
            Some(js) if comment_free => {
                debug_assert_eq!(
                    js.raw, expr,
                    "js_ast must be the node's own retained parse of `expr` (P1-5 contract)"
                );
                profile!(
                    "croquis.helpers.identifiers.retained",
                    extract_identifiers_retained_slow(js.ast)
                )
            }
            _ => profile!(
                "croquis.helpers.identifiers.slow",
                extract_identifiers_oxc_slow(stripped)
            ),
        };
        #[cfg(any(test, feature = "davinci-differential"))]
        if retained.is_some() && comment_free {
            assert_retained_identifiers_agree(expr, &result);
        }
        return result;
    }

    profile!(
        "croquis.helpers.identifiers.fast",
        extract_identifiers_fast(stripped)
    )
}

/// Davinci P1-6 differential lane: the retained-AST walk must reproduce the
/// legacy re-parse byte-for-byte (same names, same order). Any divergence is
/// a bug in one side — panic, never average. Only comment-free text reaches
/// this point, so both sides consume identical bytes.
#[cfg(any(test, feature = "davinci-differential"))]
fn assert_retained_identifiers_agree(expr: &str, retained_result: &[CompactString]) {
    let legacy = extract_identifiers_oxc_slow(expr);
    assert_eq!(
        retained_result,
        legacy.as_slice(),
        "davinci-differential (P1-6): retained-AST identifier walk diverged from the legacy re-parse for expression {expr:?}"
    );
    crate::drawer::differential::record_identifier_comparison();
}

/// Hybrid root identifier extraction with byte offsets in the original expression.
#[inline]
pub fn extract_identifier_refs_oxc(expr: &str) -> Vec<IdentifierRef> {
    // Use OXC for constructs where a string scanner cannot cheaply preserve
    // semantic identifier spans. Comments contain `/`, so this keeps offsets in
    // the original expression instead of using `strip_js_comments`.
    if needs_ast_extraction(expr) {
        return profile!(
            "croquis.helpers.identifier_refs.slow",
            extract_identifier_refs_oxc_slow(expr)
        );
    }

    profile!(
        "croquis.helpers.identifier_refs.fast",
        extract_identifier_refs_fast(expr)
    )
}
