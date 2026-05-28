//! Template definition lookup.
//!
//! Handles go-to-definition for template expressions, component tags,
//! and prop references.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_relief::BindingType;

use super::{IdeContext, helpers};
use crate::ide::{is_component_tag, kebab_to_pascal};

/// Find definition for a symbol in template context.
pub(crate) fn definition_in_template(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
    if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset)
        && is_component_tag(&tag_name)
        && let Some(def) = find_component_definition(ctx, &tag_name)
    {
        return Some(def);
    }

    // Check if this is a component attribute (e.g., :disabled -> component's props)
    if let Some(def) = find_component_prop_definition(ctx) {
        return Some(def);
    }

    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

    if word.is_empty() {
        return None;
    }

    if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
        return None;
    }

    // Check if this is a props property access (e.g., props.title -> defineProps)
    if let Some(def) = find_props_property_definition(ctx, &word) {
        return Some(def);
    }

    if crate::utils::is_standalone_html_path(ctx.uri.path())
        && let Some(def) = find_standalone_html_scope_definition(ctx, &word)
    {
        return Some(def);
    }

    if ctx.state.lsp_features().legacy_vue2
        && let Some(location) = super::script::find_analyzed_binding_location(ctx, &word, true)
    {
        return Some(GotoDefinitionResponse::Scalar(location));
    }

    // Parse SFC to get the actual script content (not virtual code)
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

    // Check if this word is a prop name (props are available directly in template)
    if helpers::is_in_vue_directive_expression(ctx)
        && let Some(def) = find_prop_definition_by_name(ctx, &descriptor, &word)
    {
        return Some(def);
    }

    // Try to find the binding in script setup
    if let Some(ref script_setup) = descriptor.script_setup {
        let content = script_setup.content.as_ref();
        if let Some(binding_loc) = super::script::find_binding_location_raw(content, &word) {
            let sfc_offset = script_setup.loc.start + binding_loc.offset;
            let (line, character) = helpers::offset_to_position(&ctx.content, sfc_offset);

            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: ctx.uri.clone(),
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + word.len() as u32,
                    },
                },
            }));
        }
    }

    // Try regular script block
    if let Some(ref script) = descriptor.script {
        let content = script.content.as_ref();
        if let Some(binding_loc) = super::script::find_binding_location_raw(content, &word) {
            let sfc_offset = script.loc.start + binding_loc.offset;
            let (line, character) = helpers::offset_to_position(&ctx.content, sfc_offset);

            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: ctx.uri.clone(),
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + word.len() as u32,
                    },
                },
            }));
        }
    }

    None
}

fn find_standalone_html_scope_definition(
    ctx: &IdeContext<'_>,
    word: &str,
) -> Option<GotoDefinitionResponse> {
    if !crate::utils::is_petite_vue_document(&ctx.content) {
        return None;
    }

    let (cursor_start, _) =
        crate::ide::token_span_at_offset(&ctx.content, ctx.offset, helpers::is_word_char)?;

    for range in v_scope_value_ranges(&ctx.content)
        .into_iter()
        .chain(create_app_object_ranges(&ctx.content))
    {
        if let Some(offset) =
            find_object_property_key_in_range(&ctx.content, range, word, cursor_start)
        {
            return Some(location_response(ctx, offset, word.len()));
        }
    }

    None
}

fn v_scope_value_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let bytes = content.as_bytes();
    let mut search_start = 0;

    while let Some(relative) = content[search_start..].find("v-scope") {
        let attr_start = search_start + relative;
        let mut pos = attr_start + "v-scope".len();

        if pos < bytes.len()
            && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'-' || bytes[pos] == b'_')
        {
            search_start = pos;
            continue;
        }

        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'=') {
            search_start = pos;
            continue;
        }
        pos += 1;
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        let Some(&quote) = bytes.get(pos) else {
            break;
        };
        if quote != b'"' && quote != b'\'' {
            search_start = pos + 1;
            continue;
        }
        let value_start = pos + 1;
        let Some(relative_end) = content[value_start..].find(quote as char) else {
            break;
        };
        let value_end = value_start + relative_end;
        ranges.push((value_start, value_end));
        search_start = value_end + 1;
    }

    ranges
}

fn create_app_object_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let bytes = content.as_bytes();
    let mut search_start = 0;

    while let Some(relative) = content[search_start..].find("createApp") {
        let name_start = search_start + relative;
        if name_start > 0 && helpers::is_word_char(bytes[name_start - 1]) {
            search_start = name_start + "createApp".len();
            continue;
        }

        let mut pos = name_start + "createApp".len();
        if pos < bytes.len() && helpers::is_word_char(bytes[pos]) {
            search_start = pos;
            continue;
        }

        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'(') {
            search_start = pos;
            continue;
        }
        pos += 1;
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'{') {
            search_start = pos;
            continue;
        }

        if let Some(end) = find_matching_byte(content, pos, b'{', b'}') {
            ranges.push((pos + 1, end));
            search_start = end + 1;
        } else {
            break;
        }
    }

    ranges
}

fn find_object_property_key_in_range(
    content: &str,
    range: (usize, usize),
    word: &str,
    cursor_start: usize,
) -> Option<usize> {
    let (start, end) = range;
    let bytes = content.as_bytes();
    let mut search_start = start;

    while search_start < end {
        let relative = content[search_start..end].find(word)?;
        let key_start = search_start + relative;
        let key_end = key_start + word.len();

        if key_start != cursor_start
            && is_identifier_boundary(bytes, key_start, key_end)
            && is_property_key_tail(bytes, key_end, end)
        {
            return Some(key_start);
        }

        search_start = key_end;
    }

    None
}

fn is_identifier_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    let after = bytes.get(end);
    !before.is_some_and(|byte| helpers::is_word_char(*byte))
        && !after.is_some_and(|byte| helpers::is_word_char(*byte))
}

fn is_property_key_tail(bytes: &[u8], key_end: usize, range_end: usize) -> bool {
    let mut pos = key_end;
    while pos < range_end && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }

    pos >= range_end || matches!(bytes[pos], b':' | b'(' | b',' | b'}')
}

fn find_matching_byte(content: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    let mut pos = start;

    while pos < bytes.len() {
        let byte = bytes[pos];
        if let Some(current_quote) = quote {
            if byte == b'\\' {
                pos += 2;
                continue;
            }
            if byte == current_quote {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' || byte == b'`' {
            quote = Some(byte);
        } else if byte == open {
            depth += 1;
        } else if byte == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(pos);
            }
        }
        pos += 1;
    }

    None
}

fn location_response(ctx: &IdeContext<'_>, offset: usize, len: usize) -> GotoDefinitionResponse {
    let (line, character) = helpers::offset_to_position(&ctx.content, offset);
    GotoDefinitionResponse::Scalar(Location {
        uri: ctx.uri.clone(),
        range: Range {
            start: Position { line, character },
            end: Position {
                line,
                character: character + len as u32,
            },
        },
    })
}

/// Find the definition of a props property (e.g., props.title -> defineProps).
pub(crate) fn find_props_property_definition(
    ctx: &IdeContext<'_>,
    property_name: &str,
) -> Option<GotoDefinitionResponse> {
    let mut word_start = ctx.offset;
    while word_start > 0 && helpers::is_word_char(ctx.content.as_bytes()[word_start - 1]) {
        word_start -= 1;
    }

    if word_start < 6 {
        return None;
    }

    let prefix = &ctx.content[word_start.saturating_sub(6)..word_start];
    if prefix != "props." {
        return None;
    }

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

    if let Some(ref script_setup) = descriptor.script_setup {
        let content = &script_setup.content;

        if let Some(define_props_pos) = content.find("defineProps") {
            let after_define_props = &content[define_props_pos..];

            if let Some(prop_pos) =
                helpers::find_prop_in_define_props(after_define_props, property_name)
            {
                let sfc_offset = script_setup.loc.start + define_props_pos + prop_pos;
                let (line, character) = helpers::offset_to_position(&ctx.content, sfc_offset);

                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position { line, character },
                        end: Position {
                            line,
                            character: character + property_name.len() as u32,
                        },
                    },
                }));
            }

            // Fallback: jump to defineProps call itself
            let sfc_offset = script_setup.loc.start + define_props_pos;
            let (line, character) = helpers::offset_to_position(&ctx.content, sfc_offset);

            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: ctx.uri.clone(),
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + "defineProps".len() as u32,
                    },
                },
            }));
        }
    }

    None
}

/// Find component prop definition from an attribute like :disabled or v-bind:disabled.
pub(crate) fn find_component_prop_definition(
    ctx: &IdeContext<'_>,
) -> Option<GotoDefinitionResponse> {
    let (attr_name, component_name) = helpers::get_attribute_and_component_at_offset(ctx)?;

    if !is_component_tag(&component_name) {
        return None;
    }

    let import_path = helpers::find_import_path(ctx, &component_name)
        .or_else(|| art_component_path(ctx, &component_name))?;
    let resolved_path = helpers::resolve_import_path(ctx.uri, &import_path)?;
    let component_content = std::fs::read_to_string(&resolved_path).ok()?;

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: resolved_path.to_string_lossy().to_string().into(),
        ..Default::default()
    };

    let descriptor = vize_atelier_sfc::parse_sfc(&component_content, options).ok()?;

    let prop_name = helpers::kebab_to_camel(&attr_name);

    if let Some(ref script_setup) = descriptor.script_setup {
        let content = &script_setup.content;

        if let Some(define_props_pos) = content.find("defineProps") {
            let after_define_props = &content[define_props_pos..];

            if let Some(prop_pos) =
                helpers::find_prop_in_define_props(after_define_props, &prop_name)
            {
                let sfc_offset = script_setup.loc.start + define_props_pos + prop_pos;
                let (line, character) = helpers::offset_to_position(&component_content, sfc_offset);

                let file_uri = tower_lsp::lsp_types::Url::from_file_path(&resolved_path).ok()?;
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: file_uri,
                    range: Range {
                        start: Position { line, character },
                        end: Position {
                            line,
                            character: character + prop_name.len() as u32,
                        },
                    },
                }));
            }

            // Fallback: jump to defineProps
            let sfc_offset = script_setup.loc.start + define_props_pos;
            let (line, character) = helpers::offset_to_position(&component_content, sfc_offset);

            let file_uri = tower_lsp::lsp_types::Url::from_file_path(&resolved_path).ok()?;
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: file_uri,
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + "defineProps".len() as u32,
                    },
                },
            }));
        }
    }

    if ctx.state.lsp_features().legacy_vue2 {
        use vize_atelier_sfc::croquis::{
            SfcCroquisOptions, analyze_sfc_descriptor_with_context_legacy_vue2,
        };

        let analysis = analyze_sfc_descriptor_with_context_legacy_vue2(
            &descriptor,
            None,
            SfcCroquisOptions::full(),
        );
        if let Some(&(start, end)) = analysis.croquis.binding_spans.get(prop_name.as_str())
            && end > start
        {
            let sfc_offset = analysis.script_offset + start;
            let (line, character) =
                helpers::offset_to_position(&component_content, sfc_offset as usize);

            let file_uri = tower_lsp::lsp_types::Url::from_file_path(&resolved_path).ok()?;
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: file_uri,
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + (end - start),
                    },
                },
            }));
        }
    }

    None
}

/// Find the definition of a component by its tag name.
pub(crate) fn find_component_definition(
    ctx: &IdeContext<'_>,
    tag_name: &str,
) -> Option<GotoDefinitionResponse> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());

    if let Some(ref script_setup) = descriptor.script_setup {
        analyzer.analyze_script_setup(&script_setup.content);
    } else if let Some(ref script) = descriptor.script {
        analyzer.analyze_script_plain(&script.content);
    }

    let summary = analyzer.finish();

    let pascal_name = kebab_to_pascal(tag_name);
    let names_to_try = [tag_name.to_string(), pascal_name];

    for name in &names_to_try {
        if let Some(import_path) = helpers::find_import_path(ctx, name)
            && let Some(resolved) = helpers::resolve_import_path(ctx.uri, &import_path)
            && let Ok(file_uri) = tower_lsp::lsp_types::Url::from_file_path(&resolved)
        {
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: file_uri,
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
            }));
        }

        if let Some(binding_type) = summary.get_binding_type(name)
            && binding_type == BindingType::ExternalModule
            && let Some(import_path) = helpers::find_import_path(ctx, name)
            && let Some(resolved) = helpers::resolve_import_path(ctx.uri, &import_path)
            && let Ok(file_uri) = tower_lsp::lsp_types::Url::from_file_path(&resolved)
        {
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: file_uri,
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
            }));
        }
    }

    if let Some(import_path) = art_component_path(ctx, tag_name)
        && let Some(resolved) = helpers::resolve_import_path(ctx.uri, &import_path)
        && let Ok(file_uri) = tower_lsp::lsp_types::Url::from_file_path(&resolved)
    {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: file_uri,
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
        }));
    }

    None
}

fn art_component_path(ctx: &IdeContext<'_>, component_name: &str) -> Option<String> {
    if !ctx.uri.path().ends_with(".art.vue") {
        return None;
    }

    let allocator = vize_carton::Bump::new();
    let art_desc = vize_musea::parse_art(
        &allocator,
        &ctx.content,
        vize_musea::ArtParseOptions::default(),
    )
    .ok()?;
    let component_path = art_desc.metadata.component?;
    let descriptor = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    if let Some(script_setup) = descriptor.script_setup.as_ref()
        && let Some(defined_component) =
            crate::virtual_code::find_define_art_component_name(script_setup.content.as_ref())
    {
        let pascal_component = kebab_to_pascal(component_name);
        if component_name == defined_component || pascal_component == defined_component {
            return Some(component_path.to_string());
        }
    }

    let stem = std::path::Path::new(component_path)
        .file_stem()
        .and_then(|stem| stem.to_str())?;

    let pascal_component = kebab_to_pascal(component_name);
    let pascal_stem = kebab_to_pascal(stem);
    (component_name == stem || pascal_component == pascal_stem).then(|| component_path.to_string())
}

/// Find definition for a prop name used directly in template.
pub(crate) fn find_prop_definition_by_name(
    ctx: &IdeContext<'_>,
    descriptor: &vize_atelier_sfc::SfcDescriptor,
    prop_name: &str,
) -> Option<GotoDefinitionResponse> {
    let script_setup = descriptor.script_setup.as_ref()?;

    let mut analyzer = Analyzer::with_options(AnalyzerOptions {
        analyze_script: true,
        ..Default::default()
    });
    analyzer.analyze_script_setup(&script_setup.content);
    let croquis = analyzer.finish();

    let props = croquis.macros.props();
    let is_prop = props.iter().any(|p| p.name.as_str() == prop_name);

    if !is_prop {
        return None;
    }

    let content = &script_setup.content;
    if let Some(define_props_pos) = content.find("defineProps") {
        let after_define_props = &content[define_props_pos..];

        if let Some(prop_pos) = helpers::find_prop_in_define_props(after_define_props, prop_name) {
            let sfc_offset = script_setup.loc.start + define_props_pos + prop_pos;
            let (line, character) = helpers::offset_to_position(&ctx.content, sfc_offset);

            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: ctx.uri.clone(),
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + prop_name.len() as u32,
                    },
                },
            }));
        }

        // Fallback: jump to defineProps
        let sfc_offset = script_setup.loc.start + define_props_pos;
        let (line, character) = helpers::offset_to_position(&ctx.content, sfc_offset);

        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: ctx.uri.clone(),
            range: Range {
                start: Position { line, character },
                end: Position {
                    line,
                    character: character + "defineProps".len() as u32,
                },
            },
        }));
    }

    None
}
