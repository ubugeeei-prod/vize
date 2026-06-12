//! Slot parameter helpers (scoped slot props parsing and prefixing).

use crate::*;
use vize_carton::String;

/// Get slot props expression as raw source (not transformed)
pub(super) fn get_slot_props(dir: &DirectiveNode<'_>) -> Option<vize_carton::String> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(s) => s.loc.source.clone(),
        ExpressionNode::Compound(c) => c.loc.source.clone(),
    })
}

/// Add _ctx. prefix to default value identifiers in destructuring patterns.
/// e.g., "{ item = defaultItem }" -> "{ item = _ctx.defaultItem }"
/// Only processes identifiers after `=` (default values), not the param names.
pub(super) fn prefix_slot_defaults(source: &str) -> String {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + 20);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'=' {
            // Skip == and =>
            if i + 1 < len && (bytes[i + 1] == b'=' || bytes[i + 1] == b'>') {
                result.push(bytes[i] as char);
                result.push(bytes[i + 1] as char);
                i += 2;
                continue;
            }
            result.push('=');
            i += 1;
            // Skip whitespace after =
            while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
                result.push(bytes[i] as char);
                i += 1;
            }
            // Check if next is a simple identifier (not a literal/number/string/object)
            if i < len && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' || bytes[i] == b'$') {
                // Collect the identifier
                let start = i;
                while i < len
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
                {
                    i += 1;
                }
                let ident = &source[start..i];
                // Don't prefix keywords/literals
                if !matches!(
                    ident,
                    "true" | "false" | "null" | "undefined" | "NaN" | "Infinity"
                ) {
                    result.push_str("_ctx.");
                }
                result.push_str(ident);
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Extract parameter names from slot props expression
/// e.g., "{ item }" -> ["item"], "{ item, index }" -> ["item", "index"]
/// e.g., "slotProps" -> ["slotProps"]
pub(super) fn extract_slot_params(props_str: &str) -> Vec<String> {
    let mut params = Vec::new();
    super::super::v_for::extract_destructure_params(props_str.trim(), &mut params);
    params
}
