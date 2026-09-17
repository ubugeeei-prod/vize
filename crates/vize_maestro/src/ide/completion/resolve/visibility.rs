//! Hide generated lexical bindings by declaration origin, never name prefixes.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use tower_lsp::lsp_types::CompletionItem;
use vize_canon::{LspPosition, LspRange};
use vize_s0::{FxHashSet, String};

use crate::ide::{IdeContext, corsa_support, offset_to_position, position_to_offset};

pub(super) fn retain_authored_bindings(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    line: u32,
    character: u32,
    items: &mut Vec<CompletionItem>,
) {
    // Member names are properties of authored types, not lexical helpers.
    if crate::ide::template_expression::is_at_member_access_position(&ctx.content, ctx.offset) {
        return;
    }
    let code = &document.virtual_result.code;
    let Some(offset) = position_to_offset(code, line, character) else {
        return;
    };
    let allocator = Allocator::default();
    let mut parsed = Parser::new(&allocator, code, SourceType::ts()).parse();
    if !parsed.diagnostics.is_empty() {
        let tsx = Parser::new(&allocator, code, SourceType::tsx()).parse();
        if !tsx.panicked && tsx.diagnostics.len() < parsed.diagnostics.len() {
            parsed = tsx;
        }
    }
    if parsed.panicked {
        return;
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let scoping = semantic.scoping();
    let scope = semantic
        .nodes()
        .iter()
        .filter(|node| {
            let span = node.kind().span();
            span.start as usize <= offset && offset <= span.end as usize
        })
        .min_by_key(|node| node.kind().span().size())
        .map_or(scoping.root_scope_id(), |node| node.scope_id());
    let mut seen = FxHashSet::default();
    let mut generated = FxHashSet::<String>::default();
    for scope in scoping.scope_ancestors(scope) {
        for (name, symbol) in scoping.get_bindings(scope) {
            if !seen.insert(name.as_str()) {
                continue;
            }
            let span = scoping.symbol_span(*symbol);
            let (line, character) = offset_to_position(code, span.start as usize);
            let (end_line, end_character) = offset_to_position(code, span.end as usize);
            let range = LspRange {
                start: LspPosition { line, character },
                end: LspPosition {
                    line: end_line,
                    character: end_character,
                },
            };
            let authored = corsa_support::map_canonical_lsp_range(ctx, document, &range)
                .and_then(|range| {
                    let start =
                        position_to_offset(&ctx.content, range.start.line, range.start.character)?;
                    let end =
                        position_to_offset(&ctx.content, range.end.line, range.end.character)?;
                    ctx.content.get(start..end)
                })
                .is_some_and(|text| Some(text) == code.get(span.start as usize..span.end as usize));
            if !authored {
                generated.insert(String::from(name.as_str()));
            }
        }
    }
    items.retain(|item| !generated.contains(item.label.as_str()));
}
