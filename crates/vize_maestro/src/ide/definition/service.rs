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

mod import_target;

use super::{IdeContext, component_event, module_specifier, script};
#[cfg(feature = "native")]
use super::{helpers, template};
#[cfg(feature = "native")]
use crate::ide::corsa_support;
#[cfg(feature = "native")]
use crate::ide::is_component_tag;
use crate::virtual_code::{ArtCursorPosition, BlockType};

impl super::DefinitionService {
    /// Get definition for the symbol at the current position.
    pub fn definition(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
        match ctx.block_type? {
            BlockType::Template => import_target::component_tag_definition(ctx)
                .or_else(|| component_event::definition(ctx))
                .or_else(|| Self::definition_in_template_sync(ctx)),
            BlockType::Script | BlockType::ScriptSetup => {
                module_specifier::definition(ctx).or_else(|| script::definition_in_script(ctx))
            }
            BlockType::Style(_) => script::definition_in_style(ctx),
            BlockType::Art(ArtCursorPosition::VariantTemplate(_)) => {
                import_target::component_tag_definition(ctx)
                    .or_else(|| component_event::definition(ctx))
                    .or_else(|| Self::definition_in_template_sync(ctx))
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
        // Follow imported component aliases and re-export barrels before the
        // direct-file finder can stop at the barrel itself.
        if let Some(def) = import_target::component_tag_definition(ctx) {
            return Some(def);
        }

        if let Some(definition) = component_event::definition(ctx) {
            return Some(definition);
        }

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

        // Try Corsa definition lookup first. A variant expression without a generated
        // counterpart still has an authored answer, so a missing mapping falls through
        // to the synchronous lookup below instead of ending the request.
        if let Some(bridge) = corsa_bridge
            && let Some(ref virtual_docs) = ctx.virtual_docs
            && let Some(tmpl) = virtual_docs.art_template(info.variant_index)
            && let Some(vts_offset) = tmpl.source_map.to_generated(ctx.offset as u32)
        {
            let vts_offset = vts_offset as usize;

            let (line, character) = crate::ide::offset_to_position(&tmpl.content, vts_offset);

            if bridge.is_initialized() {
                let vdoc_uri =
                    corsa_support::art_template_request_path(ctx.uri, info.variant_index);
                let Ok(uri) = bridge
                    .open_or_update_virtual_document(&vdoc_uri, &tmpl.content)
                    .await
                else {
                    let definition = Self::definition_in_template_sync(ctx)?;
                    return import_target::normalize_bound_name_definition(ctx, definition);
                };

                if let Ok(locations) = bridge.definition(&uri, line, character).await
                    && !locations.is_empty()
                    && let Some(definition) = Self::convert_lsp_locations(locations, ctx)
                {
                    return import_target::normalize_bound_name_definition(ctx, definition);
                }
            }
        }

        // Fall back to synchronous definition, unwrapping only an actual
        // import-alias answer so template-local shadowing remains intact.
        let definition = Self::definition_in_template_sync(ctx)?;
        import_target::normalize_bound_name_definition(ctx, definition)
    }

    /// Find definition in template with Corsa and component jump support.
    #[cfg(feature = "native")]
    async fn definition_in_template_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        // A successful manual import walk is deterministic and follows barrel
        // re-exports to their source instead of returning the barrel module.
        if let Some(def) = import_target::component_tag_definition(ctx) {
            return Some(def);
        }

        if let Some(definition) = component_event::definition(ctx) {
            return Some(definition);
        }

        if let Some(tag_name) = helpers::get_tag_at_offset(&ctx.content, ctx.offset)
            && tag_name == "Self"
            && let Some(def) = template::find_component_definition(ctx, &tag_name)
        {
            return Some(def);
        }

        if let Some(definition) =
            Self::definition_for_html_attribute_with_corsa(ctx, corsa_bridge.as_ref()).await
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

        if let Some(definition) =
            Self::definition_via_canonical_corsa(ctx, corsa_bridge.as_ref()).await
        {
            return import_target::normalize_bound_name_definition(ctx, definition);
        }

        let word = helpers::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        if !crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset) {
            return Self::definition_in_template_sync(ctx);
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

        // Fall back to synchronous definition, unwrapping only an actual
        // import-alias answer so template-local shadowing remains intact.
        let definition = Self::definition_in_template_sync(ctx)?;
        import_target::normalize_bound_name_definition(ctx, definition)
    }

    /// Find definition in script with Corsa support.
    #[cfg(feature = "native")]
    async fn definition_in_script_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        if module_specifier::specifier_at_offset(&ctx.content, ctx.offset).is_some() {
            if let Some(bridge) = corsa_bridge.as_ref()
                && bridge.is_initialized()
                && let Some(definition) = module_specifier::definition_with_corsa(ctx, bridge).await
            {
                return Some(definition);
            }
            return module_specifier::definition(ctx);
        }

        // An imported name unwraps to the exported declaration before any
        // checker round-trip: the checker answers with the local alias — the
        // import statement itself — which reads as a self-jump (#3893).
        if let Some(definition) = import_target::definition(ctx) {
            return Some(definition);
        }

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
}
