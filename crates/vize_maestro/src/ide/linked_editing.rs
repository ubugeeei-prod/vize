//! Linked editing ranges (`textDocument/linkedEditingRange`).
//!
//! When the cursor is on a tag name, the editor keeps the open and close tag
//! names in sync as the user types. `@vue/language-server` 3.3.8 advertises
//! `linkedEditingRangeProvider: true`; Maestro advertised nothing and answered
//! `-32601 Method not found`, so renaming `<div>` in a `.vue` file left a
//! dangling `</div>` behind with no indication that anything was missing.
//!
//! Ranges are authored `.vue` coordinates, never virtual TypeScript.
//!
//! # Scope
//!
//! Only tag-name pairs are linked, matching the reference server. Self-closing
//! elements, void elements (`<br>`, `<img>`) and unmatched tags have no
//! counterpart and return `None` rather than a single-element list, because a
//! one-range response would make the editor believe it had linked something.

use tower_lsp::lsp_types::{LinkedEditingRanges, Position, Range};

use super::offset_to_position;

pub struct LinkedEditingService;

/// An element whose start tag has been seen but whose close tag has not.
#[derive(Clone, Copy)]
struct OpenTag<'a> {
    name: &'a str,
    name_span: (usize, usize),
}

impl LinkedEditingService {
    /// Resolve the open/close tag-name pair the cursor sits on.
    ///
    /// `filename` only selects the region to scan (template block, or the whole
    /// file for standalone HTML and `.art.vue`); no server state is consulted.
    pub fn ranges(content: &str, filename: &str, offset: usize) -> Option<LinkedEditingRanges> {
        let offset = offset.min(content.len());
        let region = super::sfc_region::resolve(content, filename, offset).markup?;
        let (open, close) = tag_name_pair(content, region, offset)?;

        Some(LinkedEditingRanges {
            // Open tag first: the reference server orders them by position and
            // clients apply edits in the order they receive them.
            ranges: vec![to_range(content, open), to_range(content, close)],
            // No word pattern: the reference server omits it, which lets the
            // client use its own tag-name pattern for the language.
            word_pattern: None,
        })
    }
}

/// Byte spans of the open and close tag names of the element whose name the
/// cursor is on.
fn tag_name_pair(
    content: &str,
    region: (usize, usize),
    offset: usize,
) -> Option<((usize, usize), (usize, usize))> {
    let (region_start, region_end) = region;
    let bytes = content.as_bytes();
    let mut stack: Vec<OpenTag<'_>> = Vec::new();
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
            let close_span = (name_start, name_end);
            let name = &content[name_start..name_end];

            // `rposition` recovers from unclosed inner elements: the nearest
            // matching open tag wins.
            if let Some(index) = stack.iter().rposition(|open| open.name == name) {
                let open = stack[index];
                stack.truncate(index);
                if contains(open.name_span, offset) || contains(close_span, offset) {
                    return Some((open.name_span, close_span));
                }
            }

            cursor = content[cursor..region_end]
                .find('>')
                .map_or(region_end, |relative| cursor + relative + 1);
            continue;
        }

        let name_start = cursor + 1;
        let name_end = tag_name_end(bytes, name_start, region_end);
        let tag_end = start_tag_end(content, cursor, region_end)?;
        if name_end == name_start {
            cursor += 1;
            continue;
        }

        let name = &content[name_start..name_end];
        let self_closing = content[..tag_end - 1].trim_end().ends_with('/');
        if !self_closing && !vize_carton::is_void_tag(name) {
            stack.push(OpenTag {
                name,
                name_span: (name_start, name_end),
            });
        }
        cursor = tag_end;
    }

    None
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

fn to_range(content: &str, span: (usize, usize)) -> Range {
    let (start_line, start_character) = offset_to_position(content, span.0);
    let (end_line, end_character) = offset_to_position(content, span.1);
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

#[cfg(test)]
mod tests;
