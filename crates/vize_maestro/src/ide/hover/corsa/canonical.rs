//! Hover presentation from the current canonical native document.

use std::sync::Arc;
use tower_lsp::lsp_types::Hover;
use vize_canon::{CorsaBridge, LspHover, LspHoverContents};

use super::{HoverService, range::authored_hover_token_range};
use crate::ide::{IdeContext, corsa_support};

mod trace;

pub(super) async fn component_hover(
    ctx: &IdeContext<'_>,
    bridge: Option<&Arc<CorsaBridge>>,
) -> Option<Hover> {
    use super::super::component_prop;
    let mut result =
        component_prop::hover_attribute(ctx).or_else(|| component_prop::hover_event(ctx))?;
    if let Some(Answer::Hover(native)) = hover(ctx, bridge, true).await {
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

/// A native empty quick-info result is an answer, not backend unavailability.
pub(super) enum Answer {
    Empty,
    Hover(Hover),
}

pub(super) async fn hover(
    ctx: &IdeContext<'_>,
    bridge: Option<&Arc<CorsaBridge>>,
    follow_declaration: bool,
) -> Option<Answer> {
    let Some(bridge) = bridge.filter(|bridge| bridge.is_initialized()) else {
        trace::refused(ctx, "bridge-unavailable", None);
        return None;
    };
    let document = match corsa_support::open_canonical_virtual_document_strict(ctx, bridge).await {
        Ok(Some(document)) => document,
        Ok(None) => {
            trace::refused(ctx, "canonical-open-unavailable", None);
            return None;
        }
        Err(error) => {
            trace::refused(ctx, "canonical-open-error", Some(&error));
            return None;
        }
    };
    trace::document(ctx, &document);
    let Some((line, character)) =
        corsa_support::canonical_source_offset_to_position(&document, ctx.offset)
    else {
        trace::refused(ctx, "canonical-position-unmapped", None);
        return None;
    };
    trace::query(ctx, line, character);
    let mut hover = match bridge.hover(&document.request_uri, line, character).await {
        Ok(Some(hover)) => hover,
        Ok(None) => {
            trace::refused(ctx, "native-hover-empty", None);
            return Some(Answer::Empty);
        }
        Err(error) => {
            trace::refused(ctx, "native-hover-error", Some(&error));
            return None;
        }
    };
    trace::answer(ctx, &hover);
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
    Some(Answer::Hover(converted))
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
