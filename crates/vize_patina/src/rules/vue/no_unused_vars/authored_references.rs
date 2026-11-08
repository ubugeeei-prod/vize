fn authored_element_is_empty(source: &str, open: usize, close: usize) -> bool {
    let Some(opening) = source.get(open..=close) else {
        return false;
    };
    if opening.trim_end_matches('>').trim_end().ends_with('/') {
        return true;
    }
    let Some(tag) = opening
        .strip_prefix('<')
        .and_then(|opening| opening.split_ascii_whitespace().next())
        .map(|tag| tag.trim_end_matches('>'))
        .filter(|tag| !tag.is_empty())
    else {
        return false;
    };
    let closing_tag = cstr!("</{tag}>");
    source
        .get(close + 1..)
        .is_some_and(|tail| tail.trim_start().starts_with(closing_tag.as_str()))
}
fn authored_identifier_reference_present(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(start, matched)| {
        let end = start + matched.len();
        !source
            .as_bytes()
            .get(start.wrapping_sub(1))
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.'))
            && !source
                .as_bytes()
                .get(end)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
    })
}

fn authored_opening_directive_uses(source: &str, name: &str, offset: u32) -> bool {
    let offset = offset as usize;
    let Some(open) = source.get(..offset).and_then(|prefix| prefix.rfind('<')) else {
        return false;
    };
    let bytes = source.as_bytes();
    let mut cursor = open + 1;
    while cursor < bytes.len() {
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes
            .get(cursor)
            .is_none_or(|byte| matches!(byte, b'>' | b'/'))
        {
            break;
        }
        let attr_start = cursor;
        while bytes
            .get(cursor)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>' | b'/'))
        {
            cursor += 1;
        }
        let attr = source.get(attr_start..cursor).unwrap_or_default();
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let Some(quote @ (b'\'' | b'"')) = bytes.get(cursor).copied() else {
            continue;
        };
        cursor += 1;
        let value_start = cursor;
        while bytes.get(cursor).is_some_and(|byte| *byte != quote) {
            cursor += 1;
        }
        let value_end = cursor;
        cursor += usize::from(bytes.get(cursor) == Some(&quote));
        if matches!(attr, "v-for" | "v-slot")
            || attr.starts_with("v-slot:")
            || attr.starts_with('#')
            || !matches!(attr.as_bytes().first(), Some(b':' | b'@' | b'#'))
                && !attr.starts_with("v-")
        {
            continue;
        }
        if source
            .get(value_start..value_end)
            .is_some_and(|value| identifier_token_present(value, name))
        {
            return true;
        }
    }
    false
}

fn identifier_token_present(source: &str, name: &str) -> bool {
    if let Some(arrow) = source.find("=>")
        && arrow_parameter_binds(source.get(..arrow).unwrap_or_default(), name)
    {
        return false;
    }
    source.match_indices(name).any(|(start, matched)| {
        let end = start + matched.len();
        !source
            .as_bytes()
            .get(start.wrapping_sub(1))
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
            && !source
                .as_bytes()
                .get(end)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
    })
}

fn arrow_parameter_binds(prefix: &str, name: &str) -> bool {
    let prefix = prefix.trim_end();
    if prefix == name {
        return true;
    }
    let Some(parameters) = prefix
        .strip_suffix(')')
        .and_then(|text| text.rsplit_once('('))
    else {
        return false;
    };
    parameters
        .1
        .split(',')
        .map(str::trim)
        .any(|parameter| parameter == name)
}
