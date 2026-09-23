//! Petite-vue `v-scope` / `createApp({...})` object ranges used for standalone HTML definitions.

use super::helpers;

pub(super) fn v_scope_value_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let bytes = content.as_bytes();
    let mut search_start = 0;

    while let Some(relative) = content
        .get(search_start..)
        .and_then(|rest| rest.find("v-scope"))
    {
        let attr_start = search_start + relative;
        let mut pos = attr_start + "v-scope".len();

        if bytes
            .get(pos)
            .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            search_start = pos;
            continue;
        }

        while bytes.get(pos).is_some_and(u8::is_ascii_whitespace) {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'=') {
            search_start = pos;
            continue;
        }
        pos += 1;
        while bytes.get(pos).is_some_and(u8::is_ascii_whitespace) {
            pos += 1;
        }

        let Some(&quote) = bytes.get(pos) else {
            break;
        };
        if quote != b'"' && quote != b'\'' {
            search_start = pos + 1;
            continue;
        }
        let value_start = pos + 1;
        let Some(relative_end) = content
            .get(value_start..)
            .and_then(|rest| rest.find(quote as char))
        else {
            break;
        };
        let value_end = value_start + relative_end;
        ranges.push((value_start, value_end));
        search_start = value_end + 1;
    }

    ranges
}

pub(super) fn create_app_object_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let bytes = content.as_bytes();
    let mut search_start = 0;

    while let Some(relative) = content
        .get(search_start..)
        .and_then(|rest| rest.find("createApp"))
    {
        let name_start = search_start + relative;
        if name_start
            .checked_sub(1)
            .and_then(|i| bytes.get(i))
            .is_some_and(|&b| helpers::is_word_char(b))
        {
            search_start = name_start + "createApp".len();
            continue;
        }

        let mut pos = name_start + "createApp".len();
        if bytes.get(pos).is_some_and(|&b| helpers::is_word_char(b)) {
            search_start = pos;
            continue;
        }

        while bytes.get(pos).is_some_and(u8::is_ascii_whitespace) {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'(') {
            search_start = pos;
            continue;
        }
        pos += 1;
        while bytes.get(pos).is_some_and(u8::is_ascii_whitespace) {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'{') {
            search_start = pos;
            continue;
        }

        if let Some(end) = find_matching_byte(content, pos, b'{', b'}') {
            ranges.push((pos + 1, end));
            search_start = end + 1;
        } else {
            break;
        }
    }

    ranges
}

pub(super) fn find_object_property_key_in_range(
    content: &str,
    range: (usize, usize),
    word: &str,
    cursor_start: usize,
) -> Option<usize> {
    let (start, end) = range;
    let bytes = content.as_bytes();
    let mut search_start = start;

    while search_start < end {
        let relative = content.get(search_start..end)?.find(word)?;
        let key_start = search_start + relative;
        let key_end = key_start + word.len();

        if key_start != cursor_start
            && is_identifier_boundary(bytes, key_start, key_end)
            && is_property_key_tail(bytes, key_end, end)
        {
            return Some(key_start);
        }

        search_start = key_end;
    }

    None
}

fn is_identifier_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    let after = bytes.get(end);
    !before.is_some_and(|byte| helpers::is_word_char(*byte))
        && !after.is_some_and(|byte| helpers::is_word_char(*byte))
}

fn is_property_key_tail(bytes: &[u8], key_end: usize, range_end: usize) -> bool {
    let mut pos = key_end;
    while pos < range_end && bytes.get(pos).is_some_and(u8::is_ascii_whitespace) {
        pos += 1;
    }

    pos >= range_end || matches!(bytes.get(pos), None | Some(b':' | b'(' | b',' | b'}'))
}

fn find_matching_byte(content: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    let mut pos = start;

    while let Some(&byte) = bytes.get(pos) {
        if let Some(current_quote) = quote {
            if byte == b'\\' {
                pos += 2;
                continue;
            }
            if byte == current_quote {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' || byte == b'`' {
            quote = Some(byte);
        } else if byte == open {
            depth += 1;
        } else if byte == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(pos);
            }
        }
        pos += 1;
    }

    None
}
