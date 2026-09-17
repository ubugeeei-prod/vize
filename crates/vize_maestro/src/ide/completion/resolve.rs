//! Lazy checker documentation without exposing backend edits to the editor.
#![allow(clippy::disallowed_types)]

use serde_json::{Value, json};
use tower_lsp::lsp_types::{CompletionItem, Url};
use vize_canon::{CorsaBridge, LspCompletionItem};

use super::CompletionService;
use crate::{
    ide::{IdeContext, corsa_support},
    server::ServerState,
};

const RESOLVE_DATA: &str = "vizeCompletion";

#[cfg(test)]
mod tests;
mod visibility;

impl CompletionService {
    pub(super) async fn request_canonical(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<Vec<CompletionItem>> {
        // Sharing the diagnostic document avoids duplicate global declarations
        // and stale JSDoc from a second per-block TypeScript projection.
        let document = corsa_support::open_canonical_virtual_document(ctx, bridge).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
        let mut items =
            Self::request_resolvable(ctx, bridge, &document.request_uri, line, character).await;
        visibility::retain_authored_bindings(ctx, &document, line, character, &mut items);
        Some(items)
    }

    pub(super) async fn request_resolvable(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Vec<CompletionItem> {
        let revision = ctx.state.documents.get(ctx.uri).and_then(|document| {
            (document.content == ctx.content.as_str()).then(|| document.revision())
        });
        let Some(revision) = revision else {
            return vec![];
        };
        let Ok(Some(response)) = bridge.completion_raw(uri, line, character).await else {
            return vec![];
        };
        if !is_current(ctx.state, ctx.uri, revision) {
            return vec![];
        }
        let items = match response {
            Value::Array(items) => items,
            Value::Object(mut object) => match object.remove("items") {
                Some(Value::Array(items)) => items,
                _ => return vec![],
            },
            _ => return vec![],
        };
        items
            .into_iter()
            .filter_map(|raw| {
                let item = serde_json::from_value::<LspCompletionItem>(raw.clone()).ok()?;
                let mut item = Self::convert_lsp_completion(item);
                if raw.get("data").is_some_and(|data| !data.is_null()) {
                    item.data = Some(json!({ RESOLVE_DATA: {
                        "uri": ctx.uri, "revision": revision, "requestUri": uri, "item": raw,
                    }}));
                }
                Some(item)
            })
            .collect()
    }

    /// Resolve only documentation and detail. Insertion fields remain exactly
    /// as returned by completion until authored edit mapping supports them.
    pub(crate) async fn resolve(state: &ServerState, mut item: CompletionItem) -> CompletionItem {
        let Some(data) = item.data.as_ref().and_then(|data| data.get(RESOLVE_DATA)) else {
            return item;
        };
        let Some((uri, revision, request_uri, raw)) = resolve_data(data, &item.label) else {
            return item;
        };
        if !can_resolve(state, &uri, revision) {
            return item;
        }
        let Some(bridge) = state.get_corsa_bridge().await else {
            return item;
        };
        if !can_resolve(state, &uri, revision) {
            return item;
        }
        let Ok(Some(resolved)) = bridge.completion_resolve(&request_uri, raw).await else {
            return item;
        };
        if !can_resolve(state, &uri, revision) {
            return item;
        }
        if let Ok(resolved) = serde_json::from_value::<LspCompletionItem>(resolved)
            && resolved.label == item.label
        {
            let resolved = Self::convert_lsp_completion(resolved);
            item.detail = resolved.detail.or(item.detail);
            item.documentation = resolved.documentation.or(item.documentation);
        }
        item
    }
}

fn can_resolve(state: &ServerState, uri: &Url, revision: u64) -> bool {
    state.lsp_features().completion
        && state.is_lsp_typecheck_enabled()
        && is_current(state, uri, revision)
}

fn is_current(state: &ServerState, uri: &Url, revision: u64) -> bool {
    state
        .documents
        .get(uri)
        .is_some_and(|document| document.revision() == revision)
}

fn resolve_data(data: &Value, label: &str) -> Option<(Url, u64, String, Value)> {
    let uri = Url::parse(data.get("uri")?.as_str()?).ok()?;
    let revision = data.get("revision")?.as_u64()?;
    let raw = data.get("item")?;
    let request_uri = data.get("requestUri")?.as_str()?.to_owned();
    (raw.get("label")?.as_str()? == label).then(|| (uri, revision, request_uri, raw.clone()))
}
