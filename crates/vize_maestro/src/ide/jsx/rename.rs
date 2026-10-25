//! Rename for `.jsx`/`.tsx` Vue components over the Corsa bridge (#1498).
//!
//! The JSX parallel to the SFC [`RenameService`](crate::ide::RenameService):
//! lower the document to plain virtual TS, forward-map the cursor, call the
//! **same** `CorsaBridge::prepare_rename` / `CorsaBridge::rename` the SFC path
//! uses, then map every returned edit range back to the original `.jsx`/`.tsx`
//! source. Canonical project identities map edits in imported materialized
//! modules back to authored files and reject unmapped private session paths.
//!
//! The rename **guard** mirrors SFC: `rename` validates the new name is a legal
//! identifier before touching the backend (the type backend's own
//! prepare-rename gates *where* a rename may start). Gated by the caller on
//! `typeChecker.jsxTypecheck`.
#![expect(
    clippy::disallowed_types,
    reason = "lsp_types::WorkspaceEdit uses a std HashMap and the Corsa bridge is shared through std Arc"
)]

use std::sync::Arc;

use tower_lsp::lsp_types::{PrepareRenameResponse, WorkspaceEdit};
use vize_canon::CorsaBridge;

use super::service::JsxService;
use super::service_project::prepare_navigation_request;
use super::virtual_ts::JsxVirtualTs;
use crate::ide::IdeContext;
use crate::ide::corsa_support::map_canonical_corsa_workspace_edit;

/// Rename service for `.jsx`/`.tsx` components.
pub struct JsxRenameService;

impl JsxRenameService {
    /// Check whether a rename may start at the cursor, in `.jsx`/`.tsx`
    /// coordinates. Delegates the decision to the type backend (so it only
    /// allows renaming real symbols) and maps the returned range back to source.
    pub async fn prepare_rename(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<PrepareRenameResponse> {
        let bridge = corsa_bridge?;
        let (virtual_ts, uri, line, character) = JsxService::prepare_request(ctx, &bridge).await?;
        let response = bridge.prepare_rename(&uri, line, character).await.ok()??;
        let response: PrepareRenameResponse = serde_json::from_value(response).ok()?;
        Self::map_prepare_rename(ctx, &virtual_ts, response)
    }

    /// Rename the symbol at the cursor across the project, mapping the edits in
    /// this document's virtual TS back onto the original `.jsx`/`.tsx` source.
    pub async fn rename(
        ctx: &IdeContext<'_>,
        new_name: &str,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<WorkspaceEdit> {
        // Guard: reject illegal identifiers before touching the backend, as the
        // SFC rename service does.
        if !is_valid_identifier(new_name) {
            return None;
        }

        let bridge = corsa_bridge?;
        let (document, line, character) = prepare_navigation_request(ctx, &bridge).await?;

        let edit = bridge
            .rename(&document.request_uri, line, character, new_name)
            .await
            .ok()??;
        let edit: WorkspaceEdit = serde_json::from_value(edit).ok()?;
        map_canonical_corsa_workspace_edit(ctx, &document, edit)
    }

    /// Present the authored target, excluding generated quotes or canonical spelling.
    fn map_prepare_rename(
        ctx: &IdeContext<'_>,
        virtual_ts: &JsxVirtualTs,
        response: PrepareRenameResponse,
    ) -> Option<PrepareRenameResponse> {
        match response {
            PrepareRenameResponse::Range(range) => {
                JsxService::map_virtual_range(virtual_ts, &ctx.content, range)
                    .map(PrepareRenameResponse::Range)
            }
            PrepareRenameResponse::RangeWithPlaceholder { range, .. } => {
                let range = JsxService::map_virtual_range(virtual_ts, &ctx.content, range)?;
                let start = crate::ide::position_to_offset(
                    &ctx.content,
                    range.start.line,
                    range.start.character,
                )?;
                let end = crate::ide::position_to_offset(
                    &ctx.content,
                    range.end.line,
                    range.end.character,
                )?;
                let placeholder = ctx.content.get(start..end)?.to_owned();
                Some(PrepareRenameResponse::RangeWithPlaceholder { range, placeholder })
            }
            PrepareRenameResponse::DefaultBehavior { default_behavior } => {
                Some(PrepareRenameResponse::DefaultBehavior { default_behavior })
            }
        }
    }
}

/// Whether `s` is a legal JS/TS identifier (rename target guard).
fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
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
        IdeContext::testing(state, uri, offset, source.to_string())
    }

    #[test]
    fn rejects_invalid_new_name() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123abc"));
        assert!(!is_valid_identifier("has space"));
        assert!(is_valid_identifier("renamed"));
        assert!(is_valid_identifier("_x"));
        assert!(is_valid_identifier("$ref"));
    }

    #[test]
    fn rename_with_invalid_identifier_returns_none() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "props.msg");
            // Even with a (would-be) bridge, an illegal name short-circuits.
            assert!(
                JsxRenameService::rename(&ctx, "not valid", None)
                    .await
                    .is_none()
            );
        });
    }

    #[test]
    fn prepare_rename_without_bridge_returns_none() {
        crate::runtime::block_on(async {
            let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
            let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
            let state = ServerState::new();
            let ctx = ctx_for(&state, &uri, source, "props.msg");
            assert!(JsxRenameService::prepare_rename(&ctx, None).await.is_none());
        });
    }

    #[test]
    fn mapped_property_prepare_rename_uses_whole_authored_token_and_placeholder() {
        for newline in ["\n", "\r\n"] {
            for (authored, generated_name) in
                [("count", "count"), ("foo-bar", "fooBar"), ("名称", "名称")]
            {
                let source =
                    vize_l0::cstr!("// 🧭\nconst view = <Child {authored}={{value}} />;\n")
                        .replace('\n', newline);
                let generated = super::super::virtual_ts::generate_jsx_virtual_ts(
                    &source,
                    vize_atelier_jsx::JsxLang::Tsx,
                )
                .unwrap();
                let quoted = vize_l0::cstr!("\"{generated_name}\"");
                let key = generated.code.find(quoted.as_str()).unwrap();
                let state = ServerState::new();
                let uri = Url::parse("file:///tmp/Props.tsx").unwrap();
                let at = source.find(authored).unwrap();
                let ctx = IdeContext::testing(&state, &uri, at, source.clone());
                let range = |text: &str, start: usize, end: usize| {
                    let (line, character) = crate::ide::offset_to_position(text, start);
                    let from = tower_lsp::lsp_types::Position::new(line, character);
                    let (line, character) = crate::ide::offset_to_position(text, end);
                    tower_lsp::lsp_types::Range::new(
                        from,
                        tower_lsp::lsp_types::Position::new(line, character),
                    )
                };
                let native = PrepareRenameResponse::RangeWithPlaceholder {
                    range: range(&generated.code, key + 1, key + quoted.len() - 1),
                    placeholder: quoted.into(),
                };
                assert_eq!(
                    JsxRenameService::map_prepare_rename(&ctx, &generated, native),
                    Some(PrepareRenameResponse::RangeWithPlaceholder {
                        range: range(&source, at, at + authored.len()),
                        placeholder: authored.into()
                    })
                );
            }
        }
    }
}
