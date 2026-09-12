use vize_s0::String;

use crate::ExpressionNode;

pub(super) fn strip_outer_pair(input: &str, open: char, close: char) -> Option<&str> {
    let input = input.trim();
    if !input.starts_with(open) || !input.ends_with(close) {
        return None;
    }
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';
    for (index, ch) in input.char_indices() {
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
                if depth == 0 && index + ch.len_utf8() != input.len() {
                    return None;
                }
            }
            _ => {}
        }
        prev = ch;
    }
    (depth == 0).then_some(&input[open.len_utf8()..input.len() - close.len_utf8()])
}

pub(super) fn split_top_level_commas(input: &str) -> std::vec::Vec<&str> {
    split_top_level_char(input, ',')
}

pub(super) fn split_top_level_colon(input: &str) -> Option<(&str, &str)> {
    let index = find_top_level_char(input, ':')?;
    Some((&input[..index], &input[index + 1..]))
}

pub(super) fn split_top_level_char(input: &str, needle: char) -> std::vec::Vec<&str> {
    let mut parts = std::vec::Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';
    for (index, ch) in input.char_indices() {
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
            _ if ch == needle && depth == 0 => {
                parts.push(input[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
        prev = ch;
    }
    parts.push(input[start..].trim());
    parts
}

pub(super) fn find_top_level_char(input: &str, needle: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';
    for (index, ch) in input.char_indices() {
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

pub(super) fn split_top_level_keyword<'a>(
    input: &'a str,
    keyword: &str,
) -> Option<(&'a str, &'a str)> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut prev = '\0';
    for (index, ch) in input.char_indices() {
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
            _ if depth == 0 && input[index..].starts_with(keyword) => {
                let before = input[..index].chars().last();
                let after_index = index + keyword.len();
                let after = input[after_index..].chars().next();
                if before.is_some_and(is_identifier_continue)
                    || after.is_some_and(is_identifier_continue)
                {
                    prev = ch;
                    continue;
                }
                return Some((input[..index].trim(), input[after_index..].trim()));
            }
            _ => {}
        }
        prev = ch;
    }
    None
}

pub(super) fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

pub(super) fn split_top_level_or(expr: &str) -> Option<std::vec::Vec<&str>> {
    let mut parts = std::vec::Vec::new();
    let bytes = expr.as_bytes();
    let mut start = 0usize;
    let mut depth = 0u32;
    let mut quote: Option<u8> = None;
    let mut escape = false;
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(active) = quote {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == active {
                quote = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' | b'`' => quote = Some(byte),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b'|' if depth == 0
                && bytes.get(index.wrapping_sub(1)) != Some(&b'|')
                && bytes.get(index + 1) != Some(&b'|') =>
            {
                parts.push(expr[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    if parts.is_empty() {
        return None;
    }
    parts.push(expr[start..].trim());
    Some(parts)
}

pub(super) fn expression_source(exp: &ExpressionNode<'_>, source: &str) -> String {
    match exp {
        ExpressionNode::Simple(simple) => simple.content.into(),
        ExpressionNode::Compound(compound) => String::new(compound.loc.span.slice(source)),
    }
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}
