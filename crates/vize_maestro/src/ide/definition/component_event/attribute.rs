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
    let (name, event_start, event_end) = helpers::event_name_span(&attr.raw_name, attr.name_start)?;
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
    let authored = helpers::raw_attribute_with_source_span_at_offset(ctx)?;
    Some(RawAttributeAtOffset {
        raw_name: authored.raw_name,
        name_start: authored.name_start,
        component_name: authored.tag_name,
    })
}
