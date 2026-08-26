use vize_s0::String;

use crate::SourceLocation;

use super::contract::AiSourceSnippet;

pub(super) fn extract_source_snippet(
    id: String,
    location: &SourceLocation,
    source: &str,
    max_bytes: usize,
) -> Option<AiSourceSnippet> {
    let source_len = floor_char_boundary(source, source.len().min(u32::MAX as usize));
    let max_bytes = max_bytes.min(source_len);
    if max_bytes == 0 {
        return None;
    }

    let requested_start = location.start as usize;
    let requested_end = location.end as usize;
    let requested_outside_source = requested_start > source_len || requested_end > source_len;
    let focus_start = floor_char_boundary(source, requested_start.min(source_len));
    let focus_end = ceil_char_boundary(source, requested_end.min(source_len), source_len);
    let line_start = source[..focus_start]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let line_end = source[focus_end..source_len]
        .find('\n')
        .map_or(source_len, |index| focus_end + index + 1);

    let (start, end) = if line_end.saturating_sub(line_start) <= max_bytes {
        expand_around(source, line_start, line_end, max_bytes, source_len)
    } else {
        expand_around(source, focus_start, focus_end, max_bytes, source_len)
    };
    if start >= end {
        return None;
    }

    let bounded_focus_start = focus_start.max(start).min(end);
    let bounded_focus_end = focus_end.max(bounded_focus_start).min(end);
    Some(AiSourceSnippet {
        id,
        path: location.path.clone(),
        content_start: start as u32,
        content_end: end as u32,
        focus_start: bounded_focus_start as u32,
        focus_end: bounded_focus_end as u32,
        text: source[start..end].into(),
        truncated_before: start > 0,
        truncated_after: end < source.len(),
        focus_truncated: requested_outside_source || start > focus_start || end < focus_end,
    })
}

fn expand_around(
    source: &str,
    focus_start: usize,
    focus_end: usize,
    max_bytes: usize,
    source_len: usize,
) -> (usize, usize) {
    let focus_len = focus_end.saturating_sub(focus_start);
    if focus_len >= max_bytes {
        let end = floor_char_boundary(source, focus_start.saturating_add(max_bytes));
        return (focus_start, end);
    }

    let context = max_bytes - focus_len;
    let desired_before = context / 2;
    let mut start = floor_char_boundary(source, focus_start.saturating_sub(desired_before));
    let mut end = ceil_char_boundary(
        source,
        focus_end.saturating_add(context - (focus_start - start)),
        source_len,
    );
    if end.saturating_sub(start) > max_bytes {
        end = floor_char_boundary(source, start + max_bytes);
    }

    let unused = max_bytes.saturating_sub(end.saturating_sub(start));
    start = floor_char_boundary(source, start.saturating_sub(unused));
    if end.saturating_sub(start) > max_bytes {
        start = ceil_char_boundary(source, end - max_bytes, source_len);
    }
    (start, end)
}

fn floor_char_boundary(source: &str, mut offset: usize) -> usize {
    offset = offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn ceil_char_boundary(source: &str, mut offset: usize, limit: usize) -> usize {
    offset = offset.min(limit).min(source.len());
    while offset < limit && !source.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}
