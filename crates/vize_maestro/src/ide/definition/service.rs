//! Definition service entry point and Corsa integration.
//!
//! Provides the main `definition` and `definition_with_corsa` methods
//! that dispatch to block-specific handlers.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

#[cfg(feature = "native")]
use std::sync::Arc;

use tower_lsp::lsp_types::GotoDefinitionResponse;

#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

use super::{helpers, script, template, IdeContext};
#[cfg(feature = "native")]
use crate::ide::corsa_support;
use crate::ide::is_component_tag;
use crate::virtual_code::{ArtCursorPosition, BlockType};

impl super::DefinitionService {
    /// Get definition for the symbol at the current position.
    pub fn definition(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
        match ctx.block_type? {
            BlockType::Template => template::definition_in_template(ctx),
            BlockType::Script | BlockType::ScriptSetup => script::definition_in_script(ctx),
            BlockType::Style(_) => script::definition_in_style(ctx),
            BlockType::Art(ArtCursorPosition::VariantTemplate(_)) => {
                template::definition_in_template(ctx)
            }
            BlockType::Art(_) => None,
        }
    }

    /// Get definition with Corsa support (async version).
    #[cfg(feature = "native")]
    pub async fn definition_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        match ctx.block_type? {
            BlockType::Template => Self::definition_in_template_with_corsa(ctx, corsa_bridge).await,
            BlockType::Script | BlockType::ScriptSetup => {
                Self::definition_in_script_with_corsa(ctx, corsa_bridge).await
            }
            BlockType::Style(_) => script::definition_in_style(ctx),
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::definition_in_art_variant_with_corsa(ctx, info, corsa_bridge).await
            }
            BlockType::Art(_) => None,
        }
    }

    /// Find definition in art variant template with Corsa.
    #[cfg(feature = "native")]
    async fn definition_in_art_variant_with_corsa(
        ctx: &IdeContext<'_>,
        info: &crate::virtual_code::ArtVariantInfo,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        // Check if this is a component tag
        if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset) {
            if is_component_tag(&tag_name) {
                if let Some(def) = template::find_component_definition(ctx, &tag_name) {
                    return Some(def);
                }
            }
        }

        // Try Corsa definition lookup first.
        if let Some(bridge) = corsa_bridge {
            if let Some(ref virtual_docs) = ctx.virtual_docs {
                if let Some(ref tmpl) = virtual_docs.template {
                    let relative_offset = info.relative_offset as u32;
                    let vts_offset = tmpl
                        .source_map
                        .to_generated(relative_offset)
                        .map(|o| o as usize)
                        .unwrap_or(relative_offset as usize);

                    let (line, character) =
                        crate::ide::offset_to_position(&tmpl.content, vts_offset);

                    if bridge.is_initialized() {
                        let vdoc_uri = corsa_support::template_request_path(ctx.uri);
                        let Ok(uri) = bridge
                            .open_or_update_virtual_document(&vdoc_uri, &tmpl.content)
                            .await
                        else {
                            return template::definition_in_template(ctx);
                        };

                        if let Ok(locations) = bridge.definition(&uri, line, character).await {
                            if !locations.is_empty() {
                                return Self::convert_lsp_locations(locations, ctx);
                            }
                        }
                    }
                }
            }
        }

        // Fall back to synchronous definition
        template::definition_in_template(ctx)
    }

    /// Find definition in template with Corsa and component jump support.
    #[cfg(feature = "native")]
    async fn definition_in_template_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        // Check if this is a component tag
        if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset) {
            if is_component_tag(&tag_name) {
                if let Some(def) = template::find_component_definition(ctx, &tag_name) {
                    return Some(def);
                }
            }
        }

        // Check if this is a props property access
        if let Some(def) = template::find_props_property_definition(ctx, &word) {
            return Some(def);
        }

        // Check if this is a component attribute
        if let Some(def) = template::find_component_prop_definition(ctx) {
            return Some(def);
        }

        // Check if this is a prop name used directly in template
        if helpers::is_in_vue_directive_expression(ctx) {
            let options = vize_atelier_sfc::SfcParseOptions {
                filename: ctx.uri.path().to_string().into(),
                ..Default::default()
            };
            if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) {
                if let Some(def) = template::find_prop_definition_by_name(ctx, &descriptor, &word) {
                    return Some(def);
                }
            }
        }

        // Try Corsa definition lookup first.
        if let Some(bridge) = corsa_bridge {
            if let Some(ref virtual_docs) = ctx.virtual_docs {
                if let Some(ref tmpl) = virtual_docs.template {
                    if let Some(vts_offset) =
                        crate::ide::hover::HoverService::sfc_to_virtual_ts_offset(ctx, ctx.offset)
                    {
                        let (line, character) =
                            crate::ide::offset_to_position(&tmpl.content, vts_offset);

                        if bridge.is_initialized() {
                            let vdoc_uri = corsa_support::template_request_path(ctx.uri);
                            let Ok(uri) = bridge
                                .open_or_update_virtual_document(&vdoc_uri, &tmpl.content)
                                .await
                            else {
                                return template::definition_in_template(ctx);
                            };

                            if let Ok(locations) = bridge.definition(&uri, line, character).await {
                                if !locations.is_empty() {
                                    return Self::convert_lsp_locations(locations, ctx);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fall back to synchronous definition
        template::definition_in_template(ctx)
    }

    /// Find definition in script with Corsa support.
    #[cfg(feature = "native")]
    async fn definition_in_script_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        let is_setup = matches!(ctx.block_type, Some(BlockType::ScriptSetup));

        // Try Corsa definition lookup first.
        if let Some(bridge) = corsa_bridge {
            if let Some(ref virtual_docs) = ctx.virtual_docs {
                let script_doc = if is_setup {
                    virtual_docs.script_setup.as_ref()
                } else {
                    virtual_docs.script.as_ref()
                };

                if let Some(s) = script_doc {
                    if let Some(vts_offset) =
                        crate::ide::hover::HoverService::sfc_to_virtual_ts_script_offset(
                            ctx, ctx.offset,
                        )
                    {
                        let (line, character) =
                            crate::ide::offset_to_position(&s.content, vts_offset);

                        if bridge.is_initialized() {
                            let vdoc_uri = corsa_support::script_request_path(ctx.uri, is_setup);
                            let Ok(uri) = bridge
                                .open_or_update_virtual_document(&vdoc_uri, &s.content)
                                .await
                            else {
                                return script::definition_in_script(ctx);
                            };

                            if let Ok(locations) = bridge.definition(&uri, line, character).await {
                                if !locations.is_empty() {
                                    return Self::convert_lsp_locations(locations, ctx);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fall back to synchronous definition
        script::definition_in_script(ctx)
    }

    /// Convert a Corsa location to tower-lsp Location.
    #[cfg(feature = "native")]
    fn convert_lsp_locations(
        locations: Vec<vize_canon::LspLocation>,
        ctx: &IdeContext<'_>,
    ) -> Option<GotoDefinitionResponse> {
        if locations.len() == 1 {
            corsa_support::map_corsa_location(ctx, &locations[0])
                .map(GotoDefinitionResponse::Scalar)
        } else {
            let locs = corsa_support::map_corsa_locations(ctx, locations);
            if locs.is_empty() {
                None
            } else {
                Some(GotoDefinitionResponse::Array(locs))
            }
        }
    }
}
