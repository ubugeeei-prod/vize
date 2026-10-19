use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};
use vize_croquis::{Drawer, DrawerOptions, croquis::SlotUsage};

use super::{IdeContext, helpers};

pub(super) fn component_slot_definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    let (slot_name, component_name) = component_slot_at_offset(ctx)?;
    let import_path = helpers::find_import_path(ctx, &component_name)
        .or_else(|| super::art::component_path(ctx, &component_name))?;
    let resolved_path = helpers::resolve_import_path(ctx.uri, &import_path)?;
    let component_content = std::fs::read_to_string(&resolved_path).ok()?;

    let descriptor = ctx
        .state
        .component_descriptor(&resolved_path, &component_content)?;
    let script_setup = descriptor.script_setup.as_ref()?;
    let script = script_setup.content.as_ref();
    let define_slots_pos = script.find("defineSlots")?;
    let slot_pos = find_slot_in_define_slots(script.get(define_slots_pos..)?, &slot_name)?;
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
    let descriptor = ctx.descriptor()?;
    let template = descriptor.template.as_ref()?;
    if ctx.offset < template.loc.start || ctx.offset > template.loc.end {
        return None;
    }
    let relative_offset = ctx.offset.saturating_sub(template.loc.start);
    let template_source = template.content.as_ref();

    let allocator = vize_l0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template_source);
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_template_scopes: true,
        track_usage: true,
        ..Default::default()
    });
    drawer.draw_template(&root);
    let summary = drawer.finish();

    for usage in vize_croquis::facts::component_usage_list(&summary) {
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
    let raw = template_source?.get(start..end)?;
    // The name is the directive argument; `v-slot` alone names no slot here.
    let argument = raw
        .strip_prefix('#')
        .or_else(|| raw.strip_prefix("v-slot:"))?;
    if !argument.starts_with(name) {
        return None;
    }
    let name_start = start + (raw.len() - argument.len());
    Some(name_start..name_start + name.len())
}

/// Offset of `slot_name` as a direct member key of `defineSlots<{ … }>`.
/// Nested keys (slot props) and `=>` in member types are not slot keys.
fn find_slot_in_define_slots(content: &str, slot_name: &str) -> Option<usize> {
    let bytes = content.as_bytes();
    let open = content.find('<')?;
    let mut index = open + 1;
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    if bytes.get(index) != Some(&b'{') {
        return None;
    }
    let mut depth = 0usize;
    let mut member_start = false;
    while let Some(&byte) = bytes.get(index) {
        let next = bytes.get(index + 1).copied();
        match byte {
            b'/' if next == Some(b'/') => {
                index =
                    memchr::memchr(b'\n', bytes.get(index..)?).map_or(bytes.len(), |n| index + n);
                continue;
            }
            b'/' if next == Some(b'*') => {
                index = memchr::memmem::find(bytes.get(index + 2..)?, b"*/")
                    .map_or(bytes.len(), |n| index + 2 + n + 2);
                continue;
            }
            b'{' | b'(' | b'[' => {
                depth += 1;
                member_start = depth == 1;
            }
            b'}' | b')' | b']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return None;
                }
            }
            b';' | b',' | b'\n' if depth == 1 => member_start = true,
            b'\'' | b'"' | b'`' => {
                let end = string_end(bytes, index)?;
                if depth == 1 && member_start {
                    if content.get(index + 1..end) == Some(slot_name)
                        && is_member_key_end(bytes, end + 1)
                    {
                        return Some(index + 1);
                    }
                    member_start = false;
                }
                index = end + 1;
                continue;
            }
            _ if depth == 1 && member_start && is_identifier_byte(byte) => {
                let end = bytes
                    .get(index..)?
                    .iter()
                    .position(|byte| !is_identifier_byte(*byte))
                    .map_or(bytes.len(), |n| index + n);
                if content.get(index..end) == Some(slot_name) && is_member_key_end(bytes, end) {
                    return Some(index);
                }
                member_start = false;
                index = end;
                continue;
            }
            _ if depth == 1 && !byte.is_ascii_whitespace() => member_start = false,
            _ => {}
        }
        index += 1;
    }

    None
}

fn string_end(bytes: &[u8], open: usize) -> Option<usize> {
    let quote = *bytes.get(open)?;
    let mut index = open + 1;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'\\' {
            index += 2;
            continue;
        }
        if byte == quote {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn is_member_key_end(bytes: &[u8], end: usize) -> bool {
    bytes
        .get(end..)
        .and_then(|tail| tail.iter().find(|byte| !matches!(byte, b' ' | b'\t')))
        .is_some_and(|byte| matches!(byte, b'?' | b':' | b'('))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}
