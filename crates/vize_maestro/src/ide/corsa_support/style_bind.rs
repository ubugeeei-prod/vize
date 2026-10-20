//! Type-aware queries from a CSS `v-bind()` expression.
//!
//! The canonical document does not emit style bindings. For a query inside
//! one, a copy of it with `void (<expression>);` at the top of the template
//! scope, where template expressions resolve, is opened beside it. Answers
//! map back through the canonical mapping; an answer inside the inserted
//! expression lands on the authored `v-bind()` argument.
#![expect(
    clippy::disallowed_types,
    reason = "the projection text is a std String sent to the native bridge"
)]

use std::ops::Range;

use tower_lsp::lsp_types::{Location, Position};
use vize_canon::{CorsaBridge, LspLocation, LspPosition, LspRange};

use super::{CanonicalVirtualDocument, map_canonical_corsa_location};
use crate::ide::{IdeContext, offset_to_position, position_to_offset};
use crate::virtual_code::BlockType;

const TEMPLATE_SCOPE: &str = ";(function __template() {\n";

pub(crate) struct StyleBindProjection {
    pub(crate) document: CanonicalVirtualDocument,
    pub(crate) request_uri: vize_l0::String,
    text: String,
    /// Byte offset of the inserted statement in `text`.
    inserted_at: usize,
    inserted_len: usize,
    /// The expression in `text` and in the SFC.
    generated: Range<usize>,
    authored: Range<usize>,
}

impl StyleBindProjection {
    /// Open the projection and return the cursor's position in it, or `None`
    /// when the cursor is not inside a single-line, valid expression.
    pub(crate) async fn open(
        ctx: &IdeContext<'_>,
        bridge: &CorsaBridge,
    ) -> Option<(Self, u32, u32)> {
        let authored = authored_expression(ctx)?;
        let document = super::open_canonical_virtual_document(ctx, bridge).await?;
        let code = document.virtual_result.code.as_str();
        let scope = code.find(TEMPLATE_SCOPE)?;
        if code.get(scope + 1..)?.contains(TEMPLATE_SCOPE) {
            return None;
        }
        let inserted_at = scope + TEMPLATE_SCOPE.len();
        let expression = ctx.content.get(authored.clone())?;
        let statement = vize_l0::cstr!("void ({expression});\n");
        let start = inserted_at + "void (".len();
        let generated = start..start + expression.len();
        let mut text = String::with_capacity(code.len() + statement.len());
        text.push_str(code.get(..inserted_at)?);
        text.push_str(&statement);
        text.push_str(code.get(inserted_at..)?);
        let path = super::virtual_document_path(&document.request_uri)?;
        let path = vize_l0::cstr!("{path}.__vize_style__.ts");
        let request_uri = bridge
            .open_or_update_virtual_document(&path, &text)
            .await
            .ok()?;
        let cursor = generated.start + (ctx.offset - authored.start);
        let (line, character) = offset_to_position(&text, cursor);
        let projection = Self {
            document,
            request_uri,
            text,
            inserted_at,
            inserted_len: statement.len(),
            generated,
            authored,
        };
        Some((projection, line, character))
    }

    /// Map one native answer back to authored source. `Err` carries the
    /// canonical-document location of an answer with no authored span.
    fn map_location(
        &self,
        ctx: &IdeContext<'_>,
        location: &LspLocation,
    ) -> Result<Location, Option<LspLocation>> {
        if location.uri != self.request_uri.as_str() {
            return map_canonical_corsa_location(ctx, &self.document, location)
                .ok_or_else(|| Some(location.clone()));
        }
        let (Some(start), Some(end)) = (
            self.offset(&location.range.start),
            self.offset(&location.range.end),
        ) else {
            return Err(None);
        };
        if self.generated.start <= start && end <= self.generated.end {
            let shift = |offset: usize| {
                let (line, character) = offset_to_position(
                    &ctx.content,
                    self.authored.start + offset - self.generated.start,
                );
                Position::new(line, character)
            };
            return Ok(Location {
                uri: ctx.uri.clone(),
                range: tower_lsp::lsp_types::Range::new(shift(start), shift(end)),
            });
        }
        let inserted_end = self.inserted_at + self.inserted_len;
        let (start, end) = if end <= self.inserted_at {
            (start, end)
        } else if start >= inserted_end {
            (start - self.inserted_len, end - self.inserted_len)
        } else {
            return Err(None);
        };
        let code = self.document.virtual_result.code.as_str();
        let position = |offset: usize| {
            let (line, character) = offset_to_position(code, offset);
            LspPosition { line, character }
        };
        let canonical = LspLocation {
            uri: String::from(self.document.request_uri.as_str()),
            range: LspRange {
                start: position(start),
                end: position(end),
            },
        };
        map_canonical_corsa_location(ctx, &self.document, &canonical).ok_or(Some(canonical))
    }

    pub(crate) async fn close(self, bridge: &CorsaBridge) {
        if let Err(error) = bridge.close_virtual_document(&self.request_uri).await {
            tracing::warn!(%error, "style binding projection could not be closed");
        }
    }

    fn offset(&self, position: &LspPosition) -> Option<usize> {
        position_to_offset(&self.text, position.line, position.character)
    }
}

/// The `v-bind()` argument around the cursor, if it is one valid expression.
fn authored_expression(ctx: &IdeContext<'_>) -> Option<Range<usize>> {
    let Some(BlockType::Style(index)) = ctx.block_type else {
        return None;
    };
    let style = ctx.descriptor()?.styles.get(index)?;
    let base = style.loc.start;
    let content = ctx.content.get(base..style.loc.end)?;
    let argument = crate::ide::semantic_tokens::style::v_binds(content)
        .into_iter()
        .filter_map(|v_bind| v_bind.argument)
        .map(|argument| base + argument.start..base + argument.end)
        .find(|argument| argument.start <= ctx.offset && ctx.offset <= argument.end)?;
    let expression = ctx.content.get(argument.clone())?;
    if expression.trim().is_empty() || expression.contains(['\n', '\r']) {
        return None;
    }
    let allocator = oxc_allocator::Allocator::default();
    oxc_parser::Parser::new(&allocator, expression, oxc_span::SourceType::ts())
        .parse_expression()
        .ok()?;
    Some(argument)
}

/// A location query asked from a `v-bind()` expression.
#[derive(Clone, Copy)]
pub(crate) enum StyleBindQuery {
    TypeDefinition,
    Implementation,
    Declaration,
}

/// Answer `query` at the cursor inside a `v-bind()` expression.
pub(crate) async fn style_bind_locations(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    query: StyleBindQuery,
) -> Vec<Location> {
    let Some((projection, line, character)) = StyleBindProjection::open(ctx, bridge).await else {
        return Vec::new();
    };
    let uri = projection.request_uri.as_str();
    let locations = match query {
        StyleBindQuery::TypeDefinition => bridge.type_definition(uri, line, character).await,
        StyleBindQuery::Implementation => bridge.implementation(uri, line, character).await,
        StyleBindQuery::Declaration => match bridge.declaration(uri, line, character).await {
            Ok(locations) if !locations.is_empty() => Ok(locations),
            Ok(_) | Err(_) => bridge.definition(uri, line, character).await,
        },
    };
    let mut mapped = Vec::new();
    for location in locations.unwrap_or_default() {
        let canonical = match projection.map_location(ctx, &location) {
            Ok(found) => {
                mapped.push(found);
                continue;
            }
            Err(Some(canonical)) if !matches!(query, StyleBindQuery::TypeDefinition) => canonical,
            Err(_) => continue,
        };
        // A template ref-unwrap shadow has no authored span; its semantic
        // link leads to the setup binding, as from a template expression.
        let document = &projection.document;
        for position in super::linked_semantic_positions(document, &canonical.uri, &canonical.range)
        {
            let (uri, line, character) = (&position.request_uri, position.line, position.character);
            let found = match query {
                StyleBindQuery::Implementation => bridge.implementation(uri, line, character).await,
                _ => bridge.declaration(uri, line, character).await,
            };
            mapped.extend(super::map_canonical_corsa_locations(
                ctx,
                document,
                found.unwrap_or_default(),
            ));
        }
    }
    mapped.dedup();
    projection.close(bridge).await;
    mapped
}
