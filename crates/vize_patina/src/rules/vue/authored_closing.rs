//! Recover an authored closing tag when HTML-style parser recovery detached it
//! from the element node (notably `<p>` around custom components).

use std::ops::Range;

pub(super) fn find(source: &str, from: u32, tag: &str) -> Option<Range<u32>> {
    let bytes = source.as_bytes();
    let mut cursor = usize::try_from(from).ok()?;
    let mut depth = 1usize;
    // An `<li>` ends when its parent list ends. A later `</li>` belongs to
    // an enclosing item, so it is not this element's authored end tag.
    let mut list_depth = 0usize;
    let stops_at_list = tag.eq_ignore_ascii_case("li");
    while cursor < bytes.len() {
        let relative = source.get(cursor..)?.find('<')?;
        let start = cursor + relative;
        if source.get(start..)?.starts_with("<!--") {
            cursor = source.get(start + 4..)?.find("-->")? + start + 7;
            continue;
        }
        let closing = bytes.get(start + 1) == Some(&b'/');
        let name_start = start + if closing { 2 } else { 1 };
        let name_len = bytes
            .get(name_start..)?
            .iter()
            .take_while(is_name_byte)
            .count();
        let name_end = name_start + name_len;
        let name = source.get(name_start..name_end)?;
        let boundary = bytes
            .get(name_end)
            .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>'));
        let end = tag_end(bytes, name_end)?;
        if stops_at_list && boundary && is_list_parent(name) {
            if closing {
                if list_depth == 0 {
                    return None;
                }
                list_depth -= 1;
            } else if !self_closing(bytes, start, end) {
                list_depth += 1;
            }
        }
        if boundary && name.eq_ignore_ascii_case(tag) {
            if closing {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(u32::try_from(start).ok()?..u32::try_from(end).ok()?);
                }
            } else if !self_closing(bytes, start, end) {
                depth += 1;
            }
        }
        cursor = end;
    }
    None
}

fn is_name_byte(byte: &&u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b':')
}

fn is_list_parent(name: &str) -> bool {
    name.eq_ignore_ascii_case("ul") || name.eq_ignore_ascii_case("ol")
}

fn self_closing(bytes: &[u8], start: usize, end: usize) -> bool {
    bytes
        .get(start..end.saturating_sub(1))
        .and_then(|open| {
            open.iter()
                .rev()
                .find(|byte| !byte.is_ascii_whitespace())
                .copied()
        })
        .is_some_and(|byte| byte == b'/')
}

fn tag_end(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    let mut quote = None;
    while let Some(&byte) = bytes.get(cursor) {
        match (quote, byte) {
            (Some(current), byte) if current == byte => quote = None,
            (Some(_), _) => {}
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => return Some(cursor + 1),
            _ => {}
        }
        cursor += 1;
    }
    None
}
