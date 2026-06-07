//! Script and style definition lookup.
//!
//! Handles go-to-definition within script blocks and v-bind() in styles.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};

use super::{
    IdeContext,
    bindings::{BindingKind, BindingLocation},
    helpers,
};
use crate::virtual_code::BlockType;
use vize_carton::cstr;

/// Find definition for a symbol in script context.
pub(crate) fn definition_in_script(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
    if ctx.uri.path().ends_with(".art.vue")
        && let Some(source) =
            crate::ide::musea::define_art_source_at_offset(&ctx.content, ctx.uri, ctx.offset)
        && let Some(target) = crate::ide::musea::resolve_define_art_source(ctx.uri, &source.source)
        && let Ok(uri) = tower_lsp::lsp_types::Url::from_file_path(target)
    {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri,
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

    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

    if word.is_empty() {
        return None;
    }

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

    let is_setup = matches!(ctx.block_type, Some(BlockType::ScriptSetup));

    let script_block = if is_setup {
        descriptor.script_setup.as_ref()
    } else {
        descriptor.script.as_ref()
    };

    if let Some(script) = script_block {
        let content = script.content.as_ref();
        if let Some(binding_loc) = find_binding_location_raw(content, &word) {
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

    if ctx.state.options_api_enabled()
        && let Some(location) = find_analyzed_binding_location(ctx, &word)
    {
        return Some(GotoDefinitionResponse::Scalar(location));
    }

    None
}

/// Find a binding location from Croquis analysis, including opt-in Options API spans.
pub(crate) fn find_analyzed_binding_location(ctx: &IdeContext, word: &str) -> Option<Location> {
    use vize_atelier_sfc::{
        SfcParseOptions,
        croquis::{
            SfcCroquisOptions, analyze_sfc_descriptor_with_context,
            analyze_sfc_descriptor_with_context_legacy_vue2,
            analyze_sfc_descriptor_with_context_options_api,
        },
        parse_sfc,
    };

    let descriptor = parse_sfc(
        &ctx.content,
        SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;

    let croquis_options = SfcCroquisOptions::full();
    let analysis = if ctx.state.legacy_vue2_enabled() {
        analyze_sfc_descriptor_with_context_legacy_vue2(&descriptor, None, croquis_options)
    } else if ctx.state.options_api_enabled() {
        analyze_sfc_descriptor_with_context_options_api(&descriptor, None, croquis_options)
    } else {
        analyze_sfc_descriptor_with_context(&descriptor, None, croquis_options)
    };
    let &(start, end) = analysis.croquis.binding_spans.get(word)?;
    if end <= start {
        return None;
    }

    let offset = analysis.script_offset as usize + start as usize;
    Some(location_from_sfc_offset(
        ctx,
        offset,
        (end - start) as usize,
    ))
}

pub(crate) fn location_from_sfc_offset(ctx: &IdeContext, offset: usize, len: usize) -> Location {
    let (line, character) = helpers::offset_to_position(&ctx.content, offset);

    Location {
        uri: ctx.uri.clone(),
        range: Range {
            start: Position { line, character },
            end: Position {
                line,
                character: character + len as u32,
            },
        },
    }
}

/// Find definition for a symbol in style context.
pub(crate) fn definition_in_style(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

    if word.is_empty() {
        return None;
    }

    // Check for v-bind() references to script variables.
    if is_inside_style_v_bind_argument(&ctx.content, ctx.offset) {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options)
            && let Some(ref script_setup) = descriptor.script_setup
        {
            let content = script_setup.content.as_ref();
            if let Some(binding_loc) = find_binding_location_raw(content, &word) {
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
    }

    None
}

fn is_inside_style_v_bind_argument(content: &str, offset: usize) -> bool {
    let mut offset = offset.min(content.len());
    while offset > 0 && !content.is_char_boundary(offset) {
        offset -= 1;
    }

    let Some(v_bind_start) = content[..offset].rfind("v-bind(") else {
        return false;
    };
    let arg_start = v_bind_start + "v-bind(".len();

    !content[arg_start..offset].contains(')')
}

/// Find the location of a binding definition in raw script content (not virtual code).
pub(crate) fn find_binding_location_raw(content: &str, name: &str) -> Option<BindingLocation> {
    let patterns = [
        cstr!("const {name} "),
        cstr!("const {name}="),
        cstr!("const {name}:"),
        cstr!("let {name} "),
        cstr!("let {name}="),
        cstr!("let {name}:"),
        cstr!("var {name} "),
        cstr!("var {name}="),
        cstr!("function {name}("),
        cstr!("function {name} ("),
    ];

    for pattern in &patterns {
        if let Some(pos) = content.find(pattern.as_str()) {
            let name_offset = pattern.find(name).unwrap_or(0);
            let actual_offset = pos + name_offset;

            return Some(BindingLocation {
                name: name.to_string(),
                offset: actual_offset,
                kind: BindingKind::from_pattern(pattern),
            });
        }
    }

    // Check for destructuring patterns
    let destructure_patterns = [
        cstr!("{{ {name} }}"),
        cstr!("{{ {name}, "),
        cstr!("{{ {name} ,"),
        cstr!(", {name} }}"),
        cstr!(", {name}, "),
        cstr!(" {name} }}"),
        cstr!(" {name}, "),
    ];

    for pattern in &destructure_patterns {
        if let Some(pos) = content.find(pattern.as_str()) {
            let name_offset = pattern.find(name).unwrap_or(0);
            let actual_offset = pos + name_offset;

            return Some(BindingLocation {
                name: name.to_string(),
                offset: actual_offset,
                kind: BindingKind::Destructure,
            });
        }
    }

    // Check for import patterns
    let import_patterns = [
        cstr!("import {name} from"),
        cstr!("import {{ {name} }}"),
        cstr!("import {{ {name}, "),
        cstr!("import {{ {name} ,"),
        cstr!(", {name} }}"),
    ];

    for pattern in &import_patterns {
        if let Some(pos) = content.find(pattern.as_str()) {
            let name_offset = pattern.find(name).unwrap_or(0);
            let actual_offset = pos + name_offset;

            return Some(BindingLocation {
                name: name.to_string(),
                offset: actual_offset,
                kind: BindingKind::Import,
            });
        }
    }

    None
}

/// Find the location of a binding definition in script content.
#[allow(dead_code)]
pub(crate) fn find_binding_location(
    content: &str,
    name: &str,
    _is_setup: bool,
) -> Option<BindingLocation> {
    let content_start = helpers::skip_virtual_header(content);
    let search_content = &content[content_start..];

    let patterns = [
        cstr!("const {name} "),
        cstr!("const {name}="),
        cstr!("let {name} "),
        cstr!("let {name}="),
        cstr!("var {name} "),
        cstr!("var {name}="),
        cstr!("function {name}("),
        cstr!("function {name} ("),
    ];

    for pattern in &patterns {
        if let Some(pos) = search_content.find(pattern.as_str()) {
            let name_offset = pattern.find(name).unwrap_or(0);
            let actual_offset = content_start + pos + name_offset;

            return Some(BindingLocation {
                name: name.to_string(),
                offset: actual_offset,
                kind: BindingKind::from_pattern(pattern),
            });
        }
    }

    // Check for destructuring patterns
    let destructure_pattern = cstr!("{{ {name}");
    if let Some(pos) = search_content.find(destructure_pattern.as_str()) {
        let name_offset = destructure_pattern.find(name).unwrap_or(0);
        let actual_offset = content_start + pos + name_offset;

        return Some(BindingLocation {
            name: name.to_string(),
            offset: actual_offset,
            kind: BindingKind::Destructure,
        });
    }

    let destructure_patterns = [
        cstr!("{{ {name}, "),
        cstr!("{{ {name} }}"),
        cstr!(", {name} }}"),
        cstr!(", {name}, "),
    ];

    for pattern in &destructure_patterns {
        if let Some(pos) = search_content.find(pattern.as_str()) {
            let name_offset = pattern.find(name).unwrap_or(0);
            let actual_offset = content_start + pos + name_offset;

            return Some(BindingLocation {
                name: name.to_string(),
                offset: actual_offset,
                kind: BindingKind::Destructure,
            });
        }
    }

    None
}
