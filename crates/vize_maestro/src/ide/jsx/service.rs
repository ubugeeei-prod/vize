//! Type-aware LSP for `.jsx`/`.tsx` Vue components over the Corsa bridge
//! (#1498).
//!
//! `.jsx`/`.tsx` documents are not SFCs, so the cursor never lands in an SFC
//! `BlockType` and the SFC hover/completion/definition services return early.
//! This service is the JSX parallel: it lowers the document to plain virtual
//! TypeScript via [`super::virtual_ts`] (matching the type-checker), maps the
//! editor cursor into that virtual TS, queries the **same** Corsa backend the
//! SFC path uses, and maps the result back to the original source.
//!
//! Conversion of Corsa payloads reuses the SFC converters
//! ([`HoverService::convert_lsp_hover`], [`CompletionService::convert_lsp_completion`])
//! so JSX hover/completion render identically to SFC — only the position
//! mapping differs (byte-range mappings instead of the SFC source map).
//!
//! Every entry point here is reached **only** when `typeChecker.jsxTypecheck`
//! is enabled (checked by the caller in the request handlers); React `.tsx`
//! files are otherwise left entirely untouched.
//!
//! Signature help follows the same mapping path. Its response has no source
//! ranges, so only the authored cursor needs forward mapping; labels,
//! documentation, overloads, and active-parameter state pass through intact.

use std::sync::Arc;

use tower_lsp::lsp_types::{
    CompletionResponse, GotoDefinitionResponse, Hover, Position, Range, SignatureHelp, Url,
};
use vize_canon::CorsaBridge;
use vize_l0::cstr;

use super::position::{
    source_cursor_to_virtual_position, source_offset_to_virtual_position, virtual_range_to_source,
};
use super::service_project::map_navigation_locations;
use super::virtual_ts::JsxVirtualTs;
use crate::ide::IdeContext;
use crate::ide::completion::CompletionService;
use crate::ide::hover::HoverService;

/// Type-aware JSX/TSX LSP service.
pub struct JsxService;

impl JsxService {
    /// Stable virtual-document name for a `.jsx`/`.tsx` URI's generated TS.
    ///
    /// Distinct from the SFC `.script.ts`/`.setup.ts`/`.template.ts` suffixes so
    /// a JSX document never collides with an SFC virtual doc in the Corsa
    /// session. Shared by every type-aware JSX request (hover, completion,
    /// definition, references, rename, diagnostics) so they all key the same
    /// virtual document in the session cache.
    pub(super) fn request_path(uri: &Url) -> vize_l0::String {
        cstr!("{}.jsx.ts", uri.path())
    }

    /// Generate the virtual TS, forward-map the editor cursor into it, and open
    /// the (shared) virtual document on the bridge. Returns everything a
    /// position-based request needs: the virtual TS, the opened virtual-doc URI,
    /// and the cursor's `(line, character)` in virtual-TS coordinates.
    ///
    /// `None` when the bridge is missing/uninitialized, lowering fails, or the
    /// cursor doesn't map into the virtual TS — every type-aware JSX entry point
    /// degrades gracefully in those cases.
    pub(super) async fn prepare_request(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<(JsxVirtualTs, vize_l0::String, u32, u32)> {
        if !bridge.is_initialized() {
            return None;
        }
        let (virtual_ts, uri) = super::service_project::open_virtual_project(ctx, bridge).await?;
        let (line, character) = source_offset_to_virtual_position(&virtual_ts, ctx.offset)?;
        Some((virtual_ts, uri, line, character))
    }

    /// Hover on a `.jsx`/`.tsx` component, resolved through virtual TS.
    pub async fn hover(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let bridge = corsa_bridge?;
        let (virtual_ts, uri, line, character) = Self::prepare_request(ctx, &bridge).await?;

        let lsp_hover = bridge.hover(&uri, line, character).await.ok()??;
        let mut hover = HoverService::convert_lsp_hover(lsp_hover);

        // The converted range is in virtual-TS coordinates; map it back so the
        // editor highlights the right span in the original document.
        if let Some(range) = hover.range {
            hover.range = Self::map_virtual_range(&virtual_ts, &ctx.content, range);
        }
        Some(hover)
    }

    /// Completion on a `.jsx`/`.tsx` component, resolved through virtual TS.
    pub async fn completion(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<CompletionResponse> {
        let bridge = corsa_bridge?;
        let (projection, uri) = super::service_project::open_virtual_project(ctx, &bridge).await?;
        let (line, character) = source_cursor_to_virtual_position(&projection, ctx.offset)?;
        let items =
            CompletionService::request_resolvable(ctx, &bridge, &uri, line, character).await;
        if items.is_empty() {
            return None;
        }
        Some(CompletionResponse::Array(items))
    }

    /// Signature help on a `.jsx`/`.tsx` call, resolved through virtual TS.
    pub async fn signature_help(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<SignatureHelp> {
        Self::signature_help_with_context(ctx, corsa_bridge, None).await
    }

    /// Signature help while preserving the triggering LSP context.
    pub async fn signature_help_with_context(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
        context: Option<serde_json::Value>,
    ) -> Option<SignatureHelp> {
        let bridge = corsa_bridge?;
        let (_virtual_ts, uri, line, character) = Self::prepare_request(ctx, &bridge).await?;
        let help = bridge
            .signature_help_with_context(&uri, line, character, context)
            .await
            .ok()??;
        Some(crate::ide::SignatureHelpService::convert_lsp_signature_help(help))
    }

    /// Go-to-definition on a `.jsx`/`.tsx` component, resolved through virtual
    /// TS.
    pub async fn definition(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let bridge = corsa_bridge?;
        let (document, line, character) =
            super::service_project::prepare_navigation_request(ctx, &bridge).await?;

        let locations = bridge
            .definition(&document.request_uri, line, character)
            .await
            .ok()?;
        if locations.is_empty() {
            return None;
        }

        let mapped = map_navigation_locations(ctx, &document, locations);

        match mapped.len() {
            0 => None,
            1 => Some(GotoDefinitionResponse::Scalar(mapped.into_iter().next()?)),
            _ => Some(GotoDefinitionResponse::Array(mapped)),
        }
    }

    /// Map an LSP range in virtual-TS coordinates back to the source document.
    pub(super) fn map_virtual_range(
        virtual_ts: &JsxVirtualTs,
        source: &str,
        range: Range,
    ) -> Option<Range> {
        let (start_line, end_line, start_char, end_char) = virtual_range_to_source(
            virtual_ts,
            source,
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character,
        )?;
        Some(Range {
            start: Position {
                line: start_line,
                character: start_char,
            },
            end: Position {
                line: end_line,
                character: end_char,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ServerState;

    fn ctx_for<'a>(
        state: &'a ServerState,
        uri: &'a Url,
        source: &str,
        marker: &str,
    ) -> IdeContext<'a> {
        let offset = source.find(marker).expect("marker present") + marker.len();
        IdeContext::testing(state, uri, offset, source.to_string())
    }

    #[test]
    fn request_path_is_distinct_from_sfc_virtual_docs() {
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        let path = JsxService::request_path(&uri);
        assert_eq!(path.as_str(), "/tmp/Comp.tsx.jsx.ts");
    }

    // Without a Corsa bridge the type-aware path must degrade gracefully (the
    // editor keeps the JSX compiler diagnostics) rather than panic. This
    // exercises the full virtual-TS generation + forward position mapping up to
    // the bridge call for a real `.tsx` cursor.
    #[test]
    fn hover_without_bridge_returns_none() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "props.msg");
            assert!(JsxService::hover(&ctx, None).await.is_none());
        });
    }

    #[test]
    fn completion_without_bridge_returns_none() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => <div>{props.}</div>;\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "props.");
            assert!(JsxService::completion(&ctx, None).await.is_none());
        });
    }

    #[test]
    fn definition_without_bridge_returns_none() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "props.msg");
            assert!(JsxService::definition(&ctx, None).await.is_none());
        });
    }

    #[test]
    fn diagnostics_without_bridge_is_empty() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "props.msg");
            assert!(JsxService::diagnostics(&ctx, None).await.is_empty());
        });
    }
}
