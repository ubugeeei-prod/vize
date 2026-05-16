//! Template definition lookup.
//!
//! Handles go-to-definition for template expressions, component tags,
//! and prop references.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_relief::BindingType;

use super::{helpers, IdeContext};
use crate::ide::{is_component_tag, kebab_to_pascal};

/// Find definition for a symbol in template context.
pub(crate) fn definition_in_template(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
    if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset) {
        if is_component_tag(&tag_name) {
            if let Some(def) = find_component_definition(ctx, &tag_name) {
                return Some(def);
            }
        }
    }

    // Check if this is a component attribute (e.g., :disabled -> component's props)
    if let Some(def) = find_component_prop_definition(ctx) {
        return Some(def);
    }

    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

    if word.is_empty() {
        return None;
    }

    // Check if this is a props property access (e.g., props.title -> defineProps)
    if let Some(def) = find_props_property_definition(ctx, &word) {
        return Some(def);
    }

    // Parse SFC to get the actual script content (not virtual code)
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

    // Check if this word is a prop name (props are available directly in template)
    if helpers::is_in_vue_directive_expression(ctx) {
        if let Some(def) = find_prop_definition_by_name(ctx, &descriptor, &word) {
            return Some(def);
        }
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

    let import_path = helpers::find_import_path(ctx, &component_name)?;
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
        if let Some(import_path) = helpers::find_import_path(ctx, name) {
            if let Some(resolved) = helpers::resolve_import_path(ctx.uri, &import_path) {
                if let Ok(file_uri) = tower_lsp::lsp_types::Url::from_file_path(&resolved) {
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
        }

        if let Some(binding_type) = summary.get_binding_type(name) {
            if binding_type == BindingType::ExternalModule {
                if let Some(import_path) = helpers::find_import_path(ctx, name) {
                    if let Some(resolved) = helpers::resolve_import_path(ctx.uri, &import_path) {
                        if let Ok(file_uri) = tower_lsp::lsp_types::Url::from_file_path(&resolved) {
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
                }
            }
        }
    }

    None
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
