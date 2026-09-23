//! Type a template expression as a runtime Options API prop when Canon left
//! the free name as `any` so the instance check can still report `TS2339`.

pub(super) fn options_props_object(text: &str) -> Option<&str> {
    let marker = "const __vize_options_props = (";
    let at = text.find(marker)?;
    let rest = text[at + marker.len()..].trim_start();
    if !rest.starts_with('{') {
        return None;
    }
    let end = matching_close(rest)?;
    Some(&rest[..=end])
}

pub(super) fn options_prop_annotation(object: Option<&str>, expr: &str) -> Option<String> {
    let name = expr.trim();
    if !is_identifier(name) || !object_has_depth1_key(object?, name) {
        return None;
    }
    Some(format!(
        "__VizeOptionsPropShape<typeof __vize_options_props>[\"{name}\"]"
    ))
}

fn object_has_depth1_key(object: &str, name: &str) -> bool {
    let bytes = object.as_bytes();
    let mut index = 0usize;
    let mut depth = 0i32;
    while index < bytes.len() {
        match bytes[index] {
            b'{' | b'[' | b'(' => {
                depth += 1;
                index += 1;
            }
            b'}' | b']' | b')' => {
                depth -= 1;
                index += 1;
            }
            b'\'' | b'"' | b'`' => {
                let quote = bytes[index];
                let key_start = index + 1;
                index = skip_quoted(bytes, index);
                if depth == 1
                    && quote != b'`'
                    && index > key_start
                    && &object[key_start..index - 1] == name
                    && followed_by_colon(object, index)
                {
                    return true;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = object[index..]
                    .find('\n')
                    .map_or(bytes.len(), |at| index + at);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = object[index + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |at| index + 2 + at + 2);
            }
            byte if depth == 1 && is_ident_start(byte) => {
                let start = index;
                index += 1;
                while index < bytes.len() && is_ident_continue(bytes[index]) {
                    index += 1;
                }
                if &object[start..index] == name && followed_by_colon(object, index) {
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
    while index < bytes.len() {
        match bytes[index] {
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
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = source[index..]
                    .find('\n')
                    .map_or(bytes.len(), |at| index + at);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = source[index + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |at| index + 2 + at + 2);
            }
            _ => index += 1,
        }
    }
    None
}

fn skip_quoted(bytes: &[u8], start: usize) -> usize {
    let quote = bytes[start];
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
            continue;
        }
        if bytes[index] == quote {
            return index + 1;
        }
        index += 1;
    }
    bytes.len()
}

fn followed_by_colon(source: &str, index: usize) -> bool {
    source[index..].trim_start().starts_with(':')
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
