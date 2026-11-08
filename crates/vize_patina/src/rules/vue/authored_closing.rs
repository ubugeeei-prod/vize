//! Recover an authored closing tag when HTML-style parser recovery detached it
//! from the element node (notably `<p>` around custom components).

use std::ops::Range;

pub(super) fn find(source: &str, from: u32, tag: &str) -> Option<Range<u32>> {
    let bytes = source.as_bytes();
    let mut cursor = usize::try_from(from).ok()?;
    let mut depth = 1usize;
    while cursor < bytes.len() {
        let relative = source.get(cursor..)?.find('<')?;
        let start = cursor + relative;
        if source.get(start..)?.starts_with("<!--") {
            cursor = source.get(start + 4..)?.find("-->")? + start + 7;
            continue;
        }
        let closing = bytes.get(start + 1) == Some(&b'/');
        let name_start = start + if closing { 2 } else { 1 };
        let name_end = name_start.checked_add(tag.len())?;
        let same_tag = bytes
            .get(name_start..name_end)
            .is_some_and(|name| name.eq_ignore_ascii_case(tag.as_bytes()))
            && bytes
                .get(name_end)
                .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>'));
        let end = tag_end(bytes, name_end)?;
        if same_tag {
            if closing {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(u32::try_from(start).ok()?..u32::try_from(end).ok()?);
                }
            } else if !bytes
                .get(start..end.saturating_sub(1))?
                .iter()
                .rev()
                .find(|byte| !byte.is_ascii_whitespace())
                .is_some_and(|byte| *byte == b'/')
            {
                depth += 1;
            }
        }
        cursor = end;
    }
    None
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
