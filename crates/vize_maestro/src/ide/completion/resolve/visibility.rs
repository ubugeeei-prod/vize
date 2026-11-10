//! Hide generated lexical bindings by declaration origin and private component
//! contract members only after the contract marker proves their ownership.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use tower_lsp::lsp_types::CompletionItem;
use vize_canon::{LspPosition, LspRange};
use vize_l0::{FxHashSet, String};

use crate::ide::{IdeContext, corsa_support, offset_to_position, position_to_offset};

const GENERATED_COMPONENT_MARKER: &str = "__vizeComponentMarker";
const GENERATED_MEMBER_PREFIX: &str = "__vize";

pub(super) fn retain_authored_bindings(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    line: u32,
    character: u32,
    items: &mut Vec<CompletionItem>,
) {
    if crate::ide::template_expression::is_at_member_access_position(&ctx.content, ctx.offset) {
        retain_authored_component_members(items);
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

/// Remove private fields only when the completion list proves that its receiver
/// carries Vize's generated component contract. Ordinary authored objects may
/// use the same prefix and must keep those members.
fn retain_authored_component_members(items: &mut Vec<CompletionItem>) {
    if items
        .iter()
        .any(|item| item.label == GENERATED_COMPONENT_MARKER)
    {
        items.retain(|item| !item.label.starts_with(GENERATED_MEMBER_PREFIX));
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{CompletionItemKind, Documentation};

    use super::*;

    fn item(label: &str, kind: CompletionItemKind, documentation: &str) -> CompletionItem {
        CompletionItem {
            label: label.into(),
            kind: Some(kind),
            documentation: Some(Documentation::String(documentation.into())),
            ..Default::default()
        }
    }

    #[test]
    fn generated_component_contract_members_are_hidden_without_rewriting_public_items() {
        let title = item("title", CompletionItemKind::PROPERTY, "authored title");
        let render = item("render", CompletionItemKind::METHOD, "authored render");
        let mut items = vec![
            title.clone(),
            item(
                GENERATED_COMPONENT_MARKER,
                CompletionItemKind::PROPERTY,
                "generated marker",
            ),
            render.clone(),
            item(
                "__vizeRawProps",
                CompletionItemKind::PROPERTY,
                "generated props",
            ),
            item(
                "__vizeSlots",
                CompletionItemKind::PROPERTY,
                "generated slots",
            ),
        ];

        retain_authored_component_members(&mut items);

        assert_eq!(items, [title, render]);
    }

    #[test]
    fn an_authored_prefixed_member_is_preserved_without_the_component_marker() {
        let authored = item(
            "__vizeAuthored",
            CompletionItemKind::PROPERTY,
            "authored private API",
        );
        let public = item("public", CompletionItemKind::PROPERTY, "public API");
        let mut items = vec![authored.clone(), public.clone()];

        retain_authored_component_members(&mut items);

        assert_eq!(items, [authored, public]);
    }
}
