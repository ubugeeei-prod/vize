//! Lexical template bindings preserved while lowering syntax into peer graphs.

use vize_carton::{FxHashSet, String, ToCompactString};

const GENERATED_SCOPE_PREFIXES: [&str; 6] = [
    "_ctx.",
    "__props.",
    "$props.",
    "$setup.",
    "$data.",
    "$options.",
];

pub(super) fn pattern_bindings(pattern: &str) -> FxHashSet<String> {
    let mut bindings = FxHashSet::default();
    extract_pattern_bindings(pattern.trim(), &mut bindings);
    bindings
}

pub(super) fn strip_local_scope_prefixes(scopes: &[FxHashSet<String>], content: &str) -> String {
    if scopes.is_empty()
        || !GENERATED_SCOPE_PREFIXES
            .iter()
            .any(|prefix| content.contains(prefix))
    {
        return content.to_compact_string();
    }

    let mut result = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let mut modes = vec![ScanMode::Code {
        template_depth: None,
    }];
    let mut index = 0;
    while index < bytes.len() {
        match modes.last().copied().expect("root scan mode") {
            ScanMode::Template => {
                if bytes[index] == b'\\' {
                    index = copy_escaped(content, index, &mut result);
                } else if content[index..].starts_with("${") {
                    result.push_str("${");
                    index += 2;
                    modes.push(ScanMode::Code {
                        template_depth: Some(1),
                    });
                } else {
                    let character = copy_char(content, &mut index, &mut result);
                    if character == '`' {
                        modes.pop();
                    }
                }
                continue;
            }
            ScanMode::Code { template_depth } => {
                if matches!(bytes[index], b'\'' | b'"') {
                    index = copy_quoted(content, index, bytes[index], &mut result);
                    continue;
                }
                if content[index..].starts_with("//") {
                    index = copy_line_comment(content, index, &mut result);
                    continue;
                }
                if content[index..].starts_with("/*") {
                    index = copy_block_comment(content, index, &mut result);
                    continue;
                }
                if bytes[index] == b'/'
                    && likely_regex_start(content, index)
                    && let Some(end) = regex_literal_end(content, index)
                {
                    result.push_str(&content[index..end]);
                    index = end;
                    continue;
                }
                if bytes[index] == b'`' {
                    result.push('`');
                    index += 1;
                    modes.push(ScanMode::Template);
                    continue;
                }
                if let Some(mut depth) = template_depth {
                    if bytes[index] == b'{' {
                        depth += 1;
                        modes.pop();
                        modes.push(ScanMode::Code {
                            template_depth: Some(depth),
                        });
                    } else if bytes[index] == b'}' {
                        result.push('}');
                        index += 1;
                        if depth == 1 {
                            modes.pop();
                        } else {
                            modes.pop();
                            modes.push(ScanMode::Code {
                                template_depth: Some(depth - 1),
                            });
                        }
                        continue;
                    }
                }
            }
        }

        let mut stripped = false;
        if is_generated_prefix_boundary(content, index) {
            for prefix in GENERATED_SCOPE_PREFIXES {
                let prefix_bytes = prefix.as_bytes();
                if index + prefix_bytes.len() > bytes.len()
                    || &bytes[index..index + prefix_bytes.len()] != prefix_bytes
                {
                    continue;
                }
                let start = index + prefix_bytes.len();
                let mut end = start;
                while end < bytes.len() {
                    let character = content[end..]
                        .chars()
                        .next()
                        .expect("identifier scan remains on a character boundary");
                    if !is_identifier_continue(character) {
                        break;
                    }
                    end += character.len_utf8();
                }
                let identifier = &content[start..end];
                if !identifier.is_empty()
                    && scopes.iter().rev().any(|scope| scope.contains(identifier))
                {
                    result.push_str(identifier);
                    index = end;
                    stripped = true;
                    break;
                }
            }
        }
        if !stripped {
            copy_char(content, &mut index, &mut result);
        }
    }
    result
}

#[derive(Clone, Copy)]
enum ScanMode {
    Code { template_depth: Option<usize> },
    Template,
}

fn copy_char(content: &str, index: &mut usize, result: &mut String) -> char {
    let character = content[*index..]
        .chars()
        .next()
        .expect("index remains on a character boundary");
    result.push(character);
    *index += character.len_utf8();
    character
}

fn copy_escaped(content: &str, start: usize, result: &mut String) -> usize {
    let mut end = start + 1;
    if end < content.len() {
        let character = content[end..]
            .chars()
            .next()
            .expect("escape has a following character");
        end += character.len_utf8();
    }
    result.push_str(&content[start..end]);
    end
}

fn copy_quoted(content: &str, start: usize, quote: u8, result: &mut String) -> usize {
    let bytes = content.as_bytes();
    let mut end = start + 1;
    while end < bytes.len() {
        if bytes[end] == b'\\' {
            end += 1;
            if end < bytes.len() {
                let character = content[end..]
                    .chars()
                    .next()
                    .expect("escape remains on a character boundary");
                end += character.len_utf8();
            }
        } else {
            let current = bytes[end];
            let character = content[end..]
                .chars()
                .next()
                .expect("quote scan remains on a character boundary");
            end += character.len_utf8();
            if current == quote {
                break;
            }
        }
    }
    result.push_str(&content[start..end]);
    end
}

fn copy_line_comment(content: &str, start: usize, result: &mut String) -> usize {
    let end = content[start..]
        .find('\n')
        .map_or(content.len(), |offset| start + offset);
    result.push_str(&content[start..end]);
    end
}

fn copy_block_comment(content: &str, start: usize, result: &mut String) -> usize {
    let end = content[start + 2..]
        .find("*/")
        .map_or(content.len(), |offset| start + 2 + offset + 2);
    result.push_str(&content[start..end]);
    end
}

fn is_generated_prefix_boundary(content: &str, index: usize) -> bool {
    index == 0
        || content[..index]
            .chars()
            .next_back()
            .is_none_or(|previous| !is_identifier_continue(previous) && previous != '.')
}

fn is_identifier_continue(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '$')
}

fn likely_regex_start(content: &str, index: usize) -> bool {
    let before = content[..index].trim_end();
    let Some(previous) = before.chars().next_back() else {
        return true;
    };
    matches!(
        previous,
        '(' | '['
            | '{'
            | ':'
            | ';'
            | ','
            | '='
            | '!'
            | '?'
            | '&'
            | '|'
            | '+'
            | '-'
            | '*'
            | '%'
            | '^'
            | '~'
            | '<'
            | '>'
    ) || before
        .split(|character: char| !is_identifier_continue(character))
        .next_back()
        .is_some_and(|word| {
            matches!(
                word,
                "return"
                    | "case"
                    | "throw"
                    | "delete"
                    | "void"
                    | "typeof"
                    | "instanceof"
                    | "in"
                    | "of"
                    | "yield"
                    | "await"
            )
        })
}

fn regex_literal_end(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut index = start + 1;
    let mut in_class = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                index += 1;
                if index < bytes.len() {
                    let character = content[index..].chars().next()?;
                    index += character.len_utf8();
                }
            }
            b'[' => {
                in_class = true;
                index += 1;
            }
            b']' => {
                in_class = false;
                index += 1;
            }
            b'/' if !in_class => {
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
                    index += 1;
                }
                return Some(index);
            }
            b'\n' | b'\r' => return None,
            _ => {
                let character = content[index..].chars().next()?;
                index += character.len_utf8();
            }
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_nested_patterns_and_strips_only_lexical_bindings() {
        let scope = pattern_bindings("[{ id: dep = fallback }, version, ...rest]");
        assert!(scope.contains("dep"));
        assert!(scope.contains("version"));
        assert!(scope.contains("rest"));
        assert!(!scope.contains("id"));
        assert_eq!(
            strip_local_scope_prefixes(
                &[scope],
                "_ctx.route(_ctx.dep, _ctx.version, _ctx.external)"
            ),
            "_ctx.route(dep, version, _ctx.external)"
        );
    }

    #[test]
    fn rewrites_only_javascript_references_without_corrupting_literals() {
        let scope = pattern_bindings("dep, 項目");
        assert_eq!(
            strip_local_scope_prefixes(
                &[scope],
                r#"_ctx.route(_ctx.dep, _ctx.項目, `日本語 _ctx.dep ${_ctx.dep}`, "_ctx.dep", /_ctx\.dep/, /* _ctx.dep */ _ctx.external)"#,
            ),
            r#"_ctx.route(dep, 項目, `日本語 _ctx.dep ${dep}`, "_ctx.dep", /_ctx\.dep/, /* _ctx.dep */ _ctx.external)"#,
        );
    }
}
