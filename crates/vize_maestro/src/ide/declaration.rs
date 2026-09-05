//! Type-aware `textDocument/declaration` for authored Vue files.

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

/// Checker-backed declaration navigation service.
pub struct DeclarationService;

#[cfg(feature = "native")]
impl DeclarationService {
    /// Resolve declaration locations through Corsa's standard LSP surface and
    /// map every returned location back onto authored source.
    pub async fn declaration_with_corsa(
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
                Self::declaration_in_canonical_sfc(ctx, &bridge).await
            }
            BlockType::Script => Self::declaration_in_split_script(ctx, &bridge, false).await,
            BlockType::ScriptSetup => Self::declaration_in_split_script(ctx, &bridge, true).await,
            BlockType::Art(ArtCursorPosition::VariantTemplate(ref info)) => {
                Self::declaration_in_art_variant(ctx, &bridge, info).await
            }
            BlockType::Template | BlockType::Style(_) | BlockType::Art(_) => None,
        }
    }

    async fn declaration_in_canonical_sfc(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<GotoDefinitionResponse> {
        let document = corsa_support::open_canonical_virtual_project_document(ctx, bridge).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
        let locations = Self::declaration_or_definition_locations(
            bridge,
            &document.request_uri,
            line,
            character,
        )
        .await?;
        let locations = corsa_support::map_canonical_corsa_locations(ctx, &document, locations);
        TypeDefinitionService::convert_locations(locations)
    }

    async fn declaration_in_split_script(
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
        Self::declaration_in_virtual_document(
            ctx,
            bridge,
            document,
            generated_offset,
            corsa_support::script_request_path(ctx.uri, is_setup),
        )
        .await
    }

    async fn declaration_in_art_variant(
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
        Self::declaration_in_virtual_document(
            ctx,
            bridge,
            document,
            generated_offset,
            corsa_support::art_template_request_path(ctx.uri, info.variant_index),
        )
        .await
    }

    async fn declaration_in_virtual_document(
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
        let locations =
            Self::declaration_or_definition_locations(bridge, &uri, line, character).await?;
        let locations = corsa_support::map_corsa_locations(ctx, locations);
        TypeDefinitionService::convert_locations(locations)
    }

    async fn declaration_or_definition_locations(
        bridge: &CorsaBridge,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Option<Vec<vize_canon::LspLocation>> {
        match bridge.declaration(uri, line, character).await {
            Ok(locations) if !locations.is_empty() => Some(locations),
            Ok(_) | Err(_) => bridge.definition(uri, line, character).await.ok(),
        }
    }
}

#[cfg(all(test, feature = "native"))]
mod tests;
