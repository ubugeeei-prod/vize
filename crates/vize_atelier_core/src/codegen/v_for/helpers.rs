//! Helper functions for v-for code generation.
//!
//! Provides parameter extraction, destructuring pattern parsing,
//! and utility predicates for v-for rendering.

use crate::ast::{ElementNode, ExpressionNode, PropNode};
use vize_carton::String;
use vize_carton::ToCompactString;

/// Extract parameter names from a v-for callback expression.
/// Handles simple identifiers ("item"), destructuring patterns ("{ id, name }"),
/// nested destructure ("{ user: { name } }"), rest elements ("{ id, ...rest }"),
/// and array destructuring ("[first, second]").
pub(crate) fn extract_for_params(expr: &ExpressionNode<'_>, params: &mut Vec<String>) {
    let content = match expr {
        ExpressionNode::Simple(exp) => exp.content.as_str(),
        _ => return,
    };
    extract_destructure_params(content.trim(), params);
}

/// Recursively extract parameter names from a destructuring pattern string.
pub(crate) fn extract_destructure_params(trimmed: &str, params: &mut Vec<String>) {
    if trimmed.contains(',') && !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        for part in split_top_level(trimmed) {
            let part = part.trim();
            if !part.is_empty() {
                extract_destructure_params(part, params);
            }
        }
        return;
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len() - 1];
        // Split by commas at the top level (respecting nested braces/brackets)
        for part in split_top_level(inner) {
            let part = part.trim();
            // Handle rest element: ...rest
            if let Some(rest) = part.strip_prefix("...") {
                let rest = rest.trim();
                if !rest.is_empty() && is_valid_ident(rest) {
                    params.push(rest.to_compact_string());
                }
                continue;
            }
            // Handle default values: "item = default" -- take name before =
            if let Some(eq_pos) = part.find('=') {
                let name = part[..eq_pos].trim();
                if !name.is_empty() && is_valid_ident(name) {
                    params.push(name.to_compact_string());
                }
                continue;
            }
            // Handle renaming/nested: "original: value"
            if let Some(colon_pos) = part.find(':') {
                let value = part[colon_pos + 1..].trim();
                // Value might be another destructure pattern or a simple identifier
                if value.starts_with('{') || value.starts_with('[') {
                    extract_destructure_params(value, params);
                } else if is_valid_ident(value) {
                    params.push(value.to_compact_string());
                }
                continue;
            }
            // Simple identifier
            if !part.is_empty() && is_valid_ident(part) {
                params.push(part.to_compact_string());
            }
        }
    } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        for part in split_top_level(inner) {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("...") {
                let rest = rest.trim();
                if !rest.is_empty() && is_valid_ident(rest) {
                    params.push(rest.to_compact_string());
                }
            } else if part.starts_with('{') || part.starts_with('[') {
                extract_destructure_params(part, params);
            } else if !part.is_empty() && is_valid_ident(part) {
                params.push(part.to_compact_string());
            }
        }
    } else if is_valid_ident(trimmed) {
        params.push(trimmed.to_compact_string());
    }
}

/// Split a string by commas at the top level, respecting nested braces and brackets.
pub(crate) fn split_top_level(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, b) in s.bytes().enumerate() {
        match b {
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

/// Check if a string is a valid JS identifier
pub(crate) fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Check if content is a numeric literal (for v-for range)
pub(crate) fn is_numeric_content(content: &str) -> bool {
    !content.is_empty() && content.chars().all(|c| c.is_ascii_digit())
}

/// Check if source is a numeric literal (for v-for range)
pub fn is_numeric_source(source: &ExpressionNode<'_>) -> bool {
    if let ExpressionNode::Simple(exp) = source {
        is_numeric_content(exp.content.as_str())
    } else {
        false
    }
}

/// Check if element has a :key binding
pub fn get_element_key<'a, 'b>(el: &'b ElementNode<'a>) -> Option<&'b ExpressionNode<'a>>
where
    'a: 'b,
{
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    if arg.content == "key" {
                        return dir.exp.as_ref();
                    }
                }
            }
        }
    }
    None
}

/// Check if element has props besides the key
pub(crate) fn has_other_props(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| match p {
        PropNode::Attribute(_) => true,
        PropNode::Directive(dir) => {
            // Skip key binding (already handled separately)
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    if arg.content == "key" {
                        return false;
                    }
                }
            }
            // Skip v-for directive (handled by parent)
            if dir.name == "for" {
                return false;
            }
            // Skip v-memo directive (handled by withMemo wrapper)
            if dir.name == "memo" {
                return false;
            }
            true
        }
    })
}

/// Check if prop should be skipped for v-for item (key binding and v-for directive)
pub(crate) fn should_skip_prop(p: &PropNode<'_>) -> bool {
    if let PropNode::Directive(dir) = p {
        if dir.name == "bind" {
            if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                if arg.content == "key" {
                    return true;
                }
            }
        }
        // Skip v-for directive
        if dir.name == "for" {
            return true;
        }
        // Skip custom/unsupported directives (handled via withDirectives)
        if !super::super::props::is_supported_directive(dir) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_numeric_content;

    /// Test numeric source detection for v-for range expressions.
    #[test]
    fn test_is_numeric_content() {
        // Valid numeric literals (v-for="n in 10")
        assert!(is_numeric_content("10"));
        assert!(is_numeric_content("100"));
        assert!(is_numeric_content("0"));
        assert!(is_numeric_content("12345"));

        // Invalid: variable names
        assert!(!is_numeric_content("items"));
        assert!(!is_numeric_content("arr"));

        // Invalid: expressions
        assert!(!is_numeric_content("arr.length"));
        assert!(!is_numeric_content("10 + 5"));

        // Invalid: floating point
        assert!(!is_numeric_content("10.5"));

        // Invalid: empty string
        assert!(!is_numeric_content(""));
    }
}
