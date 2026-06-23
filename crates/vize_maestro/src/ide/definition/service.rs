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

use super::{IdeContext, helpers, script, template};
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
        // Check if this is a component tag
        if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset)
            && is_component_tag(&tag_name)
            && let Some(def) = template::find_component_definition(ctx, &tag_name)
        {
            return Some(def);
        }

        if let Some(def) = template::find_component_prop_definition(ctx) {
            return Some(def);
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
            return None;
        }

        // Try Corsa definition lookup first.
        if let Some(bridge) = corsa_bridge
            && let Some(ref virtual_docs) = ctx.virtual_docs
            && let Some(tmpl) = virtual_docs.art_template(info.variant_index)
        {
            let relative_offset = info.relative_offset as u32;
            let vts_offset = tmpl
                .source_map
                .to_generated(relative_offset)
                .map(|o| o as usize)
                .unwrap_or(relative_offset as usize);

            let (line, character) = crate::ide::offset_to_position(&tmpl.content, vts_offset);

            if bridge.is_initialized() {
                let vdoc_uri =
                    corsa_support::art_template_request_path(ctx.uri, info.variant_index);
                let Ok(uri) = bridge
                    .open_or_update_virtual_document(&vdoc_uri, &tmpl.content)
                    .await
                else {
                    return template::definition_in_template(ctx);
                };

                if let Ok(locations) = bridge.definition(&uri, line, character).await
                    && !locations.is_empty()
                {
                    return Self::convert_lsp_locations(locations, ctx);
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
        if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset)
            && tag_name == "Self"
            && let Some(def) = template::find_component_definition(ctx, &tag_name)
        {
            return Some(def);
        }

        if let Some(definition) =
            Self::definition_via_canonical_corsa(ctx, corsa_bridge.as_ref()).await
        {
            return Some(definition);
        }

        if let Some(definition) =
            Self::definition_for_html_tag_with_corsa(ctx, corsa_bridge.as_ref()).await
        {
            return Some(definition);
        }

        if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset)
            && is_component_tag(&tag_name)
            && let Some(def) = template::find_component_definition(ctx, &tag_name)
        {
            return Some(def);
        }

        if let Some(def) = template::find_component_prop_definition(ctx) {
            return Some(def);
        }

        let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
            return None;
        }

        // Check if this is a props property access
        if let Some(def) = template::find_props_property_definition(ctx, &word) {
            return Some(def);
        }

        // Check if this is a prop name used directly in template
        if helpers::is_in_vue_directive_expression(ctx) {
            let options = vize_atelier_sfc::SfcParseOptions {
                filename: ctx.uri.path().to_string().into(),
                ..Default::default()
            };
            if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options)
                && let Some(def) = template::find_prop_definition_by_name(ctx, &descriptor, &word)
            {
                return Some(def);
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
        if let Some(definition) = script::definition_in_script(ctx) {
            let is_define_art_source =
                crate::ide::musea::define_art_source_at_offset(&ctx.content, ctx.uri, ctx.offset)
                    .is_some();
            if is_define_art_source {
                return Some(definition);
            }
        }

        let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        // Try Corsa definition lookup first via the canonical Vue virtual TS.
        if let Some(bridge) = corsa_bridge.as_ref()
            && bridge.is_initialized()
            && let Some(doc) = corsa_support::open_canonical_virtual_document(ctx, bridge).await
            && let Some((line, character)) =
                corsa_support::canonical_source_offset_to_position(&doc, ctx.offset)
            && let Ok(locations) = bridge.definition(&doc.request_uri, line, character).await
            && !locations.is_empty()
        {
            let locations = corsa_support::map_canonical_corsa_locations(ctx, &doc, locations);
            if let Some(response) = Self::convert_locations(locations) {
                return Some(response);
            }
        }

        // Fall back to synchronous definition
        script::definition_in_script(ctx)
    }

    #[cfg(feature = "native")]
    async fn definition_via_canonical_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc = corsa_support::open_canonical_virtual_document(ctx, bridge).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&doc, ctx.offset)?;
        let locations = bridge
            .definition(&doc.request_uri, line, character)
            .await
            .ok()?;
        if locations.is_empty() {
            return None;
        }

        let locations = corsa_support::map_canonical_corsa_locations(ctx, &doc, locations);
        Self::convert_locations(locations)
    }

    /// Convert a Corsa location to tower-lsp Location.
    #[cfg(feature = "native")]
    fn convert_lsp_locations(
        locations: Vec<vize_canon::LspLocation>,
        ctx: &IdeContext<'_>,
    ) -> Option<GotoDefinitionResponse> {
        let locations = corsa_support::map_corsa_locations(ctx, locations);
        Self::convert_locations(locations)
    }

    #[cfg(feature = "native")]
    fn convert_locations(
        locations: Vec<tower_lsp::lsp_types::Location>,
    ) -> Option<GotoDefinitionResponse> {
        match locations.as_slice() {
            [] => None,
            [location] => Some(GotoDefinitionResponse::Scalar(location.clone())),
            _ => Some(GotoDefinitionResponse::Array(locations)),
        }
    }

    #[cfg(feature = "native")]
    async fn definition_for_html_tag_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<&Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let tag_name = helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if is_component_tag(&tag_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let doc = corsa_support::html_tag_virtual_document(&tag_name)?;
        let request_path = corsa_support::html_tag_request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &doc.content)
            .await
            .ok()?;
        let (line, character) = crate::ide::offset_to_position(&doc.content, doc.definition_offset);
        let locations = bridge
            .definition(&request_uri, line, character)
            .await
            .ok()?;

        Self::convert_lsp_locations(locations, ctx)
    }
}
