//! CSS `v-bind()` expressions read from `<style>` blocks.
//!
//! The descriptor records the expressions as CSS variables. The compiler's
//! stylesheet rewrite reuses the same scanner, so both agree on comment,
//! string and nesting rules.

mod scanner;

use alloc::vec::Vec;
use vize_l0::{String, ToCompactString};

pub use scanner::{find_matching_paren, find_next_v_bind};

/// The `v-bind()` expressions of `css`, trimmed and unquoted, in source order.
#[doc(hidden)]
pub fn extract_v_bind_vars(css: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut pos = 0;
    while let Some(open) = find_next_v_bind(css, pos) {
        let start = open + 7;
        let Some(end) = css.get(start..).and_then(find_matching_paren) else {
            break;
        };
        if let Some(expr) = css.get(start..start + end) {
            vars.push(trim_outer_quotes(expr.trim()).to_compact_string());
        }
        pos = start + end + 1;
    }
    vars
}

/// `expr` without one pair of matching outer quotes.
#[doc(hidden)]
pub fn trim_outer_quotes(expr: &str) -> &str {
    let bytes = expr.as_bytes();
    if bytes.len() >= 2
        && matches!(bytes.first(), Some(b'"' | b'\''))
        && bytes.first() == bytes.last()
    {
        expr.get(1..expr.len() - 1).unwrap_or_default()
    } else {
        expr
    }
}
