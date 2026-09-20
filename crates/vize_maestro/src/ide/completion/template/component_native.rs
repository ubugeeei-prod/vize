//! Authored component documentation without changing Vue insertion behavior.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use vize_canon::CorsaBridge;

use super::{component_meta, tag_context};
use crate::ide::completion::CompletionService;
use crate::ide::{IdeContext, corsa_support, is_component_tag, pascal_to_kebab};

pub(in crate::ide::completion) async fn complete_with_corsa(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
) -> Option<Vec<CompletionItem>> {
    let tag = tag_context::opening_tag_context_at_offset(&ctx.content, ctx.offset)?;
    if tag.inside_attribute_value
        || !is_component_tag(&tag.tag_name)
        || !tag_context::is_prop_completion_prefix(&tag.current_token)
        || component_meta::component_surface_completions(ctx).is_empty()
    {
        return None;
    }
    let mut items = super::complete_template(ctx);
    let document = corsa_support::open_canonical_virtual_document(ctx, bridge).await?;
    let (line, character) =
        corsa_support::canonical_component_prop_position(&document, tag.tag_start + 1)?;
    let native =
        CompletionService::request_resolvable(ctx, bridge, &document.request_uri, line, character)
            .await;
    for item in &mut items {
        if item.kind != Some(CompletionItemKind::PROPERTY) {
            continue;
        }
        if let Some(candidate) = native.iter().find(|candidate| {
            let name = candidate.filter_text.as_deref().unwrap_or(&candidate.label);
            name == item.label || pascal_to_kebab(name) == item.label
        }) {
            CompletionService::attach_native_documentation(item, candidate);
        }
    }
    Some(items)
}
