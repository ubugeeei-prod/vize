//! Checker-backed call hierarchy over authored Vue and script project sources.

#![cfg_attr(
    feature = "native",
    expect(
        clippy::disallowed_types,
        clippy::disallowed_methods,
        reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
    )
)]

#[cfg(feature = "native")]
use std::sync::Arc;

#[cfg(feature = "native")]
use serde_json::Value;
#[cfg(feature = "native")]
use tower_lsp::lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyItem, CallHierarchyOutgoingCall, Location, Range,
};
#[cfg(feature = "native")]
use vize_canon::{CorsaBridge, LspLocation, LspPosition, LspRange};

#[cfg(feature = "native")]
use super::{IdeContext, corsa_support};
#[cfg(feature = "native")]
mod document;

/// Checker-backed call-hierarchy service.
pub struct CallHierarchyService;

#[cfg(feature = "native")]
const RAW_CALL_HIERARCHY_ITEM_DATA_KEY: &str = "vizeCorsaRawCallHierarchyItem";

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

        Self::prepare_in_canonical_sfc(ctx, &bridge).await
    }

    async fn prepare_in_canonical_sfc(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<Vec<CallHierarchyItem>> {
        let document = document::open(ctx, bridge, false).await?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
        let items = bridge
            .prepare_call_hierarchy(&document.request_uri, line, character)
            .await
            .ok()??;
        let items = items.as_array()?.clone();
        let items = items
            .into_iter()
            .filter_map(|raw_item| {
                let item = serde_json::from_value::<CallHierarchyItem>(raw_item.clone()).ok()?;
                Self::map_canonical_item(ctx, &document, item, Some(raw_item))
            })
            .collect::<Vec<_>>();

        (!items.is_empty()).then_some(items)
    }

    /// Resolve incoming calls for a prepared item and keep every visible span
    /// source-faithful.
    pub async fn incoming_calls_with_corsa(
        ctx: &IdeContext<'_>,
        item: &CallHierarchyItem,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Vec<CallHierarchyIncomingCall>> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let document = document::open(ctx, &bridge, true).await?;
        let raw_item = document::refresh_item(ctx, &document, item, &bridge).await?;
        let calls = bridge
            .call_hierarchy_incoming_calls(raw_item)
            .await
            .ok()?
            .unwrap_or_else(|| serde_json::json!([]));
        Self::map_incoming_calls(ctx, &document, calls)
    }

    /// Resolve outgoing calls for a prepared item and keep call-site ranges on
    /// authored Vue text.
    pub async fn outgoing_calls_with_corsa(
        ctx: &IdeContext<'_>,
        item: &CallHierarchyItem,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Vec<CallHierarchyOutgoingCall>> {
        let bridge = corsa_bridge?;
        if !bridge.is_initialized() {
            return None;
        }

        let document = document::open(ctx, &bridge, false).await?;
        let raw_item = document::refresh_item(ctx, &document, item, &bridge).await?;
        let origin_uri = raw_item.get("uri")?.as_str()?.to_owned();
        let calls = bridge.call_hierarchy_outgoing_calls(raw_item).await.ok()?;
        let calls = calls.unwrap_or_else(|| serde_json::json!([]));
        Self::map_outgoing_calls(ctx, &document, &origin_uri, calls)
    }

    fn map_canonical_item(
        ctx: &IdeContext<'_>,
        document: &corsa_support::CanonicalVirtualDocument,
        mut item: CallHierarchyItem,
        raw_item: Option<Value>,
    ) -> Option<CallHierarchyItem> {
        let selection =
            Self::map_canonical_item_range(ctx, document, &item.uri, item.selection_range)?;
        let range = Self::map_canonical_item_range(ctx, document, &item.uri, item.range)
            .filter(|range| range.uri == selection.uri)
            .unwrap_or_else(|| selection.clone());
        if let Some(raw_item) = raw_item {
            let source = if selection.uri == *ctx.uri {
                ctx.content.to_string()
            } else if let Some(source) = document.authored_source(&selection.uri) {
                source.to_owned()
            } else {
                ctx.state
                    .documents
                    .text(&selection.uri)
                    .or_else(|| std::fs::read_to_string(selection.uri.to_file_path().ok()?).ok())?
            };
            item.data = Some(serde_json::json!({
                RAW_CALL_HIERARCHY_ITEM_DATA_KEY: raw_item,
                "vizeCallHierarchySourceHash": vize_s0::hash::hash_str(&source).to_string(),
            }));
        }

        Some(CallHierarchyItem {
            uri: selection.uri,
            range: range.range,
            selection_range: selection.range,
            ..item
        })
    }

    fn map_incoming_calls(
        ctx: &IdeContext<'_>,
        document: &corsa_support::CanonicalVirtualDocument,
        calls: Value,
    ) -> Option<Vec<CallHierarchyIncomingCall>> {
        let calls = calls.as_array()?.clone();
        let calls = calls
            .into_iter()
            .filter_map(|raw_call| {
                let call =
                    serde_json::from_value::<CallHierarchyIncomingCall>(raw_call.clone()).ok()?;
                let raw_from = raw_call.get("from").cloned();
                let raw_from_uri = call.from.uri.clone();
                let from = Self::map_canonical_item(ctx, document, call.from, raw_from)?;
                let from_ranges = Self::map_call_ranges(
                    ctx,
                    document,
                    &raw_from_uri,
                    &from.uri,
                    call.from_ranges,
                );
                Some(CallHierarchyIncomingCall { from, from_ranges })
            })
            .collect::<Vec<_>>();
        Some(calls)
    }

    fn map_outgoing_calls(
        ctx: &IdeContext<'_>,
        document: &corsa_support::CanonicalVirtualDocument,
        origin_uri: &str,
        calls: Value,
    ) -> Option<Vec<CallHierarchyOutgoingCall>> {
        let calls = calls.as_array()?.clone();
        let origin_uri = tower_lsp::lsp_types::Url::parse(origin_uri).ok()?;
        let calls = calls
            .into_iter()
            .filter_map(|raw_call| {
                let call =
                    serde_json::from_value::<CallHierarchyOutgoingCall>(raw_call.clone()).ok()?;
                let raw_to = raw_call.get("to").cloned();
                let to = Self::map_canonical_item(ctx, document, call.to, raw_to)?;
                let from_ranges =
                    Self::map_call_ranges(ctx, document, &origin_uri, ctx.uri, call.from_ranges);
                Some(CallHierarchyOutgoingCall { to, from_ranges })
            })
            .collect::<Vec<_>>();
        Some(calls)
    }

    fn map_call_ranges(
        ctx: &IdeContext<'_>,
        document: &corsa_support::CanonicalVirtualDocument,
        raw_uri: &tower_lsp::lsp_types::Url,
        expected_uri: &tower_lsp::lsp_types::Url,
        ranges: Vec<Range>,
    ) -> Vec<Range> {
        ranges
            .into_iter()
            .filter_map(|range| Self::map_canonical_item_range(ctx, document, raw_uri, range))
            .filter(|location| location.uri == *expected_uri)
            .map(|location| location.range)
            .collect()
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
