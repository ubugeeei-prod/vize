//! Literal-value completion keeps the checker's whole answer, without bindings.

use oxc_allocator::Allocator;
use oxc_parser::{Kind, Parser, config::TokensParserConfig};
use oxc_span::SourceType;

use crate::ide::IdeContext;
use crate::virtual_code::{BlockType, ProjectionFeatures};

#[cfg(test)]
mod tests;

pub(super) fn contains_cursor(ctx: &IdeContext<'_>) -> bool {
    if !ctx.uri.path().ends_with(".vue") || ctx.uri.path().ends_with(".art.vue") {
        return false;
    }
    match ctx.block_type {
        Some(BlockType::Script | BlockType::ScriptSetup) => {
            let Some(descriptor) = ctx.descriptor() else {
                return false;
            };
            let block = if matches!(ctx.block_type, Some(BlockType::ScriptSetup)) {
                descriptor.script_setup.as_ref()
            } else {
                descriptor.script.as_ref()
            };
            let Some(block) = block else { return false };
            let Some(offset) = ctx.offset.checked_sub(block.loc.start) else {
                return false;
            };
            quoted_token(
                block.content.as_ref(),
                offset,
                source_type(block.lang.as_deref()),
            )
            .is_some()
        }
        Some(BlockType::Template)
            if quote_candidate(&ctx.content, ctx.offset)
                && crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset)
                && !super::is_inside_html_comment(&ctx.content, ctx.offset) =>
        {
            let Some(document) = ctx
                .virtual_docs
                .as_ref()
                .and_then(|docs| docs.template.as_ref())
            else {
                return false;
            };
            let Some(offset) = document
                .source_map
                .to_generated_for(ctx.offset, ProjectionFeatures::COMPLETION)
            else {
                return false;
            };
            let Some(span) = quoted_token(&document.content, offset, SourceType::ts()) else {
                return false;
            };
            // The entire lexer-owned literal must still be the authored bytes.
            // Generated scaffolding and stale projections cannot grant a route.
            let Some(authored) = document
                .source_map
                .generated_range_to_authored(span.clone())
                .and_then(|authored| ctx.content.get(authored))
            else {
                return false;
            };
            document.content.get(span) == Some(authored)
        }
        _ => false,
    }
}

fn source_type(lang: Option<&str>) -> SourceType {
    match lang.unwrap_or("js") {
        "ts" => SourceType::ts().with_module(true),
        "tsx" => SourceType::tsx().with_module(true),
        "jsx" => SourceType::jsx().with_module(true),
        _ => SourceType::mjs(),
    }
}

// This is only a negative cost guard. The parser's lexer owns every positive.
// Continued quoted lines remain candidates; regex/comment quotes grant nothing.
fn quote_candidate(source: &str, offset: usize) -> bool {
    let Some(before) = source.get(..offset) else {
        return false;
    };
    let line = before.rsplit(['\r', '\n']).next().unwrap_or_default();
    line.bytes().any(|byte| matches!(byte, b'\'' | b'"' | b'`'))
        || before.contains('`')
        || before.contains("\\\n")
        || before.contains("\\\r\n")
}

fn quoted_token(
    source: &str,
    offset: usize,
    source_type: SourceType,
) -> Option<std::ops::Range<usize>> {
    if !quote_candidate(source, offset) {
        return None;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type)
        .with_config(TokensParserConfig)
        .parse();
    parsed.tokens.iter().find_map(|token| {
        let start = token.start() as usize;
        let end = token.end() as usize;
        if start >= offset || offset > end {
            return None;
        }
        match token.kind() {
            Kind::Str | Kind::NoSubstitutionTemplate if offset < end => Some(start..end),
            // Incomplete authored quotes still cannot accept identifier extras.
            // The native provider, not recovered syntax, decides their values.
            Kind::Undetermined
                if source
                    .as_bytes()
                    .get(start)
                    .is_some_and(|byte| matches!(byte, b'\'' | b'"' | b'`')) =>
            {
                Some(start..end)
            }
            _ => None,
        }
    })
}

#[cfg(feature = "native")]
pub(super) async fn complete(
    ctx: &IdeContext<'_>,
    bridge: Option<&vize_canon::CorsaBridge>,
) -> Option<tower_lsp::lsp_types::CompletionResponse> {
    let bridge = bridge.filter(|bridge| bridge.is_initialized())?;
    let document = crate::ide::corsa_support::open_canonical_virtual_document(ctx, bridge).await?;
    let (line, character) =
        crate::ide::corsa_support::canonical_source_offset_to_position(&document, ctx.offset)?;
    let items = super::CompletionService::request_resolvable(
        ctx,
        bridge,
        &document.request_uri,
        line,
        character,
    )
    .await;
    (!items.is_empty()).then_some(tower_lsp::lsp_types::CompletionResponse::Array(items))
}
