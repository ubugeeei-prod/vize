//! Binding extraction for JavaScript destructuring patterns.

use vize_carton::{FxHashSet, String, ToCompactString};

pub(crate) fn pattern_bindings(pattern: &str) -> FxHashSet<String> {
    let mut bindings = FxHashSet::default();
    extract_pattern_bindings(pattern.trim(), &mut bindings);
    bindings
}

fn extract_pattern_bindings(value: &str, bindings: &mut FxHashSet<String>) {
    if value.starts_with('(') && value.ends_with(')') {
        extract_pattern_bindings(value[1..value.len() - 1].trim(), bindings);
        return;
    }
    if value.contains(',') && !value.starts_with('{') && !value.starts_with('[') {
        for part in split_top_level(value) {
            extract_pattern_bindings(part.trim(), bindings);
        }
        return;
    }
    if !value.starts_with('{')
        && !value.starts_with('[')
        && let Some(equal) = value.find('=')
    {
        extract_pattern_bindings(value[..equal].trim(), bindings);
        return;
    }
    if value.starts_with('{') && value.ends_with('}') {
        for part in split_top_level(&value[1..value.len() - 1]) {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("...") {
                collect_identifier(rest.trim(), bindings);
            } else if let Some(colon) = part.find(':') {
                extract_pattern_bindings(part[colon + 1..].trim(), bindings);
            } else {
                extract_pattern_bindings(part, bindings);
            }
        }
    } else if value.starts_with('[') && value.ends_with(']') {
        for part in split_top_level(&value[1..value.len() - 1]) {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("...") {
                collect_identifier(rest.trim(), bindings);
            } else {
                extract_pattern_bindings(part, bindings);
            }
        }
    } else {
        collect_identifier(value, bindings);
    }
}

fn collect_identifier(value: &str, bindings: &mut FxHashSet<String>) {
    if is_identifier(value) {
        bindings.insert(value.to_compact_string());
    }
}

fn split_top_level(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (index, byte) in value.bytes().enumerate() {
        match byte {
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(&value[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(&value[start..]);
    parts
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_alphabetic() || first == '_' || first == '$')
        && chars
            .all(|character| character.is_alphanumeric() || character == '_' || character == '$')
}
