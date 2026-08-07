//! Template *member access* completion over the canonical document.
//!
//! Member access is the only template position whose answer the structural
//! list cannot produce: it cannot enumerate the properties of an arbitrary
//! type. So `it.` is routed through the checker over the canonical-document
//! route hover and definition already take (#3321), instead of the legacy
//! per-block scope list that answered with the SFC's bindings (#3911).
//!
//! An identifier position keeps the structural answer, which knows the SFC's
//! bindings, ranks them for a template author (#3224), and carries neither the
//! generated machinery nor the DOM globals the raw checker scope holds. Member
//! names, by contrast, are properties of an authored type, so the checker's
//! answer needs no curation: filtering it by label would eat authored members
//! that happen to be named `__user` or `defineProps`.

use std::sync::Arc;

use tower_lsp::lsp_types::CompletionItem;
use vize_canon::CorsaBridge;

use crate::ide::template_expression::{
    is_at_member_access_position, is_in_vue_template_expression,
};
use crate::ide::{IdeContext, corsa_support};

/// Answer a template member access through the checker, or return no items so
/// the caller falls back to the structural template list.
pub(super) async fn complete(
    ctx: &IdeContext<'_>,
    bridge: &Arc<CorsaBridge>,
) -> Vec<CompletionItem> {
    if !is_member_access_expression(ctx) {
        return vec![];
    }

    if bridge.is_initialized()
        && let Some(doc) = corsa_support::open_canonical_virtual_document(ctx, bridge).await
        && let Some((line, character)) =
            corsa_support::canonical_source_offset_to_position(&doc, ctx.offset)
        && let Ok(items) = bridge.completion(&doc.request_uri, line, character).await
    {
        return items
            .into_iter()
            .map(super::CompletionService::convert_lsp_completion)
            .collect();
    }

    vec![]
}

/// Whether the cursor sits on a member name inside a template expression.
pub(super) fn is_member_access_expression(ctx: &IdeContext<'_>) -> bool {
    is_in_vue_template_expression(&ctx.content, ctx.offset)
        && is_at_member_access_position(&ctx.content, ctx.offset)
}
