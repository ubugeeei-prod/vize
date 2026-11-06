//! Hover presentation from the current canonical native document.

use std::sync::Arc;
use tower_lsp::lsp_types::{Hover, Range};
use vize_canon::{CorsaBridge, CorsaVueHoverDocument, LspHover, LspHoverContents, LspRange};

use super::{HoverService, range::authored_hover_token_range};
use crate::ide::{IdeContext, corsa_support};

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

enum HoverDocument {
    Owned(Box<corsa_support::CanonicalVirtualDocument>),
    Shared(CorsaVueHoverDocument),
}

impl HoverDocument {
    fn request_uri(&self) -> &str {
        match self {
            Self::Owned(document) => &document.request_uri,
            Self::Shared(document) => document.request_uri(),
        }
    }

    fn source_offset_to_position(&self, source_offset: usize) -> Option<(u32, u32)> {
        match self {
            Self::Owned(document) => {
                corsa_support::canonical_source_offset_to_position(document, source_offset)
            }
            Self::Shared(document) => document.source_offset_to_position(source_offset),
        }
    }

    fn map_range(&self, ctx: &IdeContext<'_>, range: &LspRange) -> Option<Range> {
        match self {
            Self::Owned(document) => corsa_support::map_canonical_lsp_range(ctx, document, range),
            Self::Shared(document) => {
                let (start, end) = document.range_to_source_offsets(range)?;
                let start = start.min(ctx.content.len());
                let end = end.max(start).min(ctx.content.len());
                let (start_line, start_character) =
                    crate::ide::offset_to_position(&ctx.content, start);
                let (end_line, end_character) = crate::ide::offset_to_position(&ctx.content, end);
                Some(Range::new(
                    tower_lsp::lsp_types::Position::new(start_line, start_character),
                    tower_lsp::lsp_types::Position::new(end_line, end_character),
                ))
            }
        }
    }
}

pub(super) async fn hover(
    ctx: &IdeContext<'_>,
    bridge: Option<&Arc<CorsaBridge>>,
    follow_declaration: bool,
) -> Option<Answer> {
    // Each give-up is logged: on a host where typed hover silently falls back
    // (#8940) these lines are the only evidence of which step failed.
    let Some(bridge) = bridge.filter(|bridge| bridge.is_initialized()) else {
        tracing::info!(uri = %ctx.uri, "typed hover skipped: no initialized Corsa bridge");
        return None;
    };
    let opened = if ctx.has_initial_canonical_document(bridge) {
        corsa_support::open_canonical_virtual_document_strict(ctx, bridge)
            .await
            .map(|document| document.map(|document| HoverDocument::Owned(Box::new(document))))
    } else {
        corsa_support::open_canonical_hover_document_strict(ctx, bridge)
            .await
            .map(|document| document.map(HoverDocument::Shared))
    };
    let document = match opened {
        Ok(Some(document)) => document,
        Ok(None) => {
            tracing::warn!(uri = %ctx.uri, "typed hover skipped: no canonical document for this URI");
            return None;
        }
        Err(error) => {
            tracing::warn!(uri = %ctx.uri, %error, "typed hover skipped: canonical document failed");
            return None;
        }
    };
    let Some((line, character)) = document.source_offset_to_position(ctx.offset) else {
        tracing::info!(uri = %ctx.uri, offset = ctx.offset, "typed hover skipped: offset has no generated position");
        return None;
    };
    let mut hover = match bridge.hover(document.request_uri(), line, character).await {
        Ok(Some(hover)) => hover,
        Ok(None) => return Some(Answer::Empty),
        Err(error) => {
            tracing::warn!(
                uri = %ctx.uri,
                request_uri = %document.request_uri(),
                %error,
                "typed hover failed in Corsa"
            );
            return None;
        }
    };
    // Native quick info can omit JSDoc after a mapped type. Follow the
    // checker's declaration identities for documentation while retaining the
    // instantiated signature and the authored hover range from this use.
    if follow_declaration
        && documentation(&hover).is_none()
        && let Ok(definitions) = bridge
            .definition(document.request_uri(), line, character)
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
        .and_then(|range| document.map_range(ctx, range));
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
