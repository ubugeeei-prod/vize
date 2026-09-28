pub(super) fn source_contains_interpolation_ampersand(source: &str) -> bool {
    let mut rest = source;
    while let Some(start) = rest.find("{{") {
        let Some(after_open) = rest.get(start + 2..) else {
            break;
        };
        rest = after_open;
        let Some(end) = rest.find("}}") else {
            break;
        };
        if rest
            .get(..end)
            .is_some_and(|expression| expression.contains('&'))
        {
            return true;
        }
        let Some(after_close) = rest.get(end + 2..) else {
            break;
        };
        rest = after_close;
    }
    false
}

pub(super) fn source_contains_expression_ampersand(source: &str) -> bool {
    if !source.as_bytes().contains(&b'&') {
        return false;
    }
    source_contains_interpolation_ampersand(source) || source_contains_directive_ampersand(source)
}

fn source_contains_directive_ampersand(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;
    while let Some(tag_start) = super::find_byte(bytes, index, b'<') {
        let name_start = tag_start + 1;
        let name_end = super::scan_tag_name(bytes, name_start);
        if name_end == name_start {
            index = name_start;
            continue;
        }
        let tag_end = super::scan_tag_end(bytes, name_end);
        if tag_contains_directive_ampersand(bytes, name_end, tag_end) {
            return true;
        }
        index = tag_end;
    }
    false
}

fn tag_contains_directive_ampersand(bytes: &[u8], start: usize, end: usize) -> bool {
    let mut index = start;
    while index < end {
        while bytes
            .get(index)
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'/')
        {
            index += 1;
        }
        let name_start = index;
        while bytes
            .get(index)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'/' | b'>'))
        {
            index += 1;
        }
        if index == name_start {
            index += 1;
            continue;
        }
        let Some(name) = bytes.get(name_start..index) else {
            return false;
        };
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes.get(index) != Some(&b'=') {
            continue;
        }
        index += 1;
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        let quote = bytes
            .get(index)
            .copied()
            .filter(|byte| matches!(byte, b'\'' | b'"'));
        if quote.is_some() {
            index += 1;
        }
        let value_start = index;
        while bytes.get(index).is_some_and(|byte| {
            quote.map_or(!byte.is_ascii_whitespace() && *byte != b'>', |quote| {
                *byte != quote
            })
        }) {
            index += 1;
        }
        if directive_name(name)
            && bytes
                .get(value_start..index)
                .is_some_and(|value| value.contains(&b'&'))
        {
            return true;
        }
        if let Some(quote) = quote {
            index += usize::from(bytes.get(index) == Some(&quote));
        }
    }
    false
}

fn directive_name(name: &[u8]) -> bool {
    name.starts_with(b":")
        || name.starts_with(b"@")
        || name.starts_with(b"#")
        || name.starts_with(b".")
        || name.starts_with(b"v-")
}
