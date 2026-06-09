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

use fast::{extract_identifier_refs_fast, extract_identifiers_fast};
use slow::{extract_identifier_refs_oxc_slow, extract_identifiers_oxc_slow};

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

/// Hybrid identifier extraction - fast path for simple expressions, OXC for complex ones.
/// Only extracts "root" identifiers - identifiers that are references, not:
/// - Property accesses (item.name -> only "item" extracted)
/// - Object literal keys ({ active: value } -> only "value" extracted)
/// - String literals, computed property names, etc.
#[inline]
pub fn extract_identifiers_oxc(expr: &str) -> Vec<CompactString> {
    let stripped = strip_js_comments(expr);
    let expr = stripped.as_ref();

    // Use OXC parser for complex expressions:
    // - Object literals: { }
    // - Type assertions: as Type
    // - Arrow functions: () =>
    // - Regex literals and division: /
    if expr.contains('{') || expr.contains(" as ") || expr.contains("=>") || expr.contains('/') {
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

/// Hybrid root identifier extraction with byte offsets in the original expression.
#[inline]
pub fn extract_identifier_refs_oxc(expr: &str) -> Vec<IdentifierRef> {
    // Use OXC for constructs where a string scanner cannot cheaply preserve
    // semantic identifier spans. Comments contain `/`, so this keeps offsets in
    // the original expression instead of using `strip_js_comments`.
    if expr.contains('{') || expr.contains(" as ") || expr.contains("=>") || expr.contains('/') {
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
