use std::sync::Arc;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location};
use vize_canon::CorsaBridge;

use super::service::JsxService;
use crate::ide::{IdeContext, TypeDefinitionService};

/// Type-definition support for opt-in JSX/TSX virtual TypeScript.
pub struct JsxTypeDefinitionService;

impl JsxTypeDefinitionService {
    /// Go-to-type-definition on a `.jsx`/`.tsx` component, resolved through
    /// virtual TS and mapped back to authored source.
    pub async fn type_definition(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let bridge = corsa_bridge?;
        let (virtual_ts, request_uri, line, character) =
            JsxService::prepare_request(ctx, &bridge).await?;

        let locations = bridge
            .type_definition(&request_uri, line, character)
            .await
            .ok()?;
        if locations.is_empty() {
            return None;
        }

        let mapped: Vec<Location> = locations
            .iter()
            .filter_map(|location| {
                JsxService::map_location(ctx, &virtual_ts, &request_uri, location)
            })
            .collect();

        TypeDefinitionService::convert_locations(mapped)
    }
}
