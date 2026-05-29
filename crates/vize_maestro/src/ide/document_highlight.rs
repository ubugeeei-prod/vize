//! Document highlight provider.
//!
//! Highlights matching tag names and identifier occurrences in Vue and Art documents.

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
        if let Some((tag_name, _, _)) = tag_name_at_offset(&ctx.content, ctx.offset) {
            let highlights = tag_highlights(&ctx.content, &tag_name);
            return (!highlights.is_empty()).then_some(highlights);
        }

        let (start, end) = token_span_at_offset(&ctx.content, ctx.offset, is_identifier_char)?;
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

fn tag_highlights(content: &str, tag_name: &str) -> Vec<DocumentHighlight> {
    let mut spans = Vec::new();
    let mut search_start = 0usize;
    let bytes = content.as_bytes();
    while let Some(relative) = content[search_start..].find('<') {
        let tag_start = search_start + relative;
        let mut name_start = tag_start + 1;

        if bytes.get(name_start) == Some(&b'/') {
            name_start += 1;
        }

        let name_end = name_start + tag_name.len();
        if name_end <= content.len()
            && &content[name_start..name_end] == tag_name
            && is_tag_name_boundary(bytes, name_start, name_end)
        {
            spans.push((name_start, name_end));
        }

        search_start = tag_start + 1;
    }
    if spans.is_empty() {
        return Vec::new();
    }

    let mut walker = PositionWalker::new(content);
    let mut highlights = Vec::with_capacity(spans.len());
    for (start, end) in spans {
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

fn tag_name_at_offset(content: &str, offset: usize) -> Option<(String, usize, usize)> {
    let bytes = content.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut cursor = offset.min(bytes.len());
    if cursor == bytes.len() {
        cursor = cursor.saturating_sub(1);
    }

    let mut tag_start = None;
    let mut pos = cursor + 1;
    while pos > 0 {
        pos -= 1;
        match bytes[pos] {
            b'<' => {
                tag_start = Some(pos);
                break;
            }
            b'>' | b'\n' | b'\r' => return None,
            _ => {}
        }
    }

    let tag_start = tag_start?;
    let mut tag_end = tag_start;
    let mut quote = None;

    while tag_end < bytes.len() {
        let byte = bytes[tag_end];
        if let Some(current_quote) = quote {
            if byte == current_quote {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'>' {
            break;
        } else if byte == b'\n' || byte == b'\r' {
            return None;
        }
        tag_end += 1;
    }

    if tag_end >= bytes.len() || bytes[tag_end] != b'>' {
        return None;
    }

    let mut name_start = tag_start + 1;
    if bytes.get(name_start) == Some(&b'/') {
        name_start += 1;
    }

    let mut name_end = name_start;
    while name_end < tag_end {
        let byte = bytes[name_end];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            name_end += 1;
        } else {
            break;
        }
    }

    if name_start == name_end || cursor < name_start || cursor > name_end {
        return None;
    }

    Some((
        content[name_start..name_end].to_string(),
        name_start,
        name_end,
    ))
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

fn is_tag_name_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    let after = bytes.get(end);

    !before.is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        && !after.is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::{DocumentHighlightService, PositionWalker};
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::Url;

    #[test]
    fn position_walker_matches_offset_to_position_str() {
        // Multi-line content with a multi-byte (UTF-16 surrogate pair) char so
        // the walker's line/column tracking is exercised against the canonical
        // converter at every char boundary, including ascending re-queries.
        let content = "abc\ndé😀f\n\nghi";
        let mut walker = PositionWalker::new(content);
        let mut prev = 0usize;
        for offset in 0..=content.len() {
            if !content.is_char_boundary(offset) {
                continue;
            }
            // Walker requires monotonic targets; advance from the previous one.
            assert!(offset >= prev);
            prev = offset;
            let expected = crate::utils::offset_to_position_str(content, offset);
            let (line, character) = walker.position_at(offset);
            assert_eq!(
                (line, character),
                (expected.line, expected.character),
                "mismatch at byte offset {offset}",
            );
        }
    }

    fn context_for(source: &str, cursor_text: &str) -> (ServerState, Url, usize) {
        let state = ServerState::new();
        let uri = Url::parse("file:///Button.art.vue").unwrap();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);
        let offset = source.find(cursor_text).unwrap();
        (state, uri, offset)
    }

    #[test]
    fn highlights_identifier_occurrences_in_art_variant() {
        let source = r#"<art title="Button">
  <variant name="Primary">
    <Button :label="label">{{ label }}</Button>
  </variant>
</art>

<script setup lang="ts">
const label = "Primary"
</script>"#;
        let (state, uri, offset) = context_for(source, "label\">{{");
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let highlights = DocumentHighlightService::highlights(&ctx).unwrap();

        assert!(highlights.len() >= 3, "{highlights:#?}");
    }

    #[test]
    fn highlights_matching_component_tags_in_art_variant() {
        let source = r#"<art title="Button">
  <variant name="Primary">
    <Button :label="label"><span>Label</span></Button>
  </variant>
</art>"#;
        let (state, uri, offset) = context_for(source, "Button :label");
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let highlights = DocumentHighlightService::highlights(&ctx).unwrap();

        assert_eq!(highlights.len(), 2, "{highlights:#?}");
    }
}
