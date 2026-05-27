//! Document highlight provider.
//!
//! Highlights matching tag names and identifier occurrences in Vue and Art documents.

use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind, Range};

use super::{IdeContext, offset_to_position, token_span_at_offset};

pub struct DocumentHighlightService;

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
    let mut highlights = Vec::new();
    let mut search_start = 0usize;

    while let Some(relative) = content[search_start..].find(symbol) {
        let start = search_start + relative;
        let end = start + symbol.len();

        if is_identifier_boundary(content.as_bytes(), start, end) {
            highlights.push(highlight(
                content,
                start,
                end,
                identifier_highlight_kind(content, start),
            ));
        }

        search_start = end;
    }

    highlights
}

fn tag_highlights(content: &str, tag_name: &str) -> Vec<DocumentHighlight> {
    let mut highlights = Vec::new();
    let mut search_start = 0usize;

    while let Some(relative) = content[search_start..].find('<') {
        let tag_start = search_start + relative;
        let bytes = content.as_bytes();
        let mut name_start = tag_start + 1;

        if bytes.get(name_start) == Some(&b'/') {
            name_start += 1;
        }

        let name_end = name_start + tag_name.len();
        if name_end <= content.len()
            && &content[name_start..name_end] == tag_name
            && is_tag_name_boundary(bytes, name_start, name_end)
        {
            highlights.push(highlight(
                content,
                name_start,
                name_end,
                Some(DocumentHighlightKind::TEXT),
            ));
        }

        search_start = tag_start + 1;
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

fn highlight(
    content: &str,
    start: usize,
    end: usize,
    kind: Option<DocumentHighlightKind>,
) -> DocumentHighlight {
    let (start_line, start_character) = offset_to_position(content, start);
    let (end_line, end_character) = offset_to_position(content, end);

    DocumentHighlight {
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: start_line,
                character: start_character,
            },
            end: tower_lsp::lsp_types::Position {
                line: end_line,
                character: end_character,
            },
        },
        kind,
    }
}

fn identifier_highlight_kind(content: &str, start: usize) -> Option<DocumentHighlightKind> {
    let line_start = content[..start].rfind('\n').map_or(0, |offset| offset + 1);
    let prefix = content[line_start..start].trim_end();

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
    use super::DocumentHighlightService;
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::Url;

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
