//! Helper functions for props destructure handling.
//!
//! Provides utility functions for generating props access expressions
//! and text-based identifier replacement.

use vize_carton::{FxHashMap, String, ToCompactString};

/// Sentinel value for rest spread identifiers in local_to_key map.
/// When `gen_props_access_exp` receives this, it returns just `__props`.
pub(crate) const PROPS_REST_SENTINEL: &str = "\0__REST__";

/// Generate prop access expression
pub fn gen_props_access_exp(key: &str) -> String {
    // Rest spread sentinel: just return `__props` (no property access)
    if key == PROPS_REST_SENTINEL {
        return "__props".to_compact_string();
    }
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

/// Check if character can be part of an identifier
pub(crate) fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Replace identifier occurrences with proper word boundary checking
pub(crate) fn replace_identifier(source: &str, name: &str, replacement: &str) -> String {
    let mut result = String::default();
    let chars: Vec<char> = source.chars().collect();
    let name_chars: Vec<char> = name.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check if we're at the start of the identifier
        if i + name_chars.len() <= chars.len() {
            let potential_match: String = chars[i..i + name_chars.len()].iter().copied().collect();
            if potential_match.as_str() == name {
                // Check word boundaries
                let before_ok = i == 0 || !is_identifier_char(chars[i - 1]);
                let after_ok = i + name_chars.len() >= chars.len()
                    || !is_identifier_char(chars[i + name_chars.len()]);

                // Check not preceded by . (member access) or __props already
                let not_member = i == 0 || chars[i - 1] != '.';

                if before_ok && after_ok && not_member {
                    result.push_str(replacement);
                    i += name_chars.len();
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Text-based transformation fallback
pub(crate) fn transform_props_text_based(
    source: &str,
    local_to_key: &FxHashMap<&str, &str>,
) -> String {
    let mut result = source.to_compact_string();

    // Sort by length (longest first) to avoid partial replacements
    let mut props: Vec<(&str, &str)> = local_to_key.iter().map(|(k, v)| (*k, *v)).collect();
    props.sort_by_key(|prop| std::cmp::Reverse(prop.0.len()));

    for (local, key) in props {
        result = replace_identifier(&result, local, &gen_props_access_exp(key));
    }

    result
}
