/// Check if a cursor offset is inside a Vue template expression.
///
/// This covers mustache interpolations and Vue directive attribute values, but
/// deliberately excludes plain text nodes and static attribute values.
pub(crate) fn is_in_vue_template_expression(content: &str, offset: usize) -> bool {
    if content.is_empty() {
        return false;
    }

    let mut offset = offset.min(content.len());
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }

    is_in_mustache_expression(content, offset)
        || is_in_vue_directive_attribute_value(content, offset)
}

fn is_in_mustache_expression(content: &str, offset: usize) -> bool {
    let before = &content[..offset];
    let Some(mustache_start) = before.rfind("{{") else {
        return false;
    };

    let closed_before_cursor = before
        .rfind("}}")
        .is_some_and(|mustache_end| mustache_end > mustache_start);
    if closed_before_cursor {
        return false;
    }

    content[offset..].contains("}}")
}

fn is_in_vue_directive_attribute_value(content: &str, offset: usize) -> bool {
    let bytes = content.as_bytes();
    for (tag_start, _) in content[..offset].match_indices('<').rev() {
        let name_start = tag_start + 1;
        if matches!(bytes.get(name_start), Some(b'/' | b'!' | b'?')) {
            continue;
        }
        let mut name_end = name_start;
        while name_end < bytes.len()
            && (bytes[name_end].is_ascii_alphanumeric() || matches!(bytes[name_end], b'-' | b'_'))
        {
            name_end += 1;
        }
        if name_start == name_end {
            continue;
        }

        let mut quote = None;
        let mut quote_start = None;
        let mut pos = name_end;
        while pos < offset {
            let byte = bytes[pos];
            if let Some(open_quote) = quote {
                if byte == open_quote {
                    quote = None;
                    quote_start = None;
                }
            } else if byte == b'"' || byte == b'\'' {
                quote = Some(byte);
                quote_start = Some(pos);
            } else if byte == b'>' {
                break;
            }
            pos += 1;
        }

        if pos < offset || quote.is_none() {
            continue;
        }
        let Some(quote_start) = quote_start else {
            continue;
        };
        return directive_attribute_name_before_quote(content, quote_start)
            .is_some_and(is_vue_expression_attribute);
    }

    false
}

fn directive_attribute_name_before_quote(content: &str, quote_start: usize) -> Option<&str> {
    let bytes = content.as_bytes();
    let mut pos = quote_start;
    while pos > 0 && bytes[pos - 1].is_ascii_whitespace() {
        pos -= 1;
    }
    if pos == 0 || bytes[pos - 1] != b'=' {
        return None;
    }
    pos -= 1;

    while pos > 0 && bytes[pos - 1].is_ascii_whitespace() {
        pos -= 1;
    }
    let attr_end = pos;
    while pos > 0 {
        let byte = bytes[pos - 1];
        if byte.is_ascii_whitespace() || matches!(byte, b'<' | b'>' | b'/') {
            break;
        }
        pos -= 1;
    }

    Some(&content[pos..attr_end])
}

fn is_vue_expression_attribute(attr_name: &str) -> bool {
    attr_name.starts_with(':')
        || attr_name.starts_with('@')
        || attr_name.starts_with('#')
        || attr_name.starts_with("v-")
}
