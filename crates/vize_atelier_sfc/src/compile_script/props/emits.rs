//! Emit name extraction from `defineEmits` type definitions.

use vize_carton::{String, ToCompactString};

/// Extract emit names from TypeScript type definition
pub fn extract_emit_names_from_type(type_args: &str) -> Vec<String> {
    let mut emits = Vec::new();

    // First, try Vue 3.3+ shorthand format:
    //   { change: [value: string]; submit: []; update: [id: number] }
    // Property names before `:` followed by `[` are event names
    let trimmed = type_args.trim();
    let is_shorthand = trimmed.starts_with('{')
        && trimmed.contains('[')
        && !trimmed.contains("(e:")
        && !trimmed.contains("(event:");

    if is_shorthand {
        // Extract property names from { name: [...], name: [...] } format
        let inner = if trimmed.starts_with('{') && trimmed.ends_with('}') {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        // Split by lines or semicolons and extract property names.
        for segment in inner.split([';', '\n']) {
            if let Some(name) = extract_emit_shorthand_key(segment) {
                emits.push(name);
            }
        }

        if !emits.is_empty() {
            return emits;
        }
    }

    extract_call_signature_emit_names(type_args, &mut emits);
    emits
}

fn extract_call_signature_emit_names(type_args: &str, emits: &mut Vec<String>) {
    let bytes = type_args.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'(' {
            i += 1;
            continue;
        }

        let Some(close) = find_matching_paren(type_args, i) else {
            i += 1;
            continue;
        };

        if is_emit_call_signature(type_args, i, close) {
            let params = &type_args[i + 1..close];
            if let Some(first_param) = first_parameter(params)
                && let Some(type_annotation) = parameter_type_annotation(first_param)
            {
                extract_string_literals(type_annotation, emits);
            }
        }

        i = close + 1;
    }
}

fn find_matching_paren(input: &str, open: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut depth = 0;
    let mut i = open;
    let mut quote: Option<u8> = None;

    while i < bytes.len() {
        let byte = bytes[i];
        if let Some(quote_byte) = quote {
            if byte == b'\\' {
                i += 2;
                continue;
            }
            if byte == quote_byte {
                quote = None;
            }
            i += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

fn is_emit_call_signature(input: &str, open: usize, close: usize) -> bool {
    let after = input[close + 1..].trim_start();
    if after.starts_with(':') {
        return true;
    }

    // Direct function type form: defineEmits<(e: 'change') => void>().
    after.starts_with("=>") && input[..open].trim().is_empty()
}

fn first_parameter(params: &str) -> Option<&str> {
    let mut depth = 0;
    let mut quote: Option<char> = None;
    let mut prev_escape = false;

    for (idx, ch) in params.char_indices() {
        if let Some(quote_char) = quote {
            if prev_escape {
                prev_escape = false;
                continue;
            }
            if ch == '\\' {
                prev_escape = true;
                continue;
            }
            if ch == quote_char {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' if depth > 0 => depth -= 1,
            ',' if depth == 0 => return Some(params[..idx].trim()),
            _ => {}
        }
    }

    let first = params.trim();
    (!first.is_empty()).then_some(first)
}

fn parameter_type_annotation(param: &str) -> Option<&str> {
    let mut depth = 0;
    let mut quote: Option<char> = None;
    let mut prev_escape = false;

    for (idx, ch) in param.char_indices() {
        if let Some(quote_char) = quote {
            if prev_escape {
                prev_escape = false;
                continue;
            }
            if ch == '\\' {
                prev_escape = true;
                continue;
            }
            if ch == quote_char {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' if depth > 0 => depth -= 1,
            ':' if depth == 0 => return Some(param[idx + 1..].trim()),
            _ => {}
        }
    }

    None
}

fn extract_string_literals(input: &str, output: &mut Vec<String>) {
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let quote = bytes[i];
        if !matches!(quote, b'\'' | b'"') {
            i += 1;
            continue;
        }

        let mut literal = String::default();
        i += 1;
        while i < bytes.len() {
            let byte = bytes[i];
            if byte == b'\\' {
                if i + 1 < bytes.len() {
                    literal.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
                break;
            }
            if byte == quote {
                if !literal.is_empty() {
                    output.push(literal);
                }
                i += 1;
                break;
            }
            literal.push(byte as char);
            i += 1;
        }
    }
}

fn extract_emit_shorthand_key(segment: &str) -> Option<String> {
    let seg = segment.trim();
    if seg.is_empty() || seg.starts_with("...") {
        return None;
    }

    let bytes = seg.as_bytes();
    let first = *bytes.first()?;
    if matches!(first, b'\'' | b'"' | b'`') {
        let quote = first;
        let mut key = String::default();
        let mut i = 1;
        while i < bytes.len() {
            let c = bytes[i];
            if c == b'\\' {
                if i + 1 < bytes.len() {
                    key.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
                return None;
            }
            if c == quote {
                let rest = seg[i + 1..].trim_start();
                if rest.starts_with(':') && !key.is_empty() {
                    return Some(key);
                }
                return None;
            }
            key.push(c as char);
            i += 1;
        }
        return None;
    }

    let mut colon_pos = None;
    for (idx, ch) in seg.char_indices() {
        if ch == ':' {
            colon_pos = Some(idx);
            break;
        }
    }
    let name = seg[..colon_pos?].trim().trim_end_matches('?').trim();
    if !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '$')
    {
        Some(name.to_compact_string())
    } else {
        None
    }
}
