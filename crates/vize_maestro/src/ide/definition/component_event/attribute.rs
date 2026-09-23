use tower_lsp::lsp_types::{Position, Range};

use super::{IdeContext, helpers, helpers::find_tag_name_span};
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

    let component_name = content.get(name_start..name_end)?.to_string();
    let mut pos = name_end;
    // Byte at `i` while still inside the tag; `None` past `tag_end`.
    let at = |i: usize| {
        if i < tag_end {
            bytes.get(i).copied()
        } else {
            None
        }
    };
    while pos < tag_end {
        while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
            pos += 1;
        }
        if matches!(at(pos), None | Some(b'/')) {
            break;
        }

        let attr_start = pos;
        while at(pos).is_some_and(|b| !b.is_ascii_whitespace() && b != b'=' && b != b'/') {
            pos += 1;
        }
        let attr_end = pos;
        if attr_start == attr_end {
            break;
        }
        let cursor_on_attr_name = cursor >= attr_start && cursor <= attr_end;

        while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
            pos += 1;
        }
        if at(pos) == Some(b'=') {
            pos = skip_attribute_value(content, tag_end, pos + 1);
        }

        if cursor_on_attr_name {
            return Some(RawAttributeAtOffset {
                raw_name: content.get(attr_start..attr_end)?.to_string(),
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
    let at = |i: usize| {
        if i < tag_end {
            bytes.get(i).copied()
        } else {
            None
        }
    };
    while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
        pos += 1;
    }
    if let Some(quote @ (b'"' | b'\'')) = at(pos) {
        pos += 1;
        while at(pos).is_some_and(|b| b != quote) {
            pos += 1;
        }
        return (pos + 1).min(tag_end);
    }
    while at(pos).is_some_and(|b| !b.is_ascii_whitespace() && b != b'>') {
        pos += 1;
    }
    pos
}
