//! `textDocument/foldingRange` for authored `.vue` documents.
//!
//! Regions are produced for multi-line SFC blocks, template elements and
//! comments, script object/function bodies, and style rule blocks.
//!
//! # `endLine` is the last line the client hides
//!
//! It is *not* the closing line of the construct. A region that reaches the
//! closing tag folds the terminator away, leaving a collapsed block with
//! nothing to show where it ends. `@vue/language-server` 3.3.8 reports `4..8`
//! for a `<template>` whose `</template>` is on line 9; Maestro reported `4..9`
//! until #3455. Every region below therefore stops one line short of the
//! construct's closing line, and a construct with nothing between its opening
//! and closing lines produces no region at all.
//!
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

use crate::server::ServerState;

/// One region per multi-line SFC block, then nested regions in template,
/// script, and style discovery order.
pub(crate) fn folding_ranges(
    state: &ServerState,
    params: &FoldingRangeParams,
) -> Option<Vec<FoldingRange>> {
    let uri = &params.text_document.uri;
    let content = state.documents.text(uri)?;

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: uri.path().to_string().into(),
        ..Default::default()
    };
    let descriptor = vize_atelier_sfc::parse_sfc(&content, options).ok()?;
    let authored_lines = AuthoredLineMap::new(&content);

    let mut ranges = Vec::new();
    let labelled_blocks = descriptor
        .template
        .as_ref()
        .map(|block| (&block.loc, "template"))
        .into_iter()
        .chain(
            descriptor
                .script_setup
                .as_ref()
                .map(|block| (&block.loc, "script setup")),
        )
        .chain(
            descriptor
                .script
                .as_ref()
                .map(|block| (&block.loc, "script")),
        )
        .chain(descriptor.styles.iter().map(|block| (&block.loc, "style")));

    for (loc, label) in labelled_blocks {
        let Some(range) = region(
            authored_lines.line_at(loc.tag_start),
            authored_lines.line_at(loc.end),
            Some(FoldingRangeKind::Region),
            Some(label),
        ) else {
            continue;
        };
        ranges.push(range);
    }

    if let Some(template) = descriptor.template.as_ref() {
        // The template block's *content*: its own tags already fold as the
        // block region above, and scanning them again would emit that range a
        // second time.
        ranges.extend(markup_regions(
            &content,
            (template.loc.start, template.loc.end),
        ));
    }

    for script in descriptor
        .script_setup
        .iter()
        .chain(descriptor.script.iter())
    {
        ranges.extend(script::script_regions(
            script,
            authored_lines.line_at(script.loc.start),
        ));
    }
    for style in &descriptor.styles {
        ranges.extend(style::style_regions(
            style,
            authored_lines.line_at(style.loc.start),
        ));
    }

    (!ranges.is_empty()).then_some(ranges)
}

/// Maps authored byte offsets to zero-based lines in O(1) per lookup.
///
/// Construction scans the document once, so nested-region discovery neither
/// rescans source prefixes nor binary-searches a newline table.
struct AuthoredLineMap {
    lines: Vec<u32>,
}

impl AuthoredLineMap {
    fn new(source: &str) -> Self {
        let mut lines = Vec::with_capacity(source.len() + 1);
        let mut line = 0;
        for byte in source.bytes() {
            lines.push(line);
            line += u32::from(byte == b'\n');
        }
        lines.push(line);
        Self { lines }
    }

    fn line_at(&self, byte_offset: usize) -> u32 {
        self.lines[byte_offset.min(self.lines.len() - 1)]
    }
}

/// A region that hides `open_line + 1 ..= close_line - 1`.
///
/// `None` when that span is empty, which is every construct whose closing line
/// is its own line or the one right after it: folding it would hide nothing.
fn region(
    open_line: u32,
    close_line: u32,
    kind: Option<FoldingRangeKind>,
    collapsed_text: Option<&str>,
) -> Option<FoldingRange> {
    if close_line < open_line + 2 {
        return None;
    }
    Some(FoldingRange {
        start_line: open_line,
        start_character: None,
        end_line: close_line - 1,
        end_character: None,
        kind,
        collapsed_text: collapsed_text.map(|text| text.to_string()),
    })
}

/// An element whose start tag has been seen but whose close tag has not.
struct OpenElement<'a> {
    name: &'a str,
    open_line: u32,
}

/// Fold regions for the elements and comments in `region`, in document order.
///
/// A hand-rolled scan rather than a template parse: folding must keep working
/// while the user is mid-edit and the template does not parse, which is exactly
/// when they are most likely to collapse a region to see the structure.
fn markup_regions(content: &str, region_span: (usize, usize)) -> Vec<FoldingRange> {
    let (region_start, region_end) = region_span;
    let bytes = content.as_bytes();
    let mut line = newlines(&content[..region_start]);
    let mut stack: Vec<OpenElement<'_>> = Vec::new();
    let mut ranges = Vec::new();
    let mut cursor = region_start;

    while cursor < region_end {
        if bytes[cursor] == b'\n' {
            line += 1;
            cursor += 1;
            continue;
        }
        if bytes[cursor] != b'<' {
            cursor += 1;
            continue;
        }

        if content[cursor..region_end].starts_with("<!--") {
            let close = content[cursor..region_end]
                .find("-->")
                .map_or(region_end, |relative| cursor + relative);
            let close_line = line + newlines(&content[cursor..close]);
            if let Some(range) = region(line, close_line, Some(FoldingRangeKind::Comment), None) {
                ranges.push(range);
            }
            let past = (close + 3).min(region_end);
            line = close_line + newlines(&content[close..past]);
            cursor = past;
            continue;
        }

        if bytes.get(cursor + 1) == Some(&b'/') {
            let name_start = cursor + 2;
            let name = &content[name_start..tag_name_end(bytes, name_start, region_end)];
            // `rposition` recovers from unclosed inner elements: the nearest
            // matching open tag wins, and everything above it is dropped
            // unfolded because it never got a closing line.
            if let Some(index) = stack.iter().rposition(|open| open.name == name) {
                let open = &stack[index];
                if let Some(range) = region(open.open_line, line, None, None) {
                    ranges.push(range);
                }
                stack.truncate(index);
            }
            let past = content[cursor..region_end]
                .find('>')
                .map_or(region_end, |relative| cursor + relative + 1);
            line += newlines(&content[cursor..past]);
            cursor = past;
            continue;
        }

        let name_start = cursor + 1;
        let name_end = tag_name_end(bytes, name_start, region_end);
        if name_end == name_start {
            cursor += 1;
            continue;
        }
        let Some(tag_end) = start_tag_end(content, cursor, region_end) else {
            break;
        };

        let name = &content[name_start..name_end];
        let self_closing = content[..tag_end - 1].trim_end().ends_with('/');
        if !self_closing && !vize_s0::is_void_tag(name) {
            stack.push(OpenElement {
                name,
                open_line: line,
            });
        }
        line += newlines(&content[cursor..tag_end]);
        cursor = tag_end;
    }

    // Regions are discovered when the *close* tag is reached, so an outer
    // element is pushed after the inner ones it contains. Restore document
    // order, which is what a reader of a pinned expectation expects to see.
    ranges.sort_by_key(|range| (range.start_line, range.end_line));
    ranges
}

#[inline]
fn newlines(text: &str) -> u32 {
    text.bytes().filter(|byte| *byte == b'\n').count() as u32
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

mod script;
mod style;
#[cfg(test)]
mod tests;
