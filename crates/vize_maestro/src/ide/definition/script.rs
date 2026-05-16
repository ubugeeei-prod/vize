//! Script and style definition lookup.
//!
//! Handles go-to-definition within script blocks and v-bind() in styles.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};

use super::{
    bindings::{BindingKind, BindingLocation},
    helpers, IdeContext,
};
use crate::virtual_code::BlockType;
use vize_carton::cstr;

/// Find definition for a symbol in script context.
pub(crate) fn definition_in_script(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
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

    None
}

/// Find definition for a symbol in style context.
pub(crate) fn definition_in_style(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

    if word.is_empty() {
        return None;
    }

    // Check for v-bind() references to script variables
    let before_cursor = &ctx.content[..ctx.offset];
    if before_cursor.contains("v-bind(") {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) {
            if let Some(ref script_setup) = descriptor.script_setup {
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
    }

    None
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
