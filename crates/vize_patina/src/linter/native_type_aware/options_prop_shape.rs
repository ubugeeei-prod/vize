//! Type a template expression as a runtime Options API prop when Canon left
//! the free name as `any` so the instance check can still report `TS2339`.

use vize_s0::{String, cstr};

pub(super) fn options_props_object(text: &str) -> Option<&str> {
    let marker = "const __vize_options_props = (";
    let at = text.find(marker)?;
    let rest = text.get(at + marker.len()..)?.trim_start();
    if !rest.starts_with('{') {
        return None;
    }
    let end = matching_close(rest)?;
    rest.get(..=end)
}

pub(super) fn options_prop_annotation(object: Option<&str>, expr: &str) -> Option<String> {
    let name = expr.trim();
    if !is_identifier(name) || !object_has_depth1_key(object?, name) {
        return None;
    }
    Some(cstr!(
        "__VizeOptionsPropShape<typeof __vize_options_props>[\"{name}\"]"
    ))
}

fn object_has_depth1_key(object: &str, name: &str) -> bool {
    let bytes = object.as_bytes();
    let mut index = 0usize;
    let mut depth = 0i32;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'{' | b'[' | b'(' => {
                depth += 1;
                index += 1;
            }
            b'}' | b']' | b')' => {
                depth -= 1;
                index += 1;
            }
            quote @ (b'\'' | b'"' | b'`') => {
                let key_start = index + 1;
                index = skip_quoted(bytes, index, quote);
                if depth == 1
                    && quote != b'`'
                    && index > key_start
                    && object.get(key_start..index - 1) == Some(name)
                    && followed_by_colon(object, index)
                {
                    return true;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = object
                    .get(index..)
                    .unwrap_or_default()
                    .find('\n')
                    .map_or(bytes.len(), |at| index + at);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = object
                    .get(index + 2..)
                    .unwrap_or_default()
                    .find("*/")
                    .map_or(bytes.len(), |at| index + 2 + at + 2);
            }
            byte if depth == 1 && is_ident_start(byte) => {
                let start = index;
                index += 1;
                while bytes.get(index).copied().is_some_and(is_ident_continue) {
                    index += 1;
                }
                if object.get(start..index) == Some(name) && followed_by_colon(object, index) {
                    return true;
                }
            }
            _ => index += 1,
        }
    }
    false
}

fn matching_close(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut depth = 0i32;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'{' | b'[' | b'(' => {
                depth += 1;
                index += 1;
            }
            b'}' | b']' | b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            quote @ (b'\'' | b'"' | b'`') => index = skip_quoted(bytes, index, quote),
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = source
                    .get(index..)
                    .unwrap_or_default()
                    .find('\n')
                    .map_or(bytes.len(), |at| index + at);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = source
                    .get(index + 2..)
                    .unwrap_or_default()
                    .find("*/")
                    .map_or(bytes.len(), |at| index + 2 + at + 2);
            }
            _ => index += 1,
        }
    }
    None
}

fn skip_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut index = start + 1;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'\\' {
            index = (index + 2).min(bytes.len());
            continue;
        }
        if byte == quote {
            return index + 1;
        }
        index += 1;
    }
    bytes.len()
}

fn followed_by_colon(source: &str, index: usize) -> bool {
    source
        .get(index..)
        .unwrap_or_default()
        .trim_start()
        .starts_with(':')
}

fn is_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    is_ident_start(first) && bytes.all(is_ident_continue)
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

fn is_ident_continue(byte: u8) -> bool {
    is_ident_start(byte) || byte.is_ascii_digit()
}
