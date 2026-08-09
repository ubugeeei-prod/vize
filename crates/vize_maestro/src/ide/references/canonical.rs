use tower_lsp::lsp_types::Location;
use vize_canon::CorsaBridge;

use crate::ide::{IdeContext, corsa_support};

/// Query the project-aware canonical Vue document before the block-local
/// virtual documents so references can cross SFC boundaries.
pub(super) async fn references(
    ctx: &IdeContext<'_>,
    include_declaration: bool,
    bridge: Option<&CorsaBridge>,
) -> Option<Vec<Location>> {
    let bridge = bridge?;
    if !bridge.is_initialized() {
        return None;
    }
    let document = corsa_support::open_canonical_virtual_project_document(ctx, bridge).await?;
    let (line, character) =
        corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
    let locations = bridge
        .references(&document.request_uri, line, character, include_declaration)
        .await
        .ok()?;
    let locations = corsa_support::map_canonical_corsa_locations(ctx, &document, locations);
    Some(locations)
}
