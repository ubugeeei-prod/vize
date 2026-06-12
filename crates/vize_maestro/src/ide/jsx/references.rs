//! Find-all-references for `.jsx`/`.tsx` Vue components over the Corsa bridge
//! (#1498).
//!
//! The SFC [`ReferencesService`](crate::ide::ReferencesService) only fires when
//! the cursor lands in an SFC block; a `.jsx`/`.tsx` document has none, so this
//! is the JSX parallel. It reuses the exact same machinery as the JSX
//! hover/definition path: lower the document to plain virtual TS via
//! [`super::virtual_ts`], forward-map the cursor with [`JsxService`], call the
//! **same** `CorsaBridge::references` the SFC path uses, then map each returned
//! location back to the original `.jsx`/`.tsx` source (locations in real project
//! files pass through unchanged).
//!
//! Gated by the caller on `typeChecker.jsxTypecheck`, so React `.tsx` files are
//! never touched unless the user opts in.

use std::sync::Arc;

use tower_lsp::lsp_types::Location;
use vize_canon::CorsaBridge;

use super::service::JsxService;
use crate::ide::IdeContext;

/// Find-all-references service for `.jsx`/`.tsx` components.
pub struct JsxReferencesService;

impl JsxReferencesService {
    /// Find all references to the symbol at the cursor in a `.jsx`/`.tsx`
    /// document, resolved through the virtual TS and the type backend.
    pub async fn references(
        ctx: &IdeContext<'_>,
        include_declaration: bool,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Vec<Location>> {
        let bridge = corsa_bridge?;
        let (virtual_ts, uri, line, character) = JsxService::prepare_request(ctx, &bridge).await?;

        let locations = bridge
            .references(&uri, line, character, include_declaration)
            .await
            .ok()?;
        if locations.is_empty() {
            return None;
        }

        // Map each reference back: own-document locations become `.jsx`/`.tsx`
        // source ranges; locations in real files (a `.d.ts`, another module)
        // pass through. Reuses the definition path's `map_location` so JSX
        // references and definitions translate identically.
        let mapped: Vec<Location> = locations
            .iter()
            .filter_map(|location| JsxService::map_location(ctx, &virtual_ts, &uri, location))
            .collect();

        if mapped.is_empty() {
            None
        } else {
            Some(mapped)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Url;

    fn ctx_for<'a>(
        state: &'a ServerState,
        uri: &'a Url,
        source: &str,
        marker: &str,
    ) -> IdeContext<'a> {
        let offset = source.find(marker).expect("marker present") + marker.len();
        IdeContext::with_content(state, uri, offset, source.to_string())
    }

    // Without a Corsa bridge the type-aware references path degrades gracefully
    // (returns `None`) rather than panicking, exercising the full virtual-TS
    // generation + forward position mapping up to the bridge call.
    #[test]
    fn references_without_bridge_returns_none() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => {\n  const total = props.msg;\n  return <span>{total}</span>;\n};\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "total");
            assert!(
                JsxReferencesService::references(&ctx, true, None)
                    .await
                    .is_none()
            );
        });
    }
}
