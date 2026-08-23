use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};
use vize_croquis::{Drawer, DrawerOptions, croquis::SlotUsage};

use super::{IdeContext, helpers};

pub(super) fn component_slot_definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    let (slot_name, component_name) = component_slot_at_offset(ctx)?;
    let import_path = helpers::find_import_path(ctx, &component_name)
        .or_else(|| super::template::art_component_path(ctx, &component_name))?;
    let resolved_path = helpers::resolve_import_path(ctx.uri, &import_path)?;
    let component_content = std::fs::read_to_string(&resolved_path).ok()?;

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: resolved_path.to_string_lossy().to_string().into(),
        ..Default::default()
    };
    let descriptor = vize_atelier_sfc::parse_sfc(&component_content, options).ok()?;
    let script_setup = descriptor.script_setup.as_ref()?;
    let script = script_setup.content.as_ref();
    let define_slots_pos = script.find("defineSlots")?;
    let slot_pos = find_slot_in_define_slots(&script[define_slots_pos..], &slot_name)?;
    let sfc_offset = script_setup.loc.start + define_slots_pos + slot_pos;
    let (line, character) = helpers::offset_to_position(&component_content, sfc_offset);
    let file_uri = tower_lsp::lsp_types::Url::from_file_path(&resolved_path).ok()?;

    Some(GotoDefinitionResponse::Scalar(Location {
        uri: file_uri,
        range: Range {
            start: Position { line, character },
            end: Position {
                line,
                character: character + slot_name.len() as u32,
            },
        },
    }))
}

fn component_slot_at_offset(ctx: &IdeContext<'_>) -> Option<(String, String)> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };
    let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;
    let template = descriptor.template.as_ref()?;
    if ctx.offset < template.loc.start || ctx.offset > template.loc.end {
        return None;
    }
    let relative_offset = ctx.offset.saturating_sub(template.loc.start);
    let template_source = template.content.as_ref();

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template_source);
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_template_scopes: true,
        track_usage: true,
        ..Default::default()
    });
    drawer.draw_template(&root);
    let summary = drawer.finish();

    for usage in &summary.component_usages {
        for slot in &usage.slots {
            let Some(range) = slot_navigation_source_range(Some(template_source), slot) else {
                continue;
            };
            if relative_offset >= range.start && relative_offset <= range.end {
                return Some((slot.name.to_string(), usage.name.to_string()));
            }
        }
    }

    None
}

fn slot_navigation_source_range(
    template_source: Option<&str>,
    slot: &SlotUsage,
) -> Option<std::ops::Range<usize>> {
    if slot.name_is_dynamic {
        return None;
    }
    let name = slot.name.as_str();
    if name.is_empty() {
        return None;
    }

    let start = slot.start as usize;
    let end = slot.end as usize;
    let source = template_source?;
    let raw = source.get(start..end)?;
    if !(raw.contains('#') || raw.contains("v-slot")) {
        return None;
    }
    raw.find(name)
        .map(|relative_start| start + relative_start..start + relative_start + name.len())
}

fn find_slot_in_define_slots(content: &str, slot_name: &str) -> Option<usize> {
    let mut search_start = 0;
    while let Some(relative) = content[search_start..].find(slot_name) {
        let pos = search_start + relative;
        let end = pos + slot_name.len();
        if is_inside_define_slots_type(content, pos) && is_slot_key_at(content, pos, end) {
            return Some(pos);
        }
        search_start = end;
    }

    None
}

fn is_slot_key_at(content: &str, start: usize, end: usize) -> bool {
    let bytes = content.as_bytes();
    if let Some(quote) = start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .filter(|byte| matches!(byte, b'\'' | b'"'))
    {
        return bytes.get(end) == Some(quote);
    }

    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    if before.is_some_and(|byte| is_identifier_byte(*byte)) {
        return false;
    }

    content.get(end..).is_some_and(|tail| {
        tail.starts_with('(') || tail.starts_with(':') || tail.starts_with("?:")
    })
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte == b'-'
}

fn is_inside_define_slots_type(content: &str, pos: usize) -> bool {
    let before = &content[..pos];
    before.matches('<').count() > before.matches('>').count()
        && before.matches('{').count() > before.matches('}').count()
}
