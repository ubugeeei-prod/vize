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

use fast::extract_identifiers_fast;
use slow::extract_identifiers_oxc_slow;

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
