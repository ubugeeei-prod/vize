//! Helper functions for props destructure handling.
//!
//! Provides utility functions for generating props access expressions.

use vize_carton::String;

/// Generate prop access expression
pub fn gen_props_access_exp(key: &str) -> String {
    if is_simple_identifier(key) {
        let mut out = String::with_capacity(key.len() + 8);
        out.push_str("__props.");
        out.push_str(key);
        out
    } else {
        let mut out = String::with_capacity(key.len() + 10);
        out.push_str("__props[");
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:?}", key);
        out.push(']');
        out
    }
}

/// Check if string is a simple identifier
pub(crate) fn is_simple_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }

    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}
