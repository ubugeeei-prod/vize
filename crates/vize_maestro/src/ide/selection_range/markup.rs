//! Markup span scanner for [`super::SelectionRangeService`].
//!
//! Walks a region of the authored document as HTML/Vue template markup and
//! reports every element, tag, attribute and interpolation span that contains
//! the cursor. The caller sorts and nests the spans; this module only has to
//! find them, so it never allocates a tree.
//!
//! The scanner is deliberately text-based rather than AST-based: selection
//! ranges must keep working while the user is mid-edit and the template does
//! not parse.

/// An element whose start tag has been seen but whose close tag has not.
#[derive(Clone, Copy)]
struct OpenTag<'a> {
    name: &'a str,
    tag_start: usize,
    content_start: usize,
}

/// Scan `region` as markup, pushing every span that contains `offset`.
pub(super) fn markup_spans(
    content: &str,
    region: (usize, usize),
    offset: usize,
    spans: &mut Vec<(usize, usize)>,
) {
    let (region_start, region_end) = region;
    let bytes = content.as_bytes();
    let mut stack: Vec<OpenTag<'_>> = Vec::new();
    let mut cursor = region_start;

    while cursor < region_end {
        match bytes[cursor] {
            b'<' if content[cursor..region_end].starts_with("<!--") => {
                let end = content[cursor..region_end]
                    .find("-->")
                    .map_or(region_end, |relative| cursor + relative + 3);
                push_if_contains(spans, (cursor, end), offset);
                cursor = end;
            }
            b'<' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor = close_tag(content, region_end, cursor, offset, &mut stack, spans);
            }
            b'<' => {
                let Some(next) = start_tag(content, region_end, cursor, offset, &mut stack, spans)
                else {
                    return;
                };
                cursor = next;
            }
            b'{' if bytes.get(cursor + 1) == Some(&b'{') => {
                cursor = interpolation(content, region_end, cursor, offset, spans);
            }
            _ => cursor += 1,
        }
    }
}

/// Handle `</name>`: pop the matching open tag and report the element and its
/// inner content. Returns the offset to continue scanning from.
fn close_tag(
    content: &str,
    region_end: usize,
    cursor: usize,
    offset: usize,
    stack: &mut Vec<OpenTag<'_>>,
    spans: &mut Vec<(usize, usize)>,
) -> usize {
    let name_start = cursor + 2;
    let name_end = tag_name_end(content.as_bytes(), name_start, region_end);
    let close_end = content[cursor..region_end]
        .find('>')
        .map_or(region_end, |relative| cursor + relative + 1);

    let name = &content[name_start..name_end];
    // `rposition` recovers from unclosed inner elements: the nearest matching
    // open tag wins and everything opened after it is discarded.
    if let Some(index) = stack.iter().rposition(|open| open.name == name) {
        let open = stack[index];
        stack.truncate(index);
        push_if_contains(spans, (open.content_start, cursor), offset);
        push_if_contains(spans, (open.tag_start, close_end), offset);
    }

    close_end
}

/// Handle `<name …>`: report the start tag and its attributes, and push the
/// element onto the stack unless it is self-closing or a void element.
///
/// Returns `None` when the start tag never closes, which ends the scan.
fn start_tag<'a>(
    content: &'a str,
    region_end: usize,
    cursor: usize,
    offset: usize,
    stack: &mut Vec<OpenTag<'a>>,
    spans: &mut Vec<(usize, usize)>,
) -> Option<usize> {
    let name_start = cursor + 1;
    let name_end = tag_name_end(content.as_bytes(), name_start, region_end);
    let tag_end = start_tag_end(content, cursor, region_end)?;
    if name_end == name_start {
        return Some(cursor + 1);
    }

    attribute_spans(content, (name_end, tag_end - 1), offset, spans);
    push_if_contains(spans, (cursor, tag_end), offset);

    let name = &content[name_start..name_end];
    let self_closing = content[..tag_end - 1].trim_end().ends_with('/');
    if !self_closing && !vize_s0::is_void_tag(name) {
        stack.push(OpenTag {
            name,
            tag_start: cursor,
            content_start: tag_end,
        });
    }

    Some(tag_end)
}

/// Handle `{{ expr }}`: report the trimmed expression and the whole mustache.
fn interpolation(
    content: &str,
    region_end: usize,
    cursor: usize,
    offset: usize,
    spans: &mut Vec<(usize, usize)>,
) -> usize {
    let inner_start = cursor + 2;
    let Some(relative) = content[inner_start..region_end].find("}}") else {
        return inner_start;
    };

    let inner_end = inner_start + relative;
    let inner = &content[inner_start..inner_end];
    let trimmed_start = inner_start + (inner.len() - inner.trim_start().len());
    let trimmed_end = inner_end - (inner.len() - inner.trim_end().len());
    push_if_contains(spans, (trimmed_start, trimmed_end), offset);
    push_if_contains(spans, (cursor, inner_end + 2), offset);

    inner_end + 2
}

/// Scan the interior of a start tag (between the tag name and its `>`).
fn attribute_spans(
    content: &str,
    region: (usize, usize),
    offset: usize,
    spans: &mut Vec<(usize, usize)>,
) {
    let bytes = content.as_bytes();
    let (region_start, region_end) = region;
    let mut cursor = region_start;

    while cursor < region_end {
        if bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b'/' {
            cursor += 1;
            continue;
        }

        let name_start = cursor;
        while cursor < region_end && !is_attribute_terminator(bytes[cursor]) {
            cursor += 1;
        }
        let name_end = cursor;
        if name_end == name_start {
            cursor += 1;
            continue;
        }

        let mut probe = cursor;
        while probe < region_end && bytes[probe].is_ascii_whitespace() {
            probe += 1;
        }
        if probe >= region_end || bytes[probe] != b'=' {
            // A valueless attribute such as `disabled`.
            push_if_contains(spans, (name_start, name_end), offset);
            continue;
        }

        probe += 1;
        while probe < region_end && bytes[probe].is_ascii_whitespace() {
            probe += 1;
        }
        cursor = attribute_value(content, (probe, region_end), (name_start, offset), spans);
    }
}

/// Report the value of an attribute starting at `region.0` plus the whole
/// attribute, and return the offset just past the value.
fn attribute_value(
    content: &str,
    region: (usize, usize),
    name_start_and_offset: (usize, usize),
    spans: &mut Vec<(usize, usize)>,
) -> usize {
    let bytes = content.as_bytes();
    let (value_probe, region_end) = region;
    let (name_start, offset) = name_start_and_offset;

    if value_probe < region_end && (bytes[value_probe] == b'"' || bytes[value_probe] == b'\'') {
        let quote = bytes[value_probe];
        let value_start = value_probe + 1;
        let mut value_end = value_start;
        while value_end < region_end && bytes[value_end] != quote {
            value_end += 1;
        }
        let quoted_end = (value_end + 1).min(region_end);
        push_if_contains(spans, (value_start, value_end), offset);
        // `@vue/language-server` offers the quoted value as its own level
        // between the bare value and the whole attribute; keep parity.
        push_if_contains(spans, (value_start - 1, quoted_end), offset);
        push_if_contains(spans, (name_start, quoted_end), offset);
        return quoted_end;
    }

    let mut value_end = value_probe;
    while value_end < region_end && !bytes[value_end].is_ascii_whitespace() {
        value_end += 1;
    }
    push_if_contains(spans, (value_probe, value_end), offset);
    push_if_contains(spans, (name_start, value_end), offset);
    value_end
}

#[inline]
fn is_attribute_terminator(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'=' | b'>' | b'/')
}

fn tag_name_end(bytes: &[u8], name_start: usize, limit: usize) -> usize {
    let mut end = name_start;
    while end < limit
        && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'-' | b'_' | b'.' | b':'))
    {
        end += 1;
    }
    end
}

/// Byte offset just past the `>` that closes the start tag at `tag_start`.
fn start_tag_end(content: &str, tag_start: usize, limit: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut cursor = tag_start + 1;
    let mut quote: Option<u8> = None;

    while cursor < limit {
        let byte = bytes[cursor];
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'>' => return Some(cursor + 1),
            None => {}
        }
        cursor += 1;
    }

    None
}

#[inline]
pub(super) fn push_if_contains(
    spans: &mut Vec<(usize, usize)>,
    span: (usize, usize),
    offset: usize,
) {
    if span.0 <= offset && offset <= span.1 {
        spans.push(span);
    }
}
