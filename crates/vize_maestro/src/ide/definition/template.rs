//! Template go-to-definition for expressions, component tags, and props.

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};
use vize_croquis::{Drawer, DrawerOptions};
use vize_relief::BindingType;

use super::{IdeContext, helpers};
use crate::ide::{component_name_candidates, is_component_tag, template_scope};

mod scope_ranges;
use scope_ranges::{
    create_app_object_ranges, find_object_property_key_in_range, v_scope_value_ranges,
};

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

    if let Some(def) = super::slot::component_slot_definition(ctx) {
        return Some(def);
    }

    let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

    if word.is_empty() {
        return None;
    }

    if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
        return None;
    }

    if let Some(def) = find_props_property_definition(ctx, &word) {
        return Some(def);
    }

    if crate::utils::is_standalone_html_path(ctx.uri.path())
        && let Some(def) = find_standalone_html_scope_definition(ctx, &word)
    {
        return Some(def);
    }

    if let Some(definition) = template_scope::definition(ctx, &word) {
        return Some(definition);
    }
    if let Some(location) = super::script::find_analyzed_binding_location(ctx, &word) {
        return Some(GotoDefinitionResponse::Scalar(location));
    }

    // Parse SFC to get the actual script content (not virtual code)
    let descriptor = ctx.descriptor()?;

    // Check if this word is a prop name (props are available directly in template)
    if helpers::is_in_vue_directive_expression(ctx)
        && let Some(def) = find_prop_definition_by_name(ctx, descriptor, &word)
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
    if !ctx.dialect().is_petite_vue() {
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
    while word_start > 0
        && ctx
            .content
            .as_bytes()
            .get(word_start - 1)
            .is_some_and(|&b| helpers::is_word_char(b))
    {
        word_start -= 1;
    }

    if word_start < 6 {
        return None;
    }

    // Use checked slicing — non-ASCII text right before the identifier could
    // place `word_start - 6` mid-codepoint, panicking the LSP. Skip the prefix
    // check on a non-boundary instead of slicing. (#964)
    let prefix_start = word_start.saturating_sub(6);
    let prefix = ctx.content.get(prefix_start..word_start)?;
    if prefix != "props." {
        return None;
    }

    let descriptor = ctx.descriptor()?;

    if let Some(ref script_setup) = descriptor.script_setup {
        let content = &script_setup.content;

        if let Some(define_props_pos) = content.find("defineProps") {
            let after_define_props = content.get(define_props_pos..).unwrap_or_default();

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

    let resolved_path = super::component_import::resolve_component_file(ctx, &component_name)
        .or_else(|| {
            let import_path = super::art::component_path(ctx, &component_name)?;
            helpers::resolve_import_path(ctx.uri, &import_path)
        })?;
    let component_content = std::fs::read_to_string(&resolved_path).ok()?;

    let descriptor = ctx
        .state
        .component_descriptor(&resolved_path, &component_content)?;

    let prop_name = helpers::kebab_to_camel(&attr_name);

    if let Some(ref script_setup) = descriptor.script_setup {
        let content = &script_setup.content;

        let define_props_pos = content.find("defineProps");
        if let Some(define_props_pos) = define_props_pos {
            let after_define_props = content.get(define_props_pos..).unwrap_or_default();

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
        }

        if let Some((model_pos, model_len)) =
            super::component_model::find_prop_in_define_model(content, &prop_name)
        {
            let sfc_offset = script_setup.loc.start + model_pos;
            let (line, character) = helpers::offset_to_position(&component_content, sfc_offset);

            let file_uri = tower_lsp::lsp_types::Url::from_file_path(&resolved_path).ok()?;
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: file_uri,
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + model_len as u32,
                    },
                },
            }));
        }

        if let Some(define_props_pos) = define_props_pos {
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

    if ctx.state.options_api_enabled() {
        use vize_atelier_sfc::croquis::{
            SfcCroquisOptions, analyze_sfc_descriptor_with_context_legacy_vue2,
            analyze_sfc_descriptor_with_context_options_api,
        };

        let analysis = if ctx.state.legacy_vue2_enabled() {
            analyze_sfc_descriptor_with_context_legacy_vue2(
                &descriptor,
                None,
                SfcCroquisOptions::full(),
            )
        } else {
            analyze_sfc_descriptor_with_context_options_api(
                &descriptor,
                None,
                SfcCroquisOptions::full(),
            )
        };
        if let Some((start, end)) = super::facts::prop_span(&analysis.croquis, &prop_name)
            && end > start
        {
            let sfc_offset = analysis.script_source_offset(&descriptor, start);
            let (line, character) =
                helpers::offset_to_position(&component_content, sfc_offset as usize);

            let file_uri = tower_lsp::lsp_types::Url::from_file_path(&resolved_path).ok()?;
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: file_uri,
                range: Range {
                    start: Position { line, character },
                    end: Position {
                        line,
                        character: character + analysis.script_source_len(&descriptor, start, end),
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
    if tag_name == "Self" {
        return super::inline_art::self_component_definition(ctx);
    }

    let mut analyzer = Drawer::with_options(DrawerOptions::full());
    let descriptor = ctx.descriptor()?;

    if let Some(ref script_setup) = descriptor.script_setup {
        analyzer.analyze_script_setup(&script_setup.content);
    } else if let Some(ref script) = descriptor.script {
        analyzer.analyze_script_plain(&script.content);
    }

    let summary = analyzer.finish();

    for name in component_name_candidates(tag_name) {
        if let Some(resolved) = super::component_import::resolve_component_file(ctx, &name)
            && let Some(location) = component_file_location(ctx, &resolved)
        {
            return Some(GotoDefinitionResponse::Scalar(location));
        }

        if let Some(binding_type) = summary.get_binding_type(&name)
            && binding_type == BindingType::ExternalModule
            && let Some(resolved) = super::component_import::resolve_component_file(ctx, &name)
            && let Some(location) = component_file_location(ctx, &resolved)
        {
            return Some(GotoDefinitionResponse::Scalar(location));
        }
    }

    if let Some(import_path) = super::art::component_path(ctx, tag_name)
        && let Some(resolved) = helpers::resolve_import_path(ctx.uri, &import_path)
        && let Some(location) = component_file_location(ctx, &resolved)
    {
        return Some(GotoDefinitionResponse::Scalar(location));
    }

    None
}

fn component_file_location(ctx: &IdeContext<'_>, path: &std::path::Path) -> Option<Location> {
    let uri = tower_lsp::lsp_types::Url::from_file_path(path).ok()?;
    if !path.is_file() && !ctx.state.documents.contains(&uri) {
        return None;
    }

    Some(Location {
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
    })
}

/// Find definition for a prop name used directly in template.
pub(crate) fn find_prop_definition_by_name(
    ctx: &IdeContext<'_>,
    descriptor: &vize_atelier_sfc::SfcDescriptor,
    prop_name: &str,
) -> Option<GotoDefinitionResponse> {
    let script_setup = descriptor.script_setup.as_ref()?;

    let mut analyzer = Drawer::with_options(DrawerOptions {
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
        let after_define_props = content.get(define_props_pos..).unwrap_or_default();

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
