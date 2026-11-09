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
    let card = component_prop::hover_attribute(ctx).or_else(|| component_prop::hover_event(ctx));
    // A listener whose contract resident metadata cannot read (an aliased
    // `defineEmits<Emits>()`, an Options API child) still has a typed answer.
    if card.is_none() && !super::component_api::is_component_event_query(ctx) {
        return None;
    }
    let native = match hover(ctx, bridge, true).await {
        Some(Answer::Hover(native)) => Some(native),
        _ => None,
    };
    let mut result = match (card, native) {
        (Some(card), None) => card,
        (_, Some(native)) => {
            let mut documented = component_prop::hover_attribute_documented(ctx, Some(&native))
                .or_else(|| component_prop::hover_event_documented(ctx, Some(&native)))
                .unwrap_or_else(|| native.clone());
            documented.range = native.range;
            documented
        }
        (None, None) => return None,
    };
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

    fn instantiated_component_prop_position(&self, ctx: &IdeContext<'_>) -> Option<(u32, u32)> {
        match self {
            Self::Owned(document) => {
                corsa_support::instantiated_component_attribute_position(ctx, document)
            }
            Self::Shared(document) => document.instantiated_component_prop_position(ctx.offset),
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
    let profiler = vize_l0::profiler::global_profiler();
    let started = profiler.is_enabled().then(std::time::Instant::now);
    let opened = if ctx.has_initial_canonical_document(bridge) {
        corsa_support::open_canonical_virtual_document_strict(ctx, bridge)
            .await
            .map(|document| document.map(|document| HoverDocument::Owned(Box::new(document))))
    } else {
        corsa_support::open_canonical_hover_document_strict(ctx, bridge)
            .await
            .map(|document| document.map(HoverDocument::Shared))
    };
    if let Some(started) = started {
        profiler.record("maestro.hover.canonical.open", started.elapsed());
    }
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
    let Some((line, character)) = vize_l0::profile!(
        "maestro.hover.canonical.map_to_generated",
        document.source_offset_to_position(ctx.offset)
    ) else {
        tracing::info!(uri = %ctx.uri, offset = ctx.offset, "typed hover skipped: offset has no generated position");
        return None;
    };
    let started = profiler.is_enabled().then(std::time::Instant::now);
    let response = bridge.hover(document.request_uri(), line, character).await;
    if let Some(started) = started {
        profiler.record("maestro.hover.canonical.backend", started.elapsed());
    }
    let mut query_position = (line, character);
    let mut hover = match response {
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
    if follow_declaration
        && is_unknown_prop_hover(&hover)
        && let Some(position) = document.instantiated_component_prop_position(ctx)
        && position != query_position
        && let Ok(Some(instantiated)) = bridge
            .hover(document.request_uri(), position.0, position.1)
            .await
        && !is_unknown_prop_hover(&instantiated)
    {
        hover = instantiated;
        query_position = position;
    }
    if follow_declaration {
        normalize_instantiated_prop_hover(&mut hover);
    }
    // Retain the instantiated signature and authored range from this use.
    if follow_declaration || super::documentation::is_member_quick_info(&hover) {
        super::documentation::adopt_declaration_documentation(
            bridge,
            document.request_uri(),
            query_position,
            &mut hover,
        )
        .await;
    }
    vize_l0::profile!("maestro.hover.canonical.present", {
        let mapped_range = hover
            .range
            .as_ref()
            .and_then(|range| document.map_range(ctx, range));
        let mut converted = HoverService::convert_lsp_hover(hover);
        converted.range = mapped_range.or_else(|| authored_hover_token_range(ctx));
        Some(Answer::Hover(converted))
    })
}

fn is_unknown_prop_hover(hover: &LspHover) -> bool {
    let LspHoverContents::Markup(content) = &hover.contents else {
        return false;
    };
    if content.kind != "markdown" {
        return false;
    }
    let value = content.value.trim();
    if value == "unknown" {
        return true;
    }
    let Some(signature) = value
        .strip_prefix("```typescript\n")
        .and_then(|value| value.split_once("\n```").map(|(signature, _)| signature))
    else {
        return false;
    };
    let signature = signature.trim();
    signature == "unknown"
        || signature
            .strip_prefix("(property) ")
            .and_then(|property| property.split_once(':'))
            .is_some_and(|(_, ty)| ty.trim() == "unknown")
}

/// TypeScript's platform packages disagree about quoting a string-literal key
/// when it is also a legal identifier. Keep the public hover stable while
/// preserving quotes for keys that actually require them.
fn normalize_instantiated_prop_hover(hover: &mut LspHover) {
    let LspHoverContents::Markup(content) = &mut hover.contents else {
        return;
    };
    let prefix = "```typescript\n(property) ";
    let Some(signature) = content.value.strip_prefix(prefix) else {
        return;
    };
    let Some(quote) = signature
        .chars()
        .next()
        .filter(|quote| matches!(quote, '"' | '\''))
    else {
        return;
    };
    let Some((name, after)) = signature
        .get(1..)
        .and_then(|quoted| quoted.split_once(quote))
    else {
        return;
    };
    if !after.starts_with(':') && !after.starts_with("?:") {
        return;
    }
    let mut characters = name.chars();
    if !characters.next().is_some_and(|character| {
        character == '_' || character == '$' || character.is_ascii_alphabetic()
    }) || !characters
        .all(|character| character == '_' || character == '$' || character.is_ascii_alphanumeric())
    {
        return;
    }
    let quote_start = prefix.len();
    let quote_end = quote_start + 1 + name.len();
    content.value.remove(quote_end);
    content.value.remove(quote_start);
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_span::SourceType;
    use tower_lsp::lsp_types::{Position, Url};
    use vize_canon::{
        CorsaSourceCatalog, CorsaVueVirtualDocument, LspMarkupContent, LspPosition,
        virtual_ts::ProjectionMapping,
    };

    use super::*;
    use crate::ide::DiagnosticService;
    use crate::server::ServerState;

    fn markup_hover(value: &str) -> LspHover {
        LspHover {
            contents: LspHoverContents::Markup(LspMarkupContent {
                kind: "markdown".into(),
                value: value.into(),
            }),
            range: None,
        }
    }

    #[test]
    fn contextual_retry_only_replaces_an_exact_unknown_type() {
        for value in [
            "unknown",
            "```typescript\nunknown\n```",
            "```typescript\n(property) items: unknown\n```\n\nDocumentation",
            "```typescript\n(property) \"items\": unknown\n```",
        ] {
            assert!(is_unknown_prop_hover(&markup_hover(value)), "{value}");
        }
        for value in [
            "```typescript\n(property) items: unknown[]\n```",
            "```typescript\n(property) items: T[]\n```",
            "```typescript\n(property) items: Record<string, unknown>\n```",
            "```typescript\nfunction unknown(): void\n```",
        ] {
            assert!(!is_unknown_prop_hover(&markup_hover(value)), "{value}");
        }
    }

    #[test]
    fn contextual_identifier_keys_have_one_cross_platform_signature() {
        let mut identifier =
            markup_hover("```typescript\n(property) \"items\": number[]\n```\n\nDocumentation");
        normalize_instantiated_prop_hover(&mut identifier);
        let LspHoverContents::Markup(content) = identifier.contents else {
            panic!("markup hover");
        };
        assert_eq!(
            content.value,
            "```typescript\n(property) items: number[]\n```\n\nDocumentation"
        );

        let mut optional = markup_hover("```typescript\n(property) \"item\"?: number\n```");
        normalize_instantiated_prop_hover(&mut optional);
        let LspHoverContents::Markup(content) = optional.contents else {
            panic!("markup hover");
        };
        assert_eq!(
            content.value,
            "```typescript\n(property) item?: number\n```"
        );

        let mut single =
            markup_hover("```typescript\n(property) 'change': (value: string) => any\n```");
        normalize_instantiated_prop_hover(&mut single);
        let LspHoverContents::Markup(content) = single.contents else {
            panic!("markup hover");
        };
        assert_eq!(
            content.value,
            "```typescript\n(property) change: (value: string) => any\n```"
        );

        let mut literal = markup_hover("```typescript\n(property) \"item-kind\": string\n```");
        normalize_instantiated_prop_hover(&mut literal);
        let LspHoverContents::Markup(content) = literal.contents else {
            panic!("markup hover");
        };
        assert_eq!(
            content.value,
            "```typescript\n(property) \"item-kind\": string\n```"
        );
    }

    #[test]
    fn shared_hover_mapping_matches_owned_nested_expression_mapping() {
        let uri = Url::parse("file:///tmp/NestedHover.vue").unwrap();
        let source = r#"<script setup lang="ts">
import Child from "./Child.vue"
const user = { profile: { labels: ["前🌸"] } }
const index = 0
const fallback = "unknown"
</script>
<template>
  <Child :title="user.profile?.labels[index]?.trim() ?? fallback" />
</template>
"#;
        let virtual_result = DiagnosticService::generate_virtual_ts(&uri, source, false, false)
            .expect("nested expression projection");
        let shared = HoverDocument::Shared(
            CorsaVueVirtualDocument {
                request_uri: "file:///tmp/NestedHover.vue.ts".into(),
                code: virtual_result.code.clone(),
                pre_rewrite_code: "".into(),
                mapping: ProjectionMapping::from_spans(virtual_result.source_mappings.clone()),
                import_source_map: virtual_result.import_source_map.clone(),
                source_type: SourceType::from_path(Path::new("NestedHover.ts")).unwrap(),
                virtual_suffix: ".ts",
                dependencies: Vec::new(),
                resolved_dependencies: Vec::new(),
                materialized_sources: Vec::new(),
                source_catalog: CorsaSourceCatalog::default(),
                session_project_root: None,
            }
            .into(),
        );
        let owned = HoverDocument::Owned(Box::new(corsa_support::CanonicalVirtualDocument {
            source_uri: uri.clone(),
            request_uri: "file:///tmp/NestedHover.vue.ts".into(),
            virtual_result,
            dependencies: Vec::new(),
            materialized_sources: Vec::new(),
            session_project_roots: Vec::new(),
            source_catalogs: Vec::new(),
        }));
        let state = ServerState::new();

        for token in ["labels", "trim", "fallback"] {
            let start = source.rfind(token).unwrap();
            let end = start + token.len();
            let ctx = IdeContext::testing(&state, &uri, start, source.into());
            let owned_start = owned.source_offset_to_position(start).unwrap();
            let owned_end = owned.source_offset_to_position(end).unwrap();
            assert_eq!(shared.source_offset_to_position(start), Some(owned_start));
            assert_eq!(shared.source_offset_to_position(end), Some(owned_end));

            let generated = LspRange {
                start: LspPosition {
                    line: owned_start.0,
                    character: owned_start.1,
                },
                end: LspPosition {
                    line: owned_end.0,
                    character: owned_end.1,
                },
            };
            let expected_start = crate::ide::offset_to_position(source, start);
            let expected_end = crate::ide::offset_to_position(source, end);
            let expected = Range::new(
                Position::new(expected_start.0, expected_start.1),
                Position::new(expected_end.0, expected_end.1),
            );
            assert_eq!(owned.map_range(&ctx, &generated), Some(expected));
            assert_eq!(shared.map_range(&ctx, &generated), Some(expected));
        }
    }
}
