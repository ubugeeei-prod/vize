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

use std::sync::Arc;

use tower_lsp::lsp_types::{
    CompletionResponse, Diagnostic, DiagnosticSeverity, GotoDefinitionResponse, Hover, Location,
    Position, Range, Url,
};
use vize_atelier_jsx::JsxLang;
use vize_canon::{CorsaBridge, LspLocation};
use vize_carton::cstr;

use super::position::{source_offset_to_virtual_position, virtual_range_to_source};
use super::virtual_ts::{JsxVirtualTs, generate_jsx_virtual_ts};
use crate::ide::IdeContext;
use crate::ide::completion::CompletionService;
use crate::ide::diagnostics::sources;
use crate::ide::hover::HoverService;

/// Type-aware JSX/TSX LSP service.
pub struct JsxService;

impl JsxService {
    /// Stable virtual-document name for a `.jsx`/`.tsx` URI's generated TS.
    ///
    /// Distinct from the SFC `.script.ts`/`.setup.ts`/`.template.ts` suffixes so
    /// a JSX document never collides with an SFC virtual doc in the Corsa
    /// session.
    fn request_path(uri: &Url) -> vize_carton::String {
        cstr!("{}.jsx.ts", uri.path())
    }

    /// Lower the current document to its plain virtual TypeScript.
    fn virtual_ts(ctx: &IdeContext<'_>) -> Option<JsxVirtualTs> {
        let lang = JsxLang::from_path(ctx.uri.path());
        generate_jsx_virtual_ts(&ctx.content, lang)
    }

    /// Hover on a `.jsx`/`.tsx` component, resolved through virtual TS.
    pub async fn hover(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }
        let virtual_ts = Self::virtual_ts(ctx)?;
        let (line, character) =
            source_offset_to_virtual_position(&virtual_ts.code, &virtual_ts.mappings, ctx.offset)?;

        let request_path = Self::request_path(ctx.uri);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &virtual_ts.code)
            .await
            .ok()?;

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
        if !bridge.is_initialized() {
            return None;
        }
        let virtual_ts = Self::virtual_ts(ctx)?;
        let (line, character) =
            source_offset_to_virtual_position(&virtual_ts.code, &virtual_ts.mappings, ctx.offset)?;

        let request_path = Self::request_path(ctx.uri);
        let uri = bridge
            .open_or_update_virtual_document(&request_path, &virtual_ts.code)
            .await
            .ok()?;

        let items = bridge.completion(&uri, line, character).await.ok()?;
        if items.is_empty() {
            return None;
        }
        let items = items
            .into_iter()
            .map(CompletionService::convert_lsp_completion)
            .collect();
        Some(CompletionResponse::Array(items))
    }

    /// Go-to-definition on a `.jsx`/`.tsx` component, resolved through virtual
    /// TS.
    pub async fn definition(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<GotoDefinitionResponse> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }
        let virtual_ts = Self::virtual_ts(ctx)?;
        let (line, character) =
            source_offset_to_virtual_position(&virtual_ts.code, &virtual_ts.mappings, ctx.offset)?;

        let request_path = Self::request_path(ctx.uri);
        let request_uri = bridge
            .open_or_update_virtual_document(&request_path, &virtual_ts.code)
            .await
            .ok()?;

        let locations = bridge
            .definition(&request_uri, line, character)
            .await
            .ok()?;
        if locations.is_empty() {
            return None;
        }

        let mapped: Vec<Location> = locations
            .iter()
            .filter_map(|location| Self::map_location(ctx, &virtual_ts, &request_uri, location))
            .collect();

        match mapped.len() {
            0 => None,
            1 => Some(GotoDefinitionResponse::Scalar(mapped.into_iter().next()?)),
            _ => Some(GotoDefinitionResponse::Array(mapped)),
        }
    }

    /// Type diagnostics for a `.jsx`/`.tsx` document, surfaced from its virtual
    /// TS through Corsa. Returned alongside the JSX compiler diagnostics.
    ///
    /// Each Corsa diagnostic is mapped from virtual-TS coordinates back to the
    /// source via the byte-range mappings; diagnostics that don't map to any
    /// user range (e.g. ones on the synthesized ambient preamble) are dropped.
    pub async fn diagnostics(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Vec<Diagnostic> {
        let Some(bridge) = corsa_bridge else {
            return vec![];
        };
        if !bridge.is_initialized() {
            return vec![];
        }
        let Some(virtual_ts) = Self::virtual_ts(ctx) else {
            return vec![];
        };

        let request_path = Self::request_path(ctx.uri);
        let Ok(uri) = bridge
            .open_or_update_virtual_document(&request_path, &virtual_ts.code)
            .await
        else {
            return vec![];
        };

        let Ok(corsa_diags) = bridge.get_diagnostics(&uri).await else {
            return vec![];
        };

        corsa_diags
            .into_iter()
            .filter_map(|diag| {
                // Skip "declared but never used" noise on the synthesized sink
                // helper and any other internal `__vize_` symbol.
                let is_unused = diag.message.contains("is declared but")
                    && (diag.message.contains("never read") || diag.message.contains("never used"));
                if is_unused && diag.message.contains("'__vize") {
                    return None;
                }

                let (start_line, end_line, start_char, end_char) = virtual_range_to_source(
                    &virtual_ts.code,
                    &ctx.content,
                    &virtual_ts.mappings,
                    diag.range.start.line,
                    diag.range.start.character,
                    diag.range.end.line,
                    diag.range.end.character,
                )?;

                Some(Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_char,
                        },
                        end: Position {
                            line: end_line,
                            character: end_char,
                        },
                    },
                    severity: diag.severity.map(|s| match s {
                        1 => DiagnosticSeverity::ERROR,
                        2 => DiagnosticSeverity::WARNING,
                        3 => DiagnosticSeverity::INFORMATION,
                        _ => DiagnosticSeverity::HINT,
                    }),
                    source: Some(sources::TYPE_CHECKER.to_string()),
                    message: diag.message,
                    ..Default::default()
                })
            })
            .collect()
    }

    /// Map a Corsa definition location back onto the source.
    ///
    /// Locations pointing at this document's own virtual TS are remapped to the
    /// original `.jsx`/`.tsx` source range; locations in real project files
    /// (e.g. a `node_modules` `.d.ts`) pass through unchanged.
    fn map_location(
        ctx: &IdeContext<'_>,
        virtual_ts: &JsxVirtualTs,
        request_uri: &str,
        location: &LspLocation,
    ) -> Option<Location> {
        if Self::same_uri(&location.uri, request_uri) {
            let range = Self::map_virtual_range(
                virtual_ts,
                &ctx.content,
                Range {
                    start: Position {
                        line: location.range.start.line,
                        character: location.range.start.character,
                    },
                    end: Position {
                        line: location.range.end.line,
                        character: location.range.end.character,
                    },
                },
            )?;
            return Some(Location {
                uri: ctx.uri.clone(),
                range,
            });
        }

        let uri = Url::parse(&location.uri).ok()?;
        Some(Location {
            uri,
            range: Range {
                start: Position {
                    line: location.range.start.line,
                    character: location.range.start.character,
                },
                end: Position {
                    line: location.range.end.line,
                    character: location.range.end.character,
                },
            },
        })
    }

    /// Map an LSP range in virtual-TS coordinates back to the source document.
    fn map_virtual_range(virtual_ts: &JsxVirtualTs, source: &str, range: Range) -> Option<Range> {
        let (start_line, end_line, start_char, end_char) = virtual_range_to_source(
            &virtual_ts.code,
            source,
            &virtual_ts.mappings,
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

    /// Compare a Corsa-returned URI against the virtual-document URI we opened.
    /// Corsa may echo the URI in `file://` form while the request path is a
    /// bare filesystem path, so compare on the path component.
    fn same_uri(candidate: &str, request_uri: &str) -> bool {
        if candidate == request_uri {
            return true;
        }
        Self::uri_path(candidate) == Self::uri_path(request_uri)
    }

    fn uri_path(uri: &str) -> &str {
        uri.strip_prefix("file://").unwrap_or(uri)
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
        IdeContext::with_content(state, uri, offset, source.to_string())
    }

    #[test]
    fn request_path_is_distinct_from_sfc_virtual_docs() {
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        let path = JsxService::request_path(&uri);
        assert_eq!(path.as_str(), "/tmp/Comp.tsx.jsx.ts");
    }

    #[test]
    fn same_uri_matches_across_file_scheme() {
        assert!(JsxService::same_uri(
            "file:///tmp/Comp.tsx.jsx.ts",
            "/tmp/Comp.tsx.jsx.ts"
        ));
        assert!(!JsxService::same_uri(
            "file:///tmp/Other.ts",
            "/tmp/Comp.tsx.jsx.ts"
        ));
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
