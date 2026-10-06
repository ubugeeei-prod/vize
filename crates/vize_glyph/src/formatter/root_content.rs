//! Preserve adjacency inside an existing source-owned inter-block gap.

pub(super) fn is_attached_comment(gap: &str) -> bool {
    classify(gap).0
}

// Sorting can move an attached group before the first block. Retain the
// earlier document prologue, but recover the group after its blank separator.
pub(super) fn split_prologue(gap: &str) -> (&str, Option<&str>) {
    let (attached, group_start) = classify(gap);
    if attached
        && let Some(start) = group_start
        && let Some((prologue, group)) = gap.split_at_checked(start)
    {
        return (prologue.trim(), Some(group.trim()));
    }
    (gap.trim(), None)
}

fn classify(gap: &str) -> (bool, Option<usize>) {
    let content = gap.trim_end();
    if !content.ends_with("-->") {
        return (false, None);
    }
    let mut remaining = content;
    let mut offset = 0;
    let mut group_start = None;
    loop {
        let Some((before, body)) = remaining.split_once("<!--") else {
            return (false, None);
        };
        if !before.trim().is_empty() {
            group_start = None;
        }
        let Some(tail) = before.get(before.trim_end().len()..) else {
            return (false, None);
        };
        if has_blank_line(tail) && (offset > 0 || !before.trim().is_empty()) {
            group_start = Some(offset + before.len());
        }
        let Some((_, after)) = body.split_once("-->") else {
            return (false, None);
        };
        if after.is_empty() {
            break;
        }
        offset = content.len() - after.len();
        remaining = after;
    }
    let Some(tail) = gap.get(content.len()..) else {
        return (false, None);
    };
    (!has_blank_line(tail), group_start)
}

fn has_blank_line(tail: &str) -> bool {
    let bytes = tail.as_bytes();
    let mut breaks = 0;
    for (index, &byte) in bytes.iter().enumerate() {
        if byte == b'\n' || (byte == b'\r' && bytes.get(index + 1) != Some(&b'\n')) {
            breaks += 1;
            if breaks > 1 {
                return true;
            }
        }
    }
    false
}
