//! Component event contract lookup for template listeners.

use std::path::Path;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

use super::{IdeContext, component_import, helpers};

mod attribute;

use attribute::event_at_offset;

pub(crate) struct ComponentEventContract {
    pub(crate) name: String,
    pub(crate) payload_type: String,
    pub(crate) authored_range: Range,
    pub(crate) target_uri: Url,
    pub(crate) target_range: Range,
}

pub(crate) fn contract(ctx: &IdeContext<'_>) -> Option<ComponentEventContract> {
    let event = event_at_offset(ctx)?;
    let resolved_path = component_import::resolve_component_file(ctx, &event.component_name)
        .or_else(|| {
            let import_path = super::art::component_path(ctx, &event.component_name)?;
            helpers::resolve_import_path(ctx.uri, &import_path)
        })?;
    let component_content = std::fs::read_to_string(&resolved_path).ok()?;
    let descriptor = parse_component(&component_content, &resolved_path)?;
    let summary = vize_atelier_sfc::croquis::analyze_sfc_descriptor_resolved(
        &descriptor,
        None,
        vize_atelier_sfc::croquis::SfcCroquisOptions::full(),
        ctx.state.options_api_enabled(),
        ctx.state.legacy_vue2_enabled(),
        &resolved_path.to_string_lossy(),
    )
    .croquis;
    let emit = summary
        .macros
        .emits()
        .iter()
        .find(|emit| emit.name.as_str() == event.name)?;
    let payload_type = emit
        .payload_type
        .as_deref()
        .or_else(|| {
            summary
                .macros
                .define_emits()
                .and_then(|call| call.type_args.as_deref())
                .and_then(|type_args| payload_from_emit_type_args(type_args, &event.name))
        })
        .unwrap_or("unknown[]");
    let target_range = define_emits_event_range(&descriptor, &component_content, &event.name)?;
    let target_uri = Url::from_file_path(&resolved_path).ok()?;

    Some(ComponentEventContract {
        name: event.name,
        payload_type: compact_type(payload_type),
        authored_range: event.range,
        target_uri,
        target_range,
    })
}

pub(crate) fn definition(ctx: &IdeContext<'_>) -> Option<GotoDefinitionResponse> {
    let contract = contract(ctx)?;
    Some(GotoDefinitionResponse::Scalar(Location {
        uri: contract.target_uri,
        range: contract.target_range,
    }))
}

fn parse_component<'a>(
    component_content: &'a str,
    resolved_path: &Path,
) -> Option<vize_atelier_sfc::SfcDescriptor<'a>> {
    vize_atelier_sfc::parse_sfc(
        component_content,
        vize_atelier_sfc::SfcParseOptions {
            filename: resolved_path.to_string_lossy().to_string().into(),
            ..Default::default()
        },
    )
    .ok()
}

fn define_emits_event_range(
    descriptor: &vize_atelier_sfc::SfcDescriptor,
    component_content: &str,
    event_name: &str,
) -> Option<Range> {
    let script_setup = descriptor.script_setup.as_ref()?;
    let script = script_setup.content.as_ref();
    let define_emits_pos = script.find("defineEmits")?;
    let event_pos = find_event_in_define_emits(&script[define_emits_pos..], event_name)
        .map(|pos| define_emits_pos + pos)
        .unwrap_or(define_emits_pos);
    let start = script_setup.loc.start + event_pos;
    let end = start + event_name.len();
    let (start_line, start_character) = helpers::offset_to_position(component_content, start);
    let (end_line, end_character) = helpers::offset_to_position(component_content, end);
    Some(Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    })
}

fn find_event_in_define_emits(content: &str, event_name: &str) -> Option<usize> {
    let mut search_start = 0;
    while let Some(relative) = content[search_start..].find(event_name) {
        let start = search_start + relative;
        let end = start + event_name.len();
        if is_event_key_at(content, start, end) {
            return Some(start);
        }
        search_start = end;
    }
    None
}

fn payload_from_emit_type_args<'a>(type_args: &'a str, event_name: &str) -> Option<&'a str> {
    let event_pos = find_event_in_define_emits(type_args, event_name)?;
    let mut pos = event_pos + event_name.len();
    let bytes = type_args.as_bytes();
    if matches!(bytes.get(pos), Some(b'\'' | b'"')) {
        pos += 1;
    }
    while pos < type_args.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    if bytes.get(pos) == Some(&b'?') {
        pos += 1;
    }
    while pos < type_args.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    if bytes.get(pos) != Some(&b':') {
        return None;
    }
    pos += 1;
    while pos < type_args.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    let start = pos;
    let end = top_level_type_end(type_args, start);
    (end > start).then(|| type_args[start..end].trim())
}

fn top_level_type_end(source: &str, start: usize) -> usize {
    let mut angle = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    let mut paren = 0usize;
    let mut quote = None;
    let mut pos = start;
    while pos < source.len() {
        let ch = source[pos..].chars().next().unwrap();
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
        } else {
            match ch {
                '\'' | '"' => quote = Some(ch),
                '<' => angle += 1,
                '>' => angle = angle.saturating_sub(1),
                '[' => bracket += 1,
                ']' => bracket = bracket.saturating_sub(1),
                '{' => brace += 1,
                '}' if brace == 0 && bracket == 0 && paren == 0 && angle == 0 => break,
                '}' => brace -= 1,
                '(' => paren += 1,
                ')' => paren = paren.saturating_sub(1),
                ';' | ',' if brace == 0 && bracket == 0 && paren == 0 && angle == 0 => break,
                _ => {}
            }
        }
        pos += ch.len_utf8();
    }
    pos
}

fn is_event_key_at(content: &str, start: usize, end: usize) -> bool {
    let bytes = content.as_bytes();
    if let Some(quote) = start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .filter(|byte| matches!(byte, b'\'' | b'"'))
    {
        return bytes.get(end) == Some(quote);
    }
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    if before.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'$') {
        return false;
    }
    content.get(end..).is_some_and(|tail| {
        tail.starts_with(':') || tail.starts_with("?:") || tail.starts_with(',')
    })
}

fn compact_type(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::payload_from_emit_type_args;

    #[test]
    fn extracts_object_style_emit_tuple_payloads() {
        let payload = payload_from_emit_type_args(
            "{ save: [value: string]; close?: []; nested: [{ id: string }] }",
            "save",
        );
        assert_eq!(payload, Some("[value: string]"));
    }

    #[test]
    fn extracts_quoted_event_payloads_without_matching_substrings() {
        let payload =
            payload_from_emit_type_args(r#"{ "save-all": [ids: string[]], save: [] }"#, "save");
        assert_eq!(payload, Some("[]"));
    }
}
