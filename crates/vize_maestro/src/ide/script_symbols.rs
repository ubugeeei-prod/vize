//! Read-only native symbol queries for authored non-JSX scripts.
//!
//! Plain scripts have no SFC block. Their existing canonical script project
//! supplies the source/configuration/overlay identities without an SFC adapter.
#![expect(
    clippy::disallowed_types,
    reason = "the existing Corsa bridge is shared through std Arc"
)]

use std::sync::Arc;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location};
use vize_canon::{CorsaBridge, LspLocation};

use super::IdeContext;
use super::corsa_support::{
    CanonicalVirtualDocument, canonical_source_offset_to_position, map_canonical_corsa_location,
    open_canonical_script_document,
};

#[cfg(test)]
mod tests;

pub(crate) struct ScriptSymbolsService;

impl ScriptSymbolsService {
    pub(crate) async fn definition(
        ctx: &IdeContext<'_>,
        bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let bridge = bridge?;
        if !bridge.is_initialized() {
            return None;
        }
        let document = open_canonical_script_document(ctx, &bridge, false).await?;
        let (line, character) = canonical_source_offset_to_position(&document, ctx.offset)?;
        let locations = bridge
            .definition(&document.request_uri, line, character)
            .await
            .ok()?;
        let mapped = map_complete_locations(ctx, &document, &locations)?;
        match mapped.len() {
            0 => None,
            1 => Some(GotoDefinitionResponse::Scalar(mapped.into_iter().next()?)),
            _ => Some(GotoDefinitionResponse::Array(mapped)),
        }
    }

    pub(crate) async fn references(
        ctx: &IdeContext<'_>,
        include_declaration: bool,
        bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Vec<Location>> {
        let bridge = bridge?;
        if !bridge.is_initialized() {
            return None;
        }
        // Workspace synchronization retains incoming Vue importers as well as
        // the configured script project and its dirty editor overlays.
        let document = open_canonical_script_document(ctx, &bridge, true).await?;
        let (line, character) = canonical_source_offset_to_position(&document, ctx.offset)?;
        let locations = bridge
            .references(&document.request_uri, line, character, include_declaration)
            .await
            .ok()?;
        let mapped = map_complete_locations(ctx, &document, &locations)?;
        (!mapped.is_empty()).then_some(mapped)
    }
}

fn map_complete_locations(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    locations: &[LspLocation],
) -> Option<Vec<Location>> {
    // A private/unmappable member invalidates the whole native answer. Dropping
    // it would present an incomplete reference set as a successful result.
    locations
        .iter()
        .map(|location| map_canonical_corsa_location(ctx, document, location))
        .collect()
}
