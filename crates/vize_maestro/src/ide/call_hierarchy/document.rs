//! Reopen the current project before expanding an authored hierarchy item.

use serde_json::Value;
use tower_lsp::lsp_types::CallHierarchyItem;
use vize_canon::CorsaBridge;

use super::CallHierarchyService;
use crate::ide::{IdeContext, corsa_support};
use crate::virtual_code::BlockType;

pub(super) async fn open(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    workspace: bool,
) -> Option<corsa_support::CanonicalVirtualDocument> {
    if ctx.uri.path().ends_with(".vue") {
        if ctx.uri.path().ends_with(".art.vue")
            || !matches!(
                ctx.block_type?,
                BlockType::Script | BlockType::ScriptSetup | BlockType::Template
            )
        {
            return None;
        }
        if workspace {
            corsa_support::open_canonical_virtual_workspace_document(ctx, bridge).await
        } else {
            corsa_support::open_canonical_virtual_project_document(ctx, bridge).await
        }
    } else {
        corsa_support::open_canonical_script_document(ctx, bridge, workspace).await
    }
}

pub(super) async fn refresh_item(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    item: &CallHierarchyItem,
    bridge: &CorsaBridge,
) -> Option<Value> {
    // A prepared item's coordinates belong to one authored revision. Reusing
    // it after an edit can silently select a different function.
    let expected = item
        .data
        .as_ref()?
        .get("vizeCallHierarchySourceHash")?
        .as_str()?;
    if expected != vize_s0::hash::hash_str(&ctx.content).to_string() {
        return None;
    }
    let (line, character) =
        corsa_support::canonical_source_offset_to_position(document, ctx.offset)?;
    let items = bridge
        .prepare_call_hierarchy(&document.request_uri, line, character)
        .await
        .ok()??;
    // Native project reconstruction can change generated URIs and positions.
    // Resolve the same authored declaration in the current project snapshot.
    items.as_array()?.iter().find_map(|raw| {
        let candidate = serde_json::from_value::<CallHierarchyItem>(raw.clone()).ok()?;
        let mapped = CallHierarchyService::map_canonical_item(ctx, document, candidate, None)?;
        (mapped.uri == item.uri
            && mapped.selection_range == item.selection_range
            && mapped.name == item.name
            && mapped.kind == item.kind)
            .then(|| raw.clone())
    })
}
