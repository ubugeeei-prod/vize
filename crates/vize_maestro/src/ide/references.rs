//! References provider for Vue SFC files.
//!
//! Provides find-all-references for:
//! - Script bindings used in template
//! - Script bindings used in other script code
#![expect(
    clippy::disallowed_types,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]
//! - Script bindings used in style v-bind()

#[cfg(feature = "native")]
mod canonical;
#[cfg(all(test, feature = "native"))]
mod corsa_tests;
mod script;
pub(in crate::ide) mod structural;
mod template;

#[cfg(feature = "native")]
use std::sync::Arc;

use tower_lsp::lsp_types::Location;

#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

use super::IdeContext;
#[cfg(feature = "native")]
use crate::ide::corsa_support;
#[cfg(feature = "native")]
use crate::virtual_code::{ArtCursorPosition, BlockType};

/// References service for finding all references to a symbol.
pub struct ReferencesService;

impl ReferencesService {
    #[cfg(feature = "native")]
    fn get_word_at_offset(content: &str, offset: usize) -> Option<String> {
        crate::ide::token_at_offset(content, offset, |c| {
            c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
        })
    }

    /// Find all references to the symbol at the current position.
    pub fn references(ctx: &IdeContext, include_declaration: bool) -> Option<Vec<Location>> {
        if crate::ide::template_scope::needs_patterned_navigation(ctx) {
            return None;
        }
        structural::references(ctx, include_declaration)
    }

    /// Find all references using Corsa when available, with synchronous fallback.
    #[cfg(feature = "native")]
    pub async fn references_with_corsa(
        ctx: &IdeContext<'_>,
        include_declaration: bool,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Vec<Location>> {
        let canonical_locations =
            canonical::references(ctx, include_declaration, corsa_bridge.as_deref()).await;
        if crate::ide::template_scope::needs_patterned_navigation(ctx) {
            return canonical_locations;
        }
        if canonical_locations.is_some() {
            return canonical_locations;
        }
        let Some(block_type) = ctx.block_type else {
            return canonical_locations;
        };

        let corsa_locations = match block_type {
            BlockType::Template => {
                Self::template_references_with_corsa(
                    ctx,
                    include_declaration,
                    corsa_bridge.as_deref(),
                )
                .await
            }
            BlockType::Script | BlockType::ScriptSetup => {
                Self::script_references_with_corsa(
                    ctx,
                    include_declaration,
                    matches!(block_type, BlockType::ScriptSetup),
                    corsa_bridge.as_deref(),
                )
                .await
            }
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::art_variant_references_with_corsa(
                    ctx,
                    info,
                    include_declaration,
                    corsa_bridge.as_deref(),
                )
                .await
            }
            BlockType::Style(_) | BlockType::Art(_) => None,
        };

        // Corsa only answers for the virtual document the request opened, so
        // the authored hits carry the other blocks of this SFC.
        corsa_support::merge_canonical_locations(
            canonical_locations,
            corsa_locations,
            Self::references(ctx, include_declaration),
        )
        .map(|mut locations| {
            if !ctx.state.lsp_features().cross_file {
                locations.retain(|location| location.uri == *ctx.uri);
            }
            locations
        })
    }

    #[cfg(feature = "native")]
    async fn template_references_with_corsa(
        ctx: &IdeContext<'_>,
        include_declaration: bool,
        bridge: Option<&CorsaBridge>,
    ) -> Option<Vec<Location>> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let template = virtual_docs.template.as_ref()?;
        let vts_offset =
            crate::ide::hover::HoverService::sfc_to_virtual_ts_offset(ctx, ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);
        let request_path = corsa_support::template_request_path(ctx.uri);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;

        let locations = bridge
            .references(&uri, line, character, include_declaration)
            .await
            .ok()?;
        let locations = corsa_support::map_corsa_locations(ctx, locations);

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    }

    #[cfg(feature = "native")]
    async fn art_variant_references_with_corsa(
        ctx: &IdeContext<'_>,
        info: &crate::virtual_code::ArtVariantInfo,
        include_declaration: bool,
        bridge: Option<&CorsaBridge>,
    ) -> Option<Vec<Location>> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let template = virtual_docs.art_template(info.variant_index)?;
        let vts_offset = template.source_map.to_generated(ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&template.content, vts_offset);
        let request_path = corsa_support::art_template_request_path(ctx.uri, info.variant_index);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &template.content)
            .await
            .ok()?;

        let locations = bridge
            .references(&uri, line, character, include_declaration)
            .await
            .ok()?;
        let locations = corsa_support::map_corsa_locations(ctx, locations);

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    }

    #[cfg(feature = "native")]
    async fn script_references_with_corsa(
        ctx: &IdeContext<'_>,
        include_declaration: bool,
        is_setup: bool,
        bridge: Option<&CorsaBridge>,
    ) -> Option<Vec<Location>> {
        let bridge = bridge?;
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let script_doc = if is_setup {
            virtual_docs.script_setup.as_ref()
        } else {
            virtual_docs.script.as_ref()
        }?;
        let vts_offset =
            crate::ide::hover::HoverService::sfc_to_virtual_ts_script_offset(ctx, ctx.offset)?;
        let (line, character) = crate::ide::offset_to_position(&script_doc.content, vts_offset);
        let request_path = corsa_support::script_request_path(ctx.uri, is_setup);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &script_doc.content)
            .await
            .ok()?;

        let locations = bridge
            .references(&uri, line, character, include_declaration)
            .await
            .ok()?;
        let locations = corsa_support::map_corsa_locations(ctx, locations);

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    }
}
