use tower_lsp::lsp_types::{Position, Range};

use super::{IdeContext, helpers};
use crate::ide::is_component_tag;

pub(super) struct ComponentEventAtOffset {
    pub(super) name: String,
    pub(super) component_name: String,
    pub(super) range: Range,
}

pub(super) fn event_at_offset(ctx: &IdeContext<'_>) -> Option<ComponentEventAtOffset> {
    let attr = raw_attribute_and_component_at_offset(ctx)?;
    if !is_component_tag(&attr.component_name) {
        return None;
    }
    let (name, event_start, event_end) = event_name_span(&attr.raw_name, attr.name_start)?;
    let (start_line, start_character) = helpers::offset_to_position(&ctx.content, event_start);
    let (end_line, end_character) = helpers::offset_to_position(&ctx.content, event_end);

    Some(ComponentEventAtOffset {
        name: name.to_string(),
        component_name: attr.component_name,
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
    })
}

struct RawAttributeAtOffset {
    raw_name: String,
    name_start: usize,
    component_name: String,
}

fn raw_attribute_and_component_at_offset(ctx: &IdeContext<'_>) -> Option<RawAttributeAtOffset> {
    let content = &ctx.content;
    let cursor = ctx.offset.min(content.len());
    let (tag_start, tag_end, name_start, name_end) = find_tag_name_span(content, cursor)?;
    let bytes = content.as_bytes();
    if bytes.get(tag_start + 1) == Some(&b'/') {
        return None;
    }

    let component_name = content[name_start..name_end].to_string();
    let mut pos = name_end;
    while pos < tag_end {
        while pos < tag_end && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= tag_end || bytes[pos] == b'/' {
            break;
        }

        let attr_start = pos;
        while pos < tag_end
            && !bytes[pos].is_ascii_whitespace()
            && bytes[pos] != b'='
            && bytes[pos] != b'/'
        {
            pos += 1;
        }
        let attr_end = pos;
        if attr_start == attr_end {
            break;
        }
        let cursor_on_attr_name = cursor >= attr_start && cursor <= attr_end;

        while pos < tag_end && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos < tag_end && bytes[pos] == b'=' {
            pos = skip_attribute_value(content, tag_end, pos + 1);
        }

        if cursor_on_attr_name {
            return Some(RawAttributeAtOffset {
                raw_name: content[attr_start..attr_end].to_string(),
                name_start: attr_start,
                component_name,
            });
        }
    }

    None
}

fn event_name_span(raw_name: &str, attr_start: usize) -> Option<(&str, usize, usize)> {
    let (event, prefix_len) = raw_name
        .strip_prefix('@')
        .map(|event| (event, 1))
        .or_else(|| raw_name.strip_prefix("v-on:").map(|event| (event, 5)))?;
    let event = event.split_once('.').map_or(event, |(name, _)| name);
    if event.is_empty() || event.starts_with('[') {
        return None;
    }
    let start = attr_start + prefix_len;
    Some((event, start, start + event.len()))
}

fn skip_attribute_value(content: &str, tag_end: usize, mut pos: usize) -> usize {
    let bytes = content.as_bytes();
    while pos < tag_end && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    if pos < tag_end && matches!(bytes[pos], b'"' | b'\'') {
        let quote = bytes[pos];
        pos += 1;
        while pos < tag_end && bytes[pos] != quote {
            pos += 1;
        }
        return (pos + 1).min(tag_end);
    }
    while pos < tag_end && !bytes[pos].is_ascii_whitespace() && bytes[pos] != b'>' {
        pos += 1;
    }
    pos
}

fn find_tag_name_span(content: &str, offset: usize) -> Option<(usize, usize, usize, usize)> {
    let bytes = content.as_bytes();
    let mut cursor = offset.min(bytes.len());
    if cursor == bytes.len() {
        cursor = cursor.saturating_sub(1);
    }
    if cursor > 0 && bytes.get(cursor) == Some(&b'>') {
        cursor -= 1;
    }
    let mut search_end = cursor.saturating_add(1).min(content.len());
    while search_end > 0 {
        let tag_start = content[..search_end].rfind('<')?;
        let tag_end = find_tag_end_from(content, tag_start)?;
        if cursor > tag_end {
            return None;
        }
        let mut name_start = tag_start + 1;
        if name_start < tag_end && bytes[name_start] == b'/' {
            name_start += 1;
        }
        let mut name_end = name_start;
        while name_end < tag_end {
            let byte = bytes[name_end];
            if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' {
                name_end += 1;
            } else {
                break;
            }
        }
        if name_start != name_end {
            return Some((tag_start, tag_end, name_start, name_end));
        }
        search_end = tag_start;
    }
    None
}

fn find_tag_end_from(content: &str, tag_start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
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
            return Some(tag_end);
        }
        tag_end += 1;
    }
    None
}
