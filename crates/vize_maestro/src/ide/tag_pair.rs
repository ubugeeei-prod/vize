//! Resolve the open/close tag-name pair of the element under a cursor.
//!
//! Shared by `textDocument/linkedEditingRange` and
//! `textDocument/documentHighlight`: both answer "which tag names belong to the
//! element the caret is on?", and both are wrong in a way the user cannot see
//! when the answer is approximated. A document-wide name scan looks plausible
//! in a screenshot and is useless in a template with many `<div>`s, so the
//! nesting is modelled with a stack rather than matched by name.
//!
//! The caller supplies the byte region that is safe to read as markup (see
//! [`super::sfc_region`]); this module never decides that on its own, because
//! scanning a `<script>` body as markup makes `a < b` look like a tag.

/// Tag-name byte spans of the element under the cursor, in ascending document
/// order.
pub(crate) struct TagNames {
    /// The name the cursor is on, or the open-tag name when the pair resolved.
    pub(crate) first: (usize, usize),
    /// The counterpart name. `None` for a self-closing element, a void element
    /// (`<br>`, `<img>`), an open tag that is never closed, and a close tag
    /// with no open tag — none of which have a second name to keep in sync.
    pub(crate) second: Option<(usize, usize)>,
}

/// An element whose start tag has been seen but whose close tag has not.
#[derive(Clone, Copy)]
struct OpenTag<'a> {
    name: &'a str,
    name_span: (usize, usize),
}

/// Resolve the tag names of the element whose name `offset` sits on.
///
/// Returns `None` when the cursor is not on a tag name at all.
pub(crate) fn names_at(content: &str, region: (usize, usize), offset: usize) -> Option<TagNames> {
    let (region_start, region_end) = region;
    let bytes = content.as_bytes();
    let mut stack: Vec<OpenTag<'_>> = Vec::new();
    // Set when the cursor's own open tag is pushed. If the scan ends without
    // ever matching it, the element is unclosed and has a single name.
    let mut unclosed: Option<(usize, usize)> = None;
    let mut cursor = region_start;

    while cursor < region_end {
        if bytes[cursor] != b'<' {
            cursor += 1;
            continue;
        }

        if content[cursor..region_end].starts_with("<!--") {
            cursor = content[cursor..region_end]
                .find("-->")
                .map_or(region_end, |relative| cursor + relative + 3);
            continue;
        }

        if bytes.get(cursor + 1) == Some(&b'/') {
            let name_start = cursor + 2;
            let name_end = tag_name_end(bytes, name_start, region_end);
            if name_end == name_start {
                // `</` with no name yet: a zero-width span the cursor would
                // "sit on", which must not become a zero-width highlight.
                cursor = content[cursor..region_end]
                    .find('>')
                    .map_or(region_end, |relative| cursor + relative + 1);
                continue;
            }
            let close_span = (name_start, name_end);
            let name = &content[name_start..name_end];

            // `rposition` recovers from unclosed inner elements: the nearest
            // matching open tag wins.
            if let Some(index) = stack.iter().rposition(|open| open.name == name) {
                let open = stack[index];
                stack.truncate(index);
                if contains(open.name_span, offset) || contains(close_span, offset) {
                    return Some(TagNames {
                        first: open.name_span,
                        second: Some(close_span),
                    });
                }
            } else if contains(close_span, offset) {
                // A close tag with no open tag: still a tag name, but nothing
                // is paired with it.
                return Some(TagNames {
                    first: close_span,
                    second: None,
                });
            }

            cursor = content[cursor..region_end]
                .find('>')
                .map_or(region_end, |relative| cursor + relative + 1);
            continue;
        }

        let name_start = cursor + 1;
        let name_end = tag_name_end(bytes, name_start, region_end);
        if name_end == name_start {
            cursor += 1;
            continue;
        }

        let name = &content[name_start..name_end];
        let name_span = (name_start, name_end);
        // A start tag with no `>` is a name being typed (or a stray `<`): no tag
        // in the rest of the region can close either, so stop instead of
        // discarding the pair or unclosed name already resolved.
        let Some(tag_end) = start_tag_end(content, cursor, region_end) else {
            if contains(name_span, offset) {
                return Some(TagNames {
                    first: name_span,
                    second: None,
                });
            }
            break;
        };
        let self_closing = content[..tag_end - 1].trim_end().ends_with('/');
        if self_closing || vize_s0::is_void_tag(name) {
            if contains(name_span, offset) {
                return Some(TagNames {
                    first: name_span,
                    second: None,
                });
            }
        } else {
            if contains(name_span, offset) {
                unclosed = Some(name_span);
            }
            stack.push(OpenTag { name, name_span });
        }
        cursor = tag_end;
    }

    unclosed.map(|name_span| TagNames {
        first: name_span,
        second: None,
    })
}

/// The cursor counts as "on" a name when it is anywhere inside it, including the
/// position immediately after the last character — that is where an editor
/// leaves the caret while the user is typing the name.
#[inline]
fn contains(span: (usize, usize), offset: usize) -> bool {
    span.0 <= offset && offset <= span.1
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

#[cfg(test)]
mod tests;
