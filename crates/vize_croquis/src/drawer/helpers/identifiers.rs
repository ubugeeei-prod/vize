//! Identifier extraction from Vue template expressions.
//!
//! Parses template expressions with OXC and walks the AST for references.
//!
//! Only "root" identifiers are extracted -- property accesses like
//! `item.name` yield only `"item"`, not `"name"`.

mod ast;
mod checked;
mod comments;

#[cfg(test)]
mod tests;

pub use comments::strip_js_comments;

/// Demand-only reads, or `None` when syntax or eval prevents a sound conclusion.
#[doc(hidden)]
pub fn extract_identifiers_checked(source: &str) -> Option<Vec<CompactString>> {
    checked::checked_reads(source)
}

/// The same demanded expression analysis, retaining exact authored offsets.
#[doc(hidden)]
pub fn extract_identifier_refs_checked(source: &str) -> Option<Vec<IdentifierRef>> {
    checked::checked_references(source)
}

/// Retain cooked names even when their physical authored spans cannot be proved.
#[doc(hidden)]
pub fn extract_identifier_refs_with_witness(source: &str) -> Option<(Vec<IdentifierRef>, bool)> {
    checked::checked_references_with_witness(source)
}

use vize_carton::{CompactString, profile};
use vize_relief::JsExpression;

use ast::{
    extract_identifier_refs_oxc_ast, extract_identifiers_oxc_ast, extract_identifiers_retained_ast,
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

/// Identifier extraction through the OXC AST.
/// Only extracts "root" identifiers - identifiers that are references, not:
/// - Property accesses (item.name -> only "item" extracted)
/// - Object literal keys ({ active: value } -> only "value" extracted)
/// - String literals, computed property names, etc.
#[inline]
pub fn extract_identifiers_oxc(expr: &str) -> Vec<CompactString> {
    let stripped = strip_js_comments(expr);
    let expr = stripped.as_ref();

    profile!(
        "croquis.helpers.identifiers.ast",
        extract_identifiers_oxc_ast(expr)
    )
}

/// Node-aware hybrid identifier extraction over the parse-once retained AST
/// (Davinci P1-6).
///
/// Identical results to [`extract_identifiers_oxc`], but when the expression's
/// node carries the retained AST (P1-5, `SimpleExpressionNode::js_ast`) **and**
/// [`strip_js_comments`] left the text unchanged, this walks it instead of
/// re-parsing the text
/// into a throwaway arena. `retained` must be the node's own `js_ast` for
/// `expr` — by the P1-5 contract `js.raw` string-equals the node content, so
/// on the comment-free path the legacy parse and the retained AST describe
/// the **exact same bytes** and equality is a parser-determinism fact, not a
/// comment-semantics argument.
///
/// Fallback classes keep the legacy re-parse unchanged: nodes without a
/// retained AST (v-for sub-expressions, v-on statement bodies, guard-refused or
/// invalid text, compound expressions) and text `strip_js_comments` rewrites.
/// The latter is deliberate: the stripper is not regex-aware, so text like
/// `/[/*]/.test(x)` is mangled before the parse, and reading the retained AST
/// there would change that behavior outside the scanner-split deletion.
///
/// Under `cfg(any(test, feature = "legacy-differential"))` every retained
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

    let result = match retained {
        Some(js) if comment_free => {
            debug_assert_eq!(
                js.raw, expr,
                "js_ast must be the node's own retained parse of `expr` (P1-5 contract)"
            );
            profile!(
                "croquis.helpers.identifiers.retained",
                extract_identifiers_retained_ast(js.ast)
            )
        }
        _ => profile!(
            "croquis.helpers.identifiers.ast",
            extract_identifiers_oxc_ast(stripped)
        ),
    };
    #[cfg(any(test, feature = "legacy-differential"))]
    if retained.is_some() && comment_free {
        assert_retained_identifiers_agree(expr, &result);
    }
    result
}

/// Davinci P1-6 differential lane: the retained-AST walk must reproduce the
/// legacy re-parse byte-for-byte (same names, same order). Any divergence is
/// a bug in one side — panic, never average. Only comment-free text reaches
/// this point, so both sides consume identical bytes.
#[cfg(any(test, feature = "legacy-differential"))]
fn assert_retained_identifiers_agree(expr: &str, retained_result: &[CompactString]) {
    let legacy = extract_identifiers_oxc_ast(expr);
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
    profile!(
        "croquis.helpers.identifier_refs.ast",
        extract_identifier_refs_oxc_ast(expr)
    )
}

/// Read exact authored offsets during the existing expression walk.
/// The existing comment transform carries an opt-in original-byte witness.
/// A fallback parses that same view once; it does not follow a name walk.
pub(in crate::drawer) fn extract_identifier_refs_retained(
    expr: &str,
    retained: Option<&JsExpression<'_>>,
) -> (Vec<CompactString>, Option<Vec<IdentifierRef>>) {
    let (stripped, mapping) = comments::strip_comments_with_offsets(expr);
    let (references, complete) = match retained {
        Some(js) if mapping.is_none() && js.raw == expr => ast::retained_references(js.ast, true),
        _ => ast::parsed_references(stripped.as_ref(), true),
    };
    // Preserve the ordinary names from precisely the same legacy analysis view.
    let names: Vec<_> = references
        .iter()
        .map(|reference| reference.name.clone())
        .collect();
    #[cfg(any(test, feature = "legacy-differential"))]
    if retained.is_some() && mapping.is_none() {
        assert_retained_identifiers_agree(expr, &names);
    }
    let references = references
        .into_iter()
        .map(|mut reference| {
            if let Some(mapping) = mapping.as_ref() {
                let start = reference.offset as usize;
                let end = start.checked_add(reference.name.len())?;
                let mapped = *mapping.get(start)?;
                if mapped == u32::MAX
                    || !(start..end).all(|at| {
                        mapping.get(at).copied() == mapped.checked_add((at - start) as u32)
                    })
                {
                    return None;
                }
                reference.offset = mapped;
            }
            let start = reference.offset as usize;
            let end = start.checked_add(reference.name.len())?;
            (expr.get(start..end) == Some(reference.name.as_str())).then_some(reference)
        })
        .collect();
    (names, if complete { references } else { None })
}
