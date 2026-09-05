//! Type-aware `textDocument/prepareCallHierarchy` for authored Vue files.

#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

#[cfg(feature = "native")]
use std::sync::Arc;

#[cfg(feature = "native")]
use tower_lsp::lsp_types::{CallHierarchyItem, Location, Range};
#[cfg(feature = "native")]
use vize_canon::{CorsaBridge, LspLocation, LspPosition, LspRange};

#[cfg(feature = "native")]
use super::{IdeContext, corsa_support};
#[cfg(feature = "native")]
use crate::virtual_code::BlockType;

/// Checker-backed call-hierarchy service.
pub struct CallHierarchyService;

#[cfg(feature = "native")]
impl CallHierarchyService {
    /// Prepare call-hierarchy items through Corsa's standard LSP surface and
    /// map every visible item span back onto authored source.
    pub async fn prepare_with_corsa(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Vec<CallHierarchyItem>> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        match ctx.block_type? {
            BlockType::Template | BlockType::Script | BlockType::ScriptSetup
                if !ctx.uri.path().ends_with(".art.vue") =>
            {
                Self::prepare_in_canonical_sfc(ctx, &bridge).await
            }
            BlockType::Template
            | BlockType::Script
            | BlockType::ScriptSetup
            | BlockType::Style(_)
            | BlockType::Art(_) => None,
        }
    }

    async fn prepare_in_canonical_sfc(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<Vec<CallHierarchyItem>> {
        let document = corsa_support::open_canonical_virtual_project_document(ctx, bridge).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
        let items = bridge
            .prepare_call_hierarchy(&document.request_uri, line, character)
            .await
            .ok()??;
        let items = serde_json::from_value::<Vec<CallHierarchyItem>>(items).ok()?;
        let items = items
            .into_iter()
            .filter_map(|item| Self::map_canonical_item(ctx, &document, item))
            .collect::<Vec<_>>();

        (!items.is_empty()).then_some(items)
    }

    fn map_canonical_item(
        ctx: &IdeContext<'_>,
        document: &corsa_support::CanonicalVirtualDocument,
        item: CallHierarchyItem,
    ) -> Option<CallHierarchyItem> {
        let selection =
            Self::map_canonical_item_range(ctx, document, &item.uri, item.selection_range)?;
        let range = Self::map_canonical_item_range(ctx, document, &item.uri, item.range)
            .filter(|range| range.uri == selection.uri)
            .unwrap_or_else(|| selection.clone());

        Some(CallHierarchyItem {
            uri: selection.uri,
            range: range.range,
            selection_range: selection.range,
            ..item
        })
    }

    fn map_canonical_item_range(
        ctx: &IdeContext<'_>,
        document: &corsa_support::CanonicalVirtualDocument,
        uri: &tower_lsp::lsp_types::Url,
        range: Range,
    ) -> Option<Location> {
        corsa_support::map_canonical_corsa_location(
            ctx,
            document,
            &LspLocation {
                uri: uri.to_string(),
                range: LspRange {
                    start: LspPosition {
                        line: range.start.line,
                        character: range.start.character,
                    },
                    end: LspPosition {
                        line: range.end.line,
                        character: range.end.character,
                    },
                },
            },
        )
    }
}

#[cfg(all(test, feature = "native"))]
mod tests;
