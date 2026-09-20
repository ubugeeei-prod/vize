//! Hover presentation from the current canonical native document.

use std::sync::Arc;
use tower_lsp::lsp_types::Hover;
use vize_canon::{CorsaBridge, LspHover, LspHoverContents};

use super::{HoverService, range::authored_hover_token_range};
use crate::ide::{IdeContext, corsa_support};

pub(super) async fn component_hover(
    ctx: &IdeContext<'_>,
    bridge: Option<&Arc<CorsaBridge>>,
) -> Option<Hover> {
    use super::super::component_prop;
    let mut result =
        component_prop::hover_attribute(ctx).or_else(|| component_prop::hover_event(ctx))?;
    if let Some(native) = hover(ctx, bridge, true).await {
        result = component_prop::hover_attribute_documented(ctx, Some(&native))
            .or_else(|| component_prop::hover_event_documented(ctx, Some(&native)))
            .unwrap_or_else(|| native.clone());
        result.range = native.range;
    }
    if result.range.is_none() {
        result.range = authored_hover_token_range(ctx);
    }
    Some(result)
}

pub(super) async fn hover(
    ctx: &IdeContext<'_>,
    bridge: Option<&Arc<CorsaBridge>>,
    follow_declaration: bool,
) -> Option<Hover> {
    let bridge = bridge.filter(|bridge| bridge.is_initialized())?;
    let document = corsa_support::open_canonical_virtual_document(ctx, bridge).await?;
    let (line, character) =
        corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
    let mut hover = bridge
        .hover(&document.request_uri, line, character)
        .await
        .ok()??;
    // Native quick info can omit JSDoc after a mapped type. Follow the
    // checker's declaration identities for documentation while retaining the
    // instantiated signature and the authored hover range from this use.
    if follow_declaration
        && documentation(&hover).is_none()
        && let Ok(definitions) = bridge
            .definition(&document.request_uri, line, character)
            .await
    {
        for definition in definitions.into_iter().take(8) {
            let Ok(Some(origin)) = bridge
                .hover(
                    &definition.uri,
                    definition.range.start.line,
                    definition.range.start.character,
                )
                .await
            else {
                continue;
            };
            if let Some(documentation) = documentation(&origin)
                && let LspHoverContents::Markup(content) = &mut hover.contents
            {
                content.value.push_str("\n\n");
                content.value.push_str(documentation);
                break;
            }
        }
    }
    let mapped_range = hover
        .range
        .as_ref()
        .and_then(|range| corsa_support::map_canonical_lsp_range(ctx, &document, range));
    let mut converted = HoverService::convert_lsp_hover(hover);
    converted.range = mapped_range.or_else(|| authored_hover_token_range(ctx));
    Some(converted)
}

fn documentation(hover: &LspHover) -> Option<&str> {
    let LspHoverContents::Markup(content) = &hover.contents else {
        return None;
    };
    if content.kind != "markdown" {
        return None;
    }
    let (_, documentation) = content
        .value
        .strip_prefix("```typescript\n")?
        .split_once("\n```")?;
    let documentation = documentation.trim();
    (!documentation.is_empty()).then_some(documentation)
}
