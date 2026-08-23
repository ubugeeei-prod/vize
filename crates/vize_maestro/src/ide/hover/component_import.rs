//! Hover presentation for imported Vue SFC component contracts.
#![cfg(feature = "native")]
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use std::path::Path;

use tower_lsp::lsp_types::{Hover, HoverContents, Position, Range, Url};

use super::HoverService;
use crate::ide::IdeContext;
use crate::ide::definition::helpers;
use crate::ide::markup;

pub(super) fn rewrite_vue_component_import_hover(
    ctx: &IdeContext<'_>,
    local_name: &str,
    hover: &mut Hover,
) {
    let HoverContents::Markup(ref mut content) = hover.contents else {
        return;
    };
    if !contains_generated_component_marker(&content.value) {
        return;
    }
    let Some(markdown) = component_contract_markdown(ctx, local_name) else {
        return;
    };
    content.value = markdown;
    if hover.range.is_none() {
        hover.range = authored_token_range(ctx);
    }
}

fn contains_generated_component_marker(markdown: &str) -> bool {
    markdown.contains("__vizeComponentMarker")
        || markdown.contains("__vizeRawProps")
        || markdown.contains("__VizeComponentConstructor")
        || markdown.contains("__vize_component__")
}

fn component_contract_markdown(ctx: &IdeContext<'_>, local_name: &str) -> Option<String> {
    let resolved_path = imported_vue_component_path(ctx, local_name)?;
    let source = component_source(ctx, &resolved_path)?;
    let filename = resolved_path.to_string_lossy();
    let descriptor = vize_atelier_sfc::parse_sfc(
        &source,
        vize_atelier_sfc::SfcParseOptions {
            filename: filename.to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let analysis = vize_atelier_sfc::croquis::analyze_sfc_descriptor_resolved(
        &descriptor,
        None,
        vize_atelier_sfc::croquis::SfcCroquisOptions::full(),
        ctx.state.options_api_enabled(),
        ctx.state.legacy_vue2_enabled(),
        &filename,
    );
    let macros = &analysis.croquis.macros;

    let mut lines = vec![format!("const {local_name}: VueComponent")];
    let mut fields = Vec::new();

    if let Some(props) = props_contract(macros) {
        fields.push(format!("  props: {props};"));
    }
    if let Some(emits) = emits_contract(macros) {
        fields.push(format!("  emits: {emits};"));
    }
    if let Some(slots) = slots_contract(macros) {
        fields.push(format!("  slots: {slots};"));
    }
    if let Some(model) = model_contract(macros) {
        fields.push(format!("  model: {model};"));
    }

    if !fields.is_empty() {
        lines.push("{".to_string());
        lines.extend(fields);
        lines.push("}".to_string());
    }

    let mut markdown = markup::code_block("typescript", &lines.join("\n"));
    if let Some(file_name) = resolved_path.file_name().and_then(|name| name.to_str()) {
        markdown.push_str("\n\nVue component: ");
        markdown.push_str(file_name);
    }
    Some(markdown)
}

fn imported_vue_component_path(
    ctx: &IdeContext<'_>,
    local_name: &str,
) -> Option<std::path::PathBuf> {
    let import_path = helpers::find_import_path(ctx, local_name)?;
    let resolved_path = helpers::resolve_import_path(ctx.uri, &import_path)?;
    (resolved_path
        .extension()
        .and_then(|extension| extension.to_str())
        == Some("vue"))
    .then_some(resolved_path)
}

fn component_source(ctx: &IdeContext<'_>, path: &Path) -> Option<String> {
    if let Ok(uri) = Url::from_file_path(path)
        && let Some(source) = ctx.state.documents.text(&uri)
    {
        return Some(source);
    }
    std::fs::read_to_string(path).ok()
}

fn props_contract(macros: &vize_croquis::macros::MacroTracker) -> Option<String> {
    if let Some(type_args) = macros
        .define_props()
        .and_then(|call| call.type_args.as_ref())
    {
        return Some(compact_type_argument(type_args));
    }

    let props = macros.props();
    if props.is_empty() {
        return None;
    }

    let fields = props
        .iter()
        .map(|prop| {
            let optional = if prop.required { "" } else { "?" };
            let prop_type = prop.prop_type.as_deref().unwrap_or("unknown");
            format!("{}{optional}: {}", prop.name, compact_type(prop_type))
        })
        .collect::<Vec<_>>()
        .join("; ");
    Some(format!("{{ {fields} }}"))
}

fn emits_contract(macros: &vize_croquis::macros::MacroTracker) -> Option<String> {
    if let Some(type_args) = macros
        .define_emits()
        .and_then(|call| call.type_args.as_ref())
    {
        return Some(compact_type_argument(type_args));
    }

    let emits = macros.emits();
    if emits.is_empty() {
        return None;
    }

    let fields = emits
        .iter()
        .map(|emit| {
            let payload = emit.payload_type.as_deref().unwrap_or("unknown[]");
            format!("{}: {}", emit.name, compact_type(payload))
        })
        .collect::<Vec<_>>()
        .join("; ");
    Some(format!("{{ {fields} }}"))
}

fn slots_contract(macros: &vize_croquis::macros::MacroTracker) -> Option<String> {
    if let Some(type_args) = macros
        .define_slots()
        .and_then(|call| call.type_args.as_ref())
    {
        return Some(compact_type_argument(type_args));
    }

    let slots = macros.slots();
    if slots.is_empty() {
        return None;
    }

    let fields = slots
        .iter()
        .map(|slot| {
            let payload = slot
                .props_type
                .as_deref()
                .map(compact_type)
                .unwrap_or_else(|| "{}".to_string());
            format!("{}(props: {}): unknown", slot.name, payload)
        })
        .collect::<Vec<_>>()
        .join("; ");
    Some(format!("{{ {fields} }}"))
}

fn model_contract(macros: &vize_croquis::macros::MacroTracker) -> Option<String> {
    let models = macros.models();
    if models.is_empty() {
        return None;
    }

    let fields = models
        .iter()
        .map(|model| {
            let model_type = model.model_type.as_deref().unwrap_or("unknown");
            format!("\"{}\": {}", model.name, compact_type(model_type))
        })
        .collect::<Vec<_>>();

    if fields.len() == 1 {
        fields.into_iter().next()
    } else {
        Some(format!("{{ {} }}", fields.join("; ")))
    }
}

fn compact_type(source: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;
    let mut cursor = 0;
    let bytes = source.as_bytes();

    while cursor < source.len() {
        let byte = bytes[cursor];
        if byte.is_ascii_whitespace() {
            pending_space = !output.is_empty();
            cursor = consume_while(source, cursor, u8::is_ascii_whitespace);
            continue;
        }
        if byte == b'/' && bytes.get(cursor + 1) == Some(&b'/') {
            cursor = consume_line_comment(source, cursor);
            continue;
        }
        if byte == b'/' && bytes.get(cursor + 1) == Some(&b'*') {
            cursor = consume_block_comment(source, cursor);
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'`') {
            if pending_space {
                output.push(' ');
                pending_space = false;
            }
            let end = consume_quoted(source, cursor, byte);
            output.push_str(&source[cursor..end]);
            cursor = end;
            continue;
        }
        if pending_space {
            output.push(' ');
            pending_space = false;
        }
        let ch = source[cursor..]
            .chars()
            .next()
            .expect("cursor points inside source");
        output.push(ch);
        cursor += ch.len_utf8();
    }

    output.trim().to_string()
}

fn compact_type_argument(source: &str) -> String {
    let compact = compact_type(source);
    compact
        .strip_prefix('<')
        .and_then(|body| body.strip_suffix('>'))
        .map(str::trim)
        .unwrap_or(&compact)
        .to_string()
}

fn consume_while(source: &str, start: usize, predicate: fn(&u8) -> bool) -> usize {
    let mut cursor = start;
    let bytes = source.as_bytes();
    while cursor < source.len() && predicate(&bytes[cursor]) {
        cursor += 1;
    }
    cursor
}

fn consume_line_comment(source: &str, start: usize) -> usize {
    source[start + 2..]
        .find('\n')
        .map(|end| start + 2 + end)
        .unwrap_or(source.len())
}

fn consume_block_comment(source: &str, start: usize) -> usize {
    source[start + 2..]
        .find("*/")
        .map(|end| start + 2 + end + 2)
        .unwrap_or(source.len())
}

fn consume_quoted(source: &str, start: usize, quote: u8) -> usize {
    let bytes = source.as_bytes();
    let mut escaped = false;
    let mut cursor = start + 1;
    while cursor < source.len() {
        let byte = bytes[cursor];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return cursor + 1;
        }
        cursor += 1;
    }
    source.len()
}

fn authored_token_range(ctx: &IdeContext<'_>) -> Option<Range> {
    let (start, end) =
        crate::ide::token_span_at_offset(&ctx.content, ctx.offset, HoverService::is_word_char)?;
    let (start_line, start_character) = crate::ide::offset_to_position(&ctx.content, start);
    let (end_line, end_character) = crate::ide::offset_to_position(&ctx.content, end);
    Some(Range::new(
        Position::new(start_line, start_character),
        Position::new(end_line, end_character),
    ))
}

#[cfg(test)]
mod tests {
    use super::compact_type;

    #[test]
    fn compact_type_preserves_literals_and_removes_comments() {
        assert_eq!(
            compact_type(
                r#"{
                    label: "a /* value */";
                    // hidden
                    count?: number
                }"#,
            ),
            r#"{ label: "a /* value */"; count?: number }"#
        );
    }

    #[test]
    fn compact_type_argument_removes_only_the_outer_angle_pair() {
        assert_eq!(
            super::compact_type_argument("<Record<string, { value: number }>>"),
            "Record<string, { value: number }>"
        );
    }
}
