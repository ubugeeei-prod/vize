use vize_carton::{String, ToCompactString, cstr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DestructureBinding {
    pub(super) local: String,
    pub(super) path: String,
}

pub(super) fn parse_destructure_bindings(pattern: &str) -> std::vec::Vec<DestructureBinding> {
    let mut bindings = std::vec::Vec::new();
    parse_pattern(
        strip_default(pattern.trim()),
        String::default(),
        &mut bindings,
    );
    bindings
}

pub(super) fn parse_destructure_names(pattern: &str) -> std::vec::Vec<String> {
    parse_destructure_bindings(pattern)
        .into_iter()
        .map(|binding| binding.local)
        .collect()
}

fn parse_pattern(pattern: &str, prefix: String, bindings: &mut std::vec::Vec<DestructureBinding>) {
    let pattern = strip_wrapping_parens(pattern.trim());

    if pattern.starts_with('{') && pattern.ends_with('}') {
        parse_object_pattern(pattern, prefix, bindings);
    } else if pattern.starts_with('[') && pattern.ends_with(']') {
        parse_array_pattern(pattern, prefix, bindings);
    } else if is_valid_ident(pattern) {
        bindings.push(DestructureBinding {
            local: pattern.to_compact_string(),
            path: prefix,
        });
    }
}

fn parse_object_pattern(
    pattern: &str,
    prefix: String,
    bindings: &mut std::vec::Vec<DestructureBinding>,
) {
    let inner = &pattern[1..pattern.len() - 1];
    for part in split_top_level(inner) {
        let part = part.trim();
        if part.is_empty() || part.starts_with("...") {
            continue;
        }

        if let Some(colon) = find_top_level_char(part, ':') {
            let key = part[..colon].trim();
            let value = strip_default(part[colon + 1..].trim());
            let Some(segment) = object_path_segment(key) else {
                continue;
            };
            parse_pattern(value, cstr!("{prefix}{segment}"), bindings);
            continue;
        }

        let name = strip_default(part);
        if is_valid_ident(name) {
            bindings.push(DestructureBinding {
                local: name.to_compact_string(),
                path: cstr!("{prefix}.{}", name),
            });
        }
    }
}

fn parse_array_pattern(
    pattern: &str,
    prefix: String,
    bindings: &mut std::vec::Vec<DestructureBinding>,
) {
    let inner = &pattern[1..pattern.len() - 1];
    for (index, part) in split_top_level(inner).into_iter().enumerate() {
        let part = part.trim();
        if part.is_empty() || part.starts_with("...") {
            continue;
        }
        parse_pattern(strip_default(part), cstr!("{prefix}[{index}]"), bindings);
    }
}

fn strip_default(pattern: &str) -> &str {
    if let Some(index) = find_top_level_char(pattern, '=') {
        pattern[..index].trim()
    } else {
        pattern.trim()
    }
}

fn strip_wrapping_parens(pattern: &str) -> &str {
    if pattern.starts_with('(') && pattern.ends_with(')') && matching_outer_pair(pattern, '(', ')')
    {
        pattern[1..pattern.len() - 1].trim()
    } else {
        pattern
    }
}

fn object_path_segment(key: &str) -> Option<String> {
    if is_valid_ident(key) {
        return Some(cstr!(".{key}"));
    }

    if let Some(unquoted) = strip_string_literal(key) {
        return Some(cstr!("[\"{}\"]", escape_js_string_literal(unquoted)));
    }

    if key.parse::<usize>().is_ok() {
        return Some(cstr!("[{key}]"));
    }

    None
}

fn strip_string_literal(value: &str) -> Option<&str> {
    let quote = value.as_bytes().first().copied()?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    if value.as_bytes().last().copied()? != quote {
        return None;
    }
    Some(&value[1..value.len() - 1])
}

fn escape_js_string_literal(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn split_top_level(s: &str) -> std::vec::Vec<&str> {
    let mut parts = std::vec::Vec::new();
    let mut depth = 0i32;
    let mut quote = None;
    let mut start = 0usize;
    let mut prev = '\0';

    for (index, ch) in s.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
        prev = ch;
    }

    parts.push(&s[start..]);
    parts
}

fn find_top_level_char(s: &str, needle: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';

    for (index, ch) in s.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            _ if ch == needle && depth == 0 => return Some(index),
            _ => {}
        }
        prev = ch;
    }

    None
}

fn matching_outer_pair(s: &str, open: char, close: char) -> bool {
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';

    for (index, ch) in s.char_indices() {
        if let Some(open_quote) = quote {
            if ch == open_quote && prev != '\\' {
                quote = None;
            }
            prev = ch;
            continue;
        }

        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            _ if ch == open => depth += 1,
            _ if ch == close => {
                depth -= 1;
                if depth == 0 && index + ch.len_utf8() < s.len() {
                    return false;
                }
            }
            _ => {}
        }
        prev = ch;
    }

    depth == 0
}

fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

#[cfg(test)]
mod tests {
    use super::parse_destructure_bindings;

    #[test]
    fn parses_object_aliases_and_nested_paths() {
        let bindings = parse_destructure_bindings(
            r#"{ id, name: label, user: { id: userId }, meta: { count: total = 0 }, "data-id": dataId }"#,
        );

        let pairs = bindings
            .iter()
            .map(|binding| (binding.local.as_str(), binding.path.as_str()))
            .collect::<std::vec::Vec<_>>();

        assert_eq!(
            pairs,
            vec![
                ("id", ".id"),
                ("label", ".name"),
                ("userId", ".user.id"),
                ("total", ".meta.count"),
                ("dataId", "[\"data-id\"]"),
            ]
        );
    }

    #[test]
    fn parses_array_aliases_and_nested_objects() {
        let bindings = parse_destructure_bindings("[first, { id: secondId }, third = fallback]");

        let pairs = bindings
            .iter()
            .map(|binding| (binding.local.as_str(), binding.path.as_str()))
            .collect::<std::vec::Vec<_>>();

        assert_eq!(
            pairs,
            vec![("first", "[0]"), ("secondId", "[1].id"), ("third", "[2]"),]
        );
    }
}
