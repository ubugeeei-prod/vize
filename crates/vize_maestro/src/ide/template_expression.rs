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

/// Check if a cursor offset completes a *member* of the expression to its left,
/// i.e. the partial name under the caret is introduced by `.`, `?.`, or `!.`.
///
/// Member access is the one template position whose answer only the type checker
/// knows: the structural binding list cannot enumerate the properties of an
/// arbitrary type (#3911). Identifier positions keep the structural answer, so
/// this predicate is what separates the two routes.
///
/// Deliberately not `CursorContext::MemberAccess`, which only recognizes a caret
/// sitting directly on the `.`. A completion request arrives with the member
/// name partially typed (`it.na|`, `theme.notFound?.co|`), and its receiver may
/// end in `?`/`!`, both of which that detector reports as an identifier. Widening
/// it would change what hover, definition, and references see at the same
/// position, so the routing question is answered here instead.
#[cfg(feature = "native")]
pub(crate) fn is_at_member_access_position(content: &str, offset: usize) -> bool {
    if content.is_empty() {
        return false;
    }

    let mut offset = offset.min(content.len());
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }

    let bytes = content.as_bytes();
    let mut pos = offset;
    // Step back over the partial member name already typed.
    while pos > 0 && is_member_name_byte(bytes[pos - 1]) {
        pos -= 1;
    }
    // A digit-led token is a numeric literal fragment (`1.5`), not a member.
    if pos < offset && bytes[pos].is_ascii_digit() {
        return false;
    }
    while pos > 0 && bytes[pos - 1].is_ascii_whitespace() {
        pos -= 1;
    }
    if pos == 0 || bytes[pos - 1] != b'.' {
        return false;
    }
    // `...rest` spreads an expression rather than reading a member off it.
    pos -= 1;
    pos == 0 || bytes[pos - 1] != b'.'
}

#[cfg(feature = "native")]
fn is_member_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
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

#[cfg(all(test, feature = "native"))]
mod member_access_tests {
    use super::is_at_member_access_position;

    /// Offset of the caret marker `|`, with the marker removed.
    fn at_caret(marked: &str) -> (String, usize) {
        let offset = marked.find('|').expect("test input needs a `|` caret");
        (marked.replace('|', ""), offset)
    }

    fn is_member_access(marked: &str) -> bool {
        let (content, offset) = at_caret(marked);
        is_at_member_access_position(&content, offset)
    }

    #[test]
    fn member_access_covers_dot_optional_and_non_null_chains() {
        assert!(is_member_access("{{ it.| }}"));
        assert!(is_member_access("{{ it.na| }}"));
        assert!(is_member_access("{{ theme.notFound?.co| }}"));
        assert!(is_member_access("{{ user!.na| }}"));
        assert!(is_member_access("{{ a.b.c.| }}"));
        assert!(is_member_access("{{ items[0].| }}"));
        assert!(is_member_access("{{ f().| }}"));
        assert!(is_member_access("{{ it.$pr| }}"));
        assert!(is_member_access("{{ it._pr| }}"));
    }

    #[test]
    fn identifier_positions_are_not_member_access() {
        assert!(!is_member_access("{{ | }}"));
        assert!(!is_member_access("{{ cou| }}"));
        assert!(!is_member_access("{{ val| }}"));
        assert!(!is_member_access("{{ it.name + val| }}"));
        assert!(!is_member_access("{{ f(arg| ) }}"));
    }

    #[test]
    fn spreads_and_numeric_literals_are_not_member_access() {
        assert!(!is_member_access("{{ f(...arg| ) }}"));
        assert!(!is_member_access("{{ 1.5| }}"));
    }
}
