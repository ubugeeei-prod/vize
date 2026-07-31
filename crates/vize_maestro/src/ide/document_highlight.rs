//! Document highlight provider.
//!
//! Highlights the tag-name pair of the element under the cursor, or the
//! occurrences of the identifier under the cursor, in Vue and Art documents.
//!
//! Tag highlighting resolves the enclosing element with the shared stack-based
//! scanner in [`super::tag_pair`]. The previous implementation scanned the whole
//! document for the name, so a cursor on one of four `<div>`s highlighted all
//! eight names — which destroys the one signal the feature exists to give:
//! *which* close tag belongs to the tag under the cursor (#3454).

use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind, Position, Range};

use super::{IdeContext, token_span_at_offset};

pub struct DocumentHighlightService;

/// Forward-only cursor that converts ascending byte offsets to LSP positions
/// in a single pass over the document.
///
/// The previous code called `offset_to_position` (which re-walks the document
/// from offset 0) twice per match, making highlighting O(occurrences × length).
/// Matches are produced left-to-right, so a monotonic cursor turns the whole
/// pass into O(length). Mirrors `offset_to_position_str`: lines count `\n`,
/// columns count UTF-16 code units and reset at each newline.
struct PositionWalker<'a> {
    chars: std::str::CharIndices<'a>,
    content_len: usize,
    /// Byte offset of the next char to process.
    offset: usize,
    line: u32,
    character: u32,
    /// Byte offset where the current line begins (after the last `\n`).
    line_start: usize,
}

impl<'a> PositionWalker<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            chars: content.char_indices(),
            content_len: content.len(),
            offset: 0,
            line: 0,
            character: 0,
            line_start: 0,
        }
    }

    /// Advance to `target` (a byte offset >= any previously requested target)
    /// and return its (line, character) position.
    fn position_at(&mut self, target: usize) -> (u32, u32) {
        let target = target.min(self.content_len);
        while self.offset < target {
            let Some((byte, ch)) = self.chars.next() else {
                break;
            };
            if ch == '\n' {
                self.line += 1;
                self.character = 0;
                self.line_start = byte + 1;
            } else {
                self.character += ch.len_utf16() as u32;
            }
            self.offset = byte + ch.len_utf8();
        }
        (self.line, self.character)
    }

    /// Byte offset where the line containing the most recently visited target
    /// begins. Valid immediately after a `position_at` call.
    fn line_start(&self) -> usize {
        self.line_start
    }
}

impl DocumentHighlightService {
    pub fn highlights(ctx: &IdeContext<'_>) -> Option<Vec<DocumentHighlight>> {
        let offset = ctx.offset.min(ctx.content.len());
        // A cursor inside a raw-text block (`<script>`, `<style>`) has no markup
        // region, so it never takes the tag path and falls through to the
        // identifier scan below.
        if let Some(region) =
            super::sfc_region::resolve(&ctx.content, ctx.uri.path(), offset).markup
            && let Some(names) = super::tag_pair::names_at(&ctx.content, region, offset)
        {
            return Some(tag_highlights(&ctx.content, &names));
        }

        let (start, end) = token_span_at_offset(&ctx.content, offset, is_identifier_char)?;
        let symbol = &ctx.content[start..end];
        if !is_highlightable_symbol(symbol) {
            return None;
        }

        let highlights = identifier_highlights(&ctx.content, symbol);
        (!highlights.is_empty()).then_some(highlights)
    }
}

fn identifier_highlights(content: &str, symbol: &str) -> Vec<DocumentHighlight> {
    // Collect matching spans first (ascending, non-overlapping), then convert
    // every offset to a position with a single forward walk over the document.
    let mut spans = Vec::new();
    let mut search_start = 0usize;
    while let Some(relative) = content[search_start..].find(symbol) {
        let start = search_start + relative;
        let end = start + symbol.len();
        if is_identifier_boundary(content.as_bytes(), start, end) {
            spans.push((start, end));
        }
        search_start = end;
    }
    if spans.is_empty() {
        return Vec::new();
    }

    let mut walker = PositionWalker::new(content);
    let mut highlights = Vec::with_capacity(spans.len());
    for (start, end) in spans {
        let (start_line, start_character) = walker.position_at(start);
        let kind = highlight_kind_for_prefix(&content[walker.line_start()..start]);
        let (end_line, end_character) = walker.position_at(end);
        highlights.push(span_highlight(
            start_line,
            start_character,
            end_line,
            end_character,
            kind,
        ));
    }
    highlights
}

/// One highlight per name of the resolved element: two for a matched pair, one
/// for a self-closing, void or unmatched tag. Never a document-wide name scan.
fn tag_highlights(content: &str, names: &super::tag_pair::TagNames) -> Vec<DocumentHighlight> {
    let mut walker = PositionWalker::new(content);
    let mut highlights = Vec::with_capacity(2);
    for (start, end) in [Some(names.first), names.second].into_iter().flatten() {
        let (start_line, start_character) = walker.position_at(start);
        let (end_line, end_character) = walker.position_at(end);
        highlights.push(span_highlight(
            start_line,
            start_character,
            end_line,
            end_character,
            Some(DocumentHighlightKind::TEXT),
        ));
    }
    highlights
}

fn span_highlight(
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
    kind: Option<DocumentHighlightKind>,
) -> DocumentHighlight {
    DocumentHighlight {
        range: Range {
            start: Position {
                line: start_line,
                character: start_character,
            },
            end: Position {
                line: end_line,
                character: end_character,
            },
        },
        kind,
    }
}

fn highlight_kind_for_prefix(prefix: &str) -> Option<DocumentHighlightKind> {
    let prefix = prefix.trim_end();

    if prefix.ends_with("const")
        || prefix.ends_with("let")
        || prefix.ends_with("var")
        || prefix.ends_with("function")
        || prefix.ends_with("class")
        || prefix.ends_with("interface")
        || prefix.ends_with("type")
        || prefix.ends_with("import")
    {
        Some(DocumentHighlightKind::WRITE)
    } else {
        Some(DocumentHighlightKind::READ)
    }
}

fn is_highlightable_symbol(symbol: &str) -> bool {
    !matches!(
        symbol,
        "true"
            | "false"
            | "null"
            | "undefined"
            | "if"
            | "else"
            | "for"
            | "in"
            | "of"
            | "const"
            | "let"
            | "var"
            | "function"
            | "return"
            | "import"
            | "from"
            | "export"
    )
}

#[inline]
fn is_identifier_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

fn is_identifier_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    let after = bytes.get(end);
    !before.is_some_and(|byte| is_identifier_char(*byte))
        && !after.is_some_and(|byte| is_identifier_char(*byte))
}

#[cfg(test)]
mod tests;
