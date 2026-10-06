//! Checker-owned labels retain their structured native presentation.

use super::InlayHintService;
use crate::ide::corsa_support::{
    CanonicalVirtualDocument, map_canonical_corsa_location, map_canonical_exact_edit_range,
    open_canonical_script_document, open_canonical_virtual_document,
};
use crate::ide::{IdeContext, offset_to_position};
use tower_lsp::lsp_types::{InlayHint, InlayHintLabel, Range};
use vize_canon::CorsaBridge;

impl InlayHintService {
    pub(crate) async fn native_hints(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
        range: Range,
    ) -> Vec<InlayHint> {
        if range.start > range.end {
            return Vec::new();
        }
        let document = if crate::utils::is_plain_script_path(ctx.uri.path())
            || crate::utils::is_jsx_path(ctx.uri.path())
        {
            open_canonical_script_document(ctx, bridge, false).await
        } else {
            open_canonical_virtual_document(ctx, bridge).await
        };
        let Some(document) = document else {
            return Vec::new();
        };
        // One range request, independent of the number of authored bindings.
        // Generated scaffolding is rejected by the exact authored projection.
        let end = offset_to_position(
            &document.virtual_result.code,
            document.virtual_result.code.len(),
        );
        let Ok(Some(response)) = bridge
            .inlay_hints(document.request_uri.as_str(), (0, 0), end)
            .await
        else {
            return Vec::new();
        };
        let Ok(hints) = serde_json::from_value::<Vec<InlayHint>>(response) else {
            return Vec::new();
        };
        hints
            .into_iter()
            .filter_map(|hint| map_hint(ctx, &document, range, hint))
            .collect()
    }
}

fn map_hint(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    requested: Range,
    mut hint: InlayHint,
) -> Option<InlayHint> {
    let point = Range::new(hint.position, hint.position);
    hint.position = map_canonical_exact_edit_range(ctx, document, point)?.start;
    if !InlayHintService::position_in_range(hint.position, requested) {
        return None;
    }
    if let Some(edits) = &mut hint.text_edits {
        for edit in edits {
            edit.range = map_canonical_exact_edit_range(ctx, document, edit.range)?;
        }
    }
    if let InlayHintLabel::LabelParts(parts) = &mut hint.label {
        for part in parts {
            if let Some(location) = &part.location {
                let native = serde_json::from_value(serde_json::to_value(location).ok()?).ok()?;
                part.location = map_canonical_corsa_location(ctx, document, &native);
            }
        }
    }
    Some(hint)
}

#[cfg(test)]
mod tests;
