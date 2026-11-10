//! Hide generated lexical bindings by declaration origin and private component
//! contract members only after a generated contract field proves ownership.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use tower_lsp::lsp_types::CompletionItem;
use vize_canon::{LspPosition, LspRange};
use vize_l0::{FxHashSet, String};

use crate::ide::{IdeContext, corsa_support, offset_to_position, position_to_offset};

const GENERATED_COMPONENT_MARKER: &str = "__vizeComponentMarker";
const GENERATED_COMPONENT_EMIT_PROPS: &str = "__vizeEmitProps";
const GENERATED_MEMBER_PREFIX: &str = "__vize";

/// Remove generated lexical bindings and private generated component members
/// while retaining the checker's order and metadata for authored items.
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
/// carries Vize's generated component contract. Vue 3 contracts carry the
/// marker; unmarked Vue 2 emit-only contracts still carry the generated emit
/// props field. Ordinary authored objects may use other members with the same
/// prefix and must keep them.
fn retain_authored_component_members(items: &mut Vec<CompletionItem>) {
    let generated_contract = items.iter().any(|item| {
        let label = item.label.strip_suffix('?').unwrap_or(item.label.as_str());
        matches!(
            label,
            GENERATED_COMPONENT_MARKER | GENERATED_COMPONENT_EMIT_PROPS
        )
    });
    if generated_contract {
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
    /// Marker-bearing Vue 3 contracts lose every generated member without
    /// changing authored item order, kind or documentation.
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
    /// Emit-only legacy contracts use their generated emit-props field as the
    /// ownership proof when the Vue 3 marker is absent.
    fn unmarked_legacy_component_contract_members_are_hidden() {
        let render = item("render", CompletionItemKind::METHOD, "authored render");
        let mut items = vec![
            item(
                GENERATED_COMPONENT_EMIT_PROPS,
                CompletionItemKind::PROPERTY,
                "generated emits",
            ),
            item(
                "__vizeRawEmits",
                CompletionItemKind::PROPERTY,
                "generated raw emits",
            ),
            render.clone(),
        ];

        retain_authored_component_members(&mut items);

        assert_eq!(items, [render]);
    }

    #[test]
    /// A prefix alone is not ownership evidence for an ordinary authored
    /// object, so unrelated authored names remain visible.
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
