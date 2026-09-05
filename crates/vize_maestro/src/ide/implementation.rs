//! Type-aware `textDocument/implementation` for authored Vue files.

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

#[cfg(feature = "native")]
use std::sync::Arc;

#[cfg(feature = "native")]
use tower_lsp::lsp_types::GotoDefinitionResponse;
#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

#[cfg(feature = "native")]
use super::{IdeContext, TypeDefinitionService, corsa_support};
#[cfg(feature = "native")]
use crate::virtual_code::{ArtCursorPosition, ArtVariantInfo, BlockType, VirtualDocument};

/// Checker-backed implementation navigation service.
pub struct ImplementationService;

#[cfg(feature = "native")]
impl ImplementationService {
    /// Resolve implementation locations through Corsa's standard LSP surface
    /// and map every returned location back onto authored source.
    pub async fn implementation_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        match ctx.block_type? {
            BlockType::Template | BlockType::Script | BlockType::ScriptSetup
                if !ctx.uri.path().ends_with(".art.vue") =>
            {
                Self::implementation_in_canonical_sfc(ctx, &bridge).await
            }
            BlockType::Script => Self::implementation_in_split_script(ctx, &bridge, false).await,
            BlockType::ScriptSetup => {
                Self::implementation_in_split_script(ctx, &bridge, true).await
            }
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::implementation_in_art_variant(ctx, &bridge, info).await
            }
            BlockType::Template | BlockType::Style(_) | BlockType::Art(_) => None,
        }
    }

    async fn implementation_in_canonical_sfc(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<GotoDefinitionResponse> {
        let document = corsa_support::open_canonical_virtual_project_document(ctx, bridge).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
        let locations = bridge
            .implementation(&document.request_uri, line, character)
            .await
            .ok()?;
        let locations = corsa_support::map_canonical_corsa_locations(ctx, &document, locations);
        TypeDefinitionService::convert_locations(locations)
    }

    async fn implementation_in_split_script(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        is_setup: bool,
    ) -> Option<GotoDefinitionResponse> {
        let virtual_docs = ctx.virtual_docs.as_ref()?;
        let document = if is_setup {
            virtual_docs.script_setup.as_ref()
        } else {
            virtual_docs.script.as_ref()
        }?;
        let generated_offset =
            super::hover::HoverService::sfc_to_virtual_ts_script_offset(ctx, ctx.offset)?;
        Self::implementation_in_virtual_document(
            ctx,
            bridge,
            document,
            generated_offset,
            corsa_support::script_request_path(ctx.uri, is_setup),
        )
        .await
    }

    async fn implementation_in_art_variant(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        info: &ArtVariantInfo,
    ) -> Option<GotoDefinitionResponse> {
        let document = ctx
            .virtual_docs
            .as_ref()?
            .art_template(info.variant_index)?;
        let generated_offset = document
            .source_map
            .to_generated_for(ctx.offset as u32, |features| features.definition)?
            as usize;
        Self::implementation_in_virtual_document(
            ctx,
            bridge,
            document,
            generated_offset,
            corsa_support::art_template_request_path(ctx.uri, info.variant_index),
        )
        .await
    }

    async fn implementation_in_virtual_document(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        document: &VirtualDocument,
        generated_offset: usize,
        request_path: vize_s0::String,
    ) -> Option<GotoDefinitionResponse> {
        let (line, character) = super::offset_to_position(&document.content, generated_offset);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &document.content)
            .await
            .ok()?;
        let locations = bridge.implementation(&uri, line, character).await.ok()?;
        let locations = corsa_support::map_corsa_locations(ctx, locations);
        TypeDefinitionService::convert_locations(locations)
    }
}
