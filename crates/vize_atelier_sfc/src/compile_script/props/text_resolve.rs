//! Text-based prop type extraction.
//!
//! Public entry points for extracting prop types from a TypeScript type
//! definition, plus the string-splitting fallback used when AST-based
//! resolution does not apply.

use vize_carton::FxHashMap;
use vize_carton::{String, ToCompactString};

use super::ast_resolve::extract_prop_types_from_ast;
use super::runtime_type::{is_valid_identifier, ts_type_to_js_type, type_includes_top_level_null};
use super::types::PropTypeInfo;

/// Strip TypeScript comments from source while preserving string literals.
fn strip_ts_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = b'"';

    while i < bytes.len() {
        if in_string {
            if bytes[i] == string_char && (i == 0 || bytes[i - 1] != b'\\') {
                in_string = false;
            }
            result.push(bytes[i] as char);
            i += 1;
            continue;
        }

        match bytes[i] {
            b'\'' | b'"' | b'`' => {
                in_string = true;
                string_char = bytes[i];
                result.push(bytes[i] as char);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                // Line comment: skip until newline
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                // Block comment: skip until */
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 2; // skip */
                }
            }
            _ => {
                result.push(bytes[i] as char);
                i += 1;
            }
        }
    }
    result
}

/// Join multi-line type definitions where continuation lines start with `|` or `&`.
/// For example:
/// ```text
/// type?:
///     | 'input'
///     | 'text';
/// ```
/// becomes: `type?: | 'input' | 'text';`
fn join_union_continuation_lines(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result = String::with_capacity(input.len());
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') || trimmed.starts_with('&') {
            // Join to previous line with a space
            result.push(' ');
            result.push_str(trimmed);
        } else {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(line);
        }
    }
    result
}

/// Extract prop types from TypeScript type definition.
/// Returns a Vec to preserve definition order (important for matching Vue's output).
pub fn extract_prop_types_from_type(type_args: &str) -> Vec<(String, PropTypeInfo)> {
    extract_prop_types_from_type_with_context(type_args, None, None)
}

pub(crate) fn extract_prop_types_from_type_with_context(
    type_args: &str,
    interfaces: Option<&FxHashMap<String, String>>,
    type_aliases: Option<&FxHashMap<String, String>>,
) -> Vec<(String, PropTypeInfo)> {
    if let Some(props) = extract_prop_types_from_ast(type_args, interfaces, type_aliases) {
        return props;
    }

    extract_prop_types_from_type_text(type_args)
}

fn extract_prop_types_from_type_text(type_args: &str) -> Vec<(String, PropTypeInfo)> {
    let mut props = Vec::new();

    // Strip comments before parsing
    let stripped = strip_ts_comments(type_args);
    // Join multi-line union/intersection types (lines starting with | or &)
    let joined = join_union_continuation_lines(&stripped);
    let content = joined.trim();
    let content = if content.starts_with('{') && content.ends_with('}') {
        &content[1..content.len() - 1]
    } else {
        content
    };

    // Split by commas/semicolons/newlines in a single character pass. Keeping
    // `prev` avoids building a `Vec<char>` just to look behind for `=>`, which
    // used to allocate on every type-literal prop extraction.
    let mut depth: i32 = 0;
    let mut current = String::default();
    let mut prev = '\0';

    for c in content.chars() {
        match c {
            '{' | '<' | '(' | '[' => {
                depth += 1;
                current.push(c);
            }
            '}' | ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
                current.push(c);
            }
            '>' => {
                // Don't count `>` as closing angle bracket when preceded by `=` (arrow function `=>`)
                if prev == '=' {
                    current.push(c);
                } else {
                    if depth > 0 {
                        depth -= 1;
                    }
                    current.push(c);
                }
            }
            ',' | ';' if depth <= 0 => {
                extract_prop_type_info(&current, &mut props);
                current.clear();
                depth = 0;
            }
            '\n' if depth <= 0 => {
                // Don't split on newline if the current segment ends with ':' (type on next line)
                let trimmed_current = current.trim();
                if !trimmed_current.is_empty() && !trimmed_current.ends_with(':') {
                    extract_prop_type_info(&current, &mut props);
                    current.clear();
                    depth = 0;
                }
                // If ends with ':', keep accumulating (type continues on next line)
            }
            _ => current.push(c),
        }
        prev = c;
    }
    extract_prop_type_info(&current, &mut props);

    props
}

fn extract_prop_type_info(segment: &str, props: &mut Vec<(String, PropTypeInfo)>) {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return;
    }

    // Parse "name?: Type" or "name: Type"
    if let Some(colon_pos) = trimmed.find(':') {
        // Method-signature props (`onChange(e: E): void`, `update?(): T`)
        // have a `(` somewhere in the parameter list *before* this colon.
        // Detect by scanning the bytes up to `colon_pos` for `(`; on a hit,
        // recover the name from before that `(` and treat the prop as
        // `Function`-typed. Plain props (`name: Type`) skip this branch
        // because there's no `(` before the colon. (#967)
        let before_colon_bytes = &trimmed.as_bytes()[..colon_pos];
        if let Some(paren_pos) = before_colon_bytes.iter().position(|&b| b == b'(') {
            let before_paren = &trimmed[..paren_pos];
            let optional = before_paren.trim_end().ends_with('?');
            let name = before_paren.trim_end_matches('?').trim();
            if !name.is_empty() && is_valid_identifier(name) {
                let ts_type_str: String = (&trimmed[paren_pos..]).to_compact_string();
                if !props.iter().any(|(n, _)| n == name) {
                    props.push((
                        name.to_compact_string(),
                        PropTypeInfo {
                            js_type: "Function".to_compact_string(),
                            ts_type: Some(ts_type_str),
                            optional,
                            nullable: false,
                        },
                    ));
                }
            }
            return;
        }
        let name_part = &trimmed[..colon_pos];
        let type_part = &trimmed[colon_pos + 1..];

        // Per Vue's type-only inference, a property is `required: false` only
        // when the declaration carries the `?` optional modifier. `T |
        // undefined` (no `?`) is still required. (#967)
        let optional = name_part.ends_with('?');
        let nullable = type_includes_top_level_null(type_part);
        let name = name_part.trim().trim_end_matches('?').trim();

        if !name.is_empty() && is_valid_identifier(name) {
            let ts_type_str = type_part.trim().to_compact_string();
            let js_type = ts_type_to_js_type(&ts_type_str);
            // Avoid duplicates (intersection types may have overlapping props)
            if !props.iter().any(|(n, _)| n == name) {
                props.push((
                    name.to_compact_string(),
                    PropTypeInfo {
                        js_type,
                        ts_type: Some(ts_type_str),
                        optional,
                        nullable,
                    },
                ));
            }
        }
    }
}
