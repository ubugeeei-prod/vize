//! Rewriting of reserved-name template props into `props["name"]` accesses.
//!
//! Vue allows template props whose names collide with TypeScript reserved
//! identifiers (`static`, `default`, `class`, ...). This module scans an
//! expression and rewrites bare references to such props into bracketed
//! `props[...]` accesses, while leaving string/regex literals, member
//! accesses, object property keys, and TypeScript `as` assertions untouched.

use super::super::helpers::is_reserved_identifier;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;

pub(crate) fn rewrite_reserved_template_prop(
    expression: &str,
    template_prop_names: &FxHashSet<String>,
) -> Option<String> {
    if template_prop_names.is_empty() {
        return None;
    }

    let bytes = expression.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut output = String::with_capacity(expression.len());
    let mut changed = false;

    while i < len {
        let current = bytes[i];

        if current == b'\'' || current == b'"' || current == b'`' {
            let end = skip_quoted_literal(bytes, i);
            output.push_str(&expression[i..end]);
            i = end;
            continue;
        }

        if current == b'/'
            && i + 1 < len
            && bytes[i + 1] != b'/'
            && bytes[i + 1] != b'*'
            && starts_regex_literal(bytes, i)
        {
            let end = skip_regex_literal(bytes, i);
            output.push_str(&expression[i..end]);
            i = end;
            continue;
        }

        if is_identifier_start(current) {
            let start = i;
            i += 1;
            while i < len && is_identifier_continue(bytes[i]) {
                i += 1;
            }
            let ident = &expression[start..i];
            if is_reserved_identifier(ident)
                && template_prop_names.contains(ident)
                && !is_property_access(bytes, start)
                && !is_object_property_key(bytes, i)
                && !is_typescript_as_assertion_operator(bytes, start, i, ident)
            {
                if is_object_shorthand(bytes, start, i) {
                    append!(output, "{ident}: props[\"{ident}\"]");
                } else {
                    append!(output, "props[\"{ident}\"]");
                }
                changed = true;
            } else {
                output.push_str(ident);
            }
            continue;
        }

        output.push(current as char);
        i += 1;
    }

    changed.then_some(output)
}

fn skip_quoted_literal(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut i = start + 1;
    while i < bytes.len() {
        let current = bytes[i];
        i += 1;
        if current == b'\\' {
            i = (i + 1).min(bytes.len());
            continue;
        }
        if current == quote {
            break;
        }
    }
    i
}

fn skip_regex_literal(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    let mut in_class = false;
    while i < bytes.len() {
        let current = bytes[i];
        i += 1;
        if current == b'\\' {
            i = (i + 1).min(bytes.len());
            continue;
        }
        match current {
            b'[' => in_class = true,
            b']' => in_class = false,
            b'/' if !in_class => break,
            _ => {}
        }
    }
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    i
}

fn starts_regex_literal(bytes: &[u8], slash: usize) -> bool {
    let Some(prev) = previous_significant_byte(bytes, slash) else {
        return true;
    };
    matches!(
        prev,
        b'(' | b'{'
            | b'['
            | b','
            | b':'
            | b';'
            | b'='
            | b'?'
            | b'!'
            | b'&'
            | b'|'
            | b'+'
            | b'-'
            | b'*'
            | b'%'
            | b'^'
            | b'~'
            | b'<'
            | b'>'
    )
}

fn previous_significant_byte(bytes: &[u8], before: usize) -> Option<u8> {
    bytes[..before]
        .iter()
        .rev()
        .copied()
        .find(|b| !b.is_ascii_whitespace())
}

fn next_significant_byte(bytes: &[u8], after: usize) -> Option<u8> {
    bytes[after..]
        .iter()
        .copied()
        .find(|b| !b.is_ascii_whitespace())
}

fn is_property_access(bytes: &[u8], ident_start: usize) -> bool {
    previous_significant_byte(bytes, ident_start) == Some(b'.')
}

fn is_object_property_key(bytes: &[u8], ident_end: usize) -> bool {
    next_significant_byte(bytes, ident_end) == Some(b':')
}

fn is_object_shorthand(bytes: &[u8], ident_start: usize, ident_end: usize) -> bool {
    matches!(
        previous_significant_byte(bytes, ident_start),
        Some(b'{') | Some(b',')
    ) && matches!(
        next_significant_byte(bytes, ident_end),
        Some(b'}') | Some(b',')
    )
}

fn is_typescript_as_assertion_operator(
    bytes: &[u8],
    ident_start: usize,
    ident_end: usize,
    ident: &str,
) -> bool {
    if ident != "as" {
        return false;
    }

    let Some(prev) = previous_significant_byte(bytes, ident_start) else {
        return false;
    };
    let Some(next) = next_significant_byte(bytes, ident_end) else {
        return false;
    };

    let expression_before_as = prev.is_ascii_alphanumeric()
        || matches!(prev, b'_' | b'$' | b')' | b']' | b'}' | b'\'' | b'"' | b'`');
    let type_after_as =
        is_identifier_start(next) || matches!(next, b'{' | b'[' | b'(' | b'\'' | b'"');

    expression_before_as && type_after_as
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}
