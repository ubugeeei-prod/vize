//! HTML/art element trees use original, untransformed markup spans.

use tower_lsp::lsp_types::{DocumentSymbol, SymbolKind};
use vize_atelier_sfc::SfcTemplateBlock;
use vize_l0::{Allocator, line_index::LineIndex};
use vize_relief::{ElementType, TemplateChildNode};

pub(super) fn children(
    block: &SfcTemplateBlock<'_>,
    index: &LineIndex<'_>,
) -> Option<Vec<DocumentSymbol>> {
    // A preprocessor's generated markup positions are not source positions.
    // Keep the existing block until an original-span producer is available.
    if block
        .lang
        .as_deref()
        .is_some_and(|lang| !matches!(lang, "html" | "art"))
    {
        return None;
    }
    let allocator = Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, &block.content);
    if !errors.is_empty() {
        return None;
    }
    let symbols = elements(&root.children, index, block.loc.start);
    (!symbols.is_empty()).then_some(symbols)
}

fn elements(
    nodes: &[TemplateChildNode<'_>],
    index: &LineIndex<'_>,
    base: usize,
) -> Vec<DocumentSymbol> {
    nodes
        .iter()
        .filter_map(|node| {
            let TemplateChildNode::Element(element) = node else {
                return None;
            };
            let span = oxc_span::Span::new(element.loc.span.start, element.loc.span.end);
            let selection =
                oxc_span::Span::new(span.start + 1, span.start + 1 + element.tag.len() as u32);
            let kind = if element.tag_type == ElementType::Component {
                SymbolKind::CLASS
            } else {
                SymbolKind::OBJECT
            };
            Some(super::symbol(
                element.tag,
                kind,
                index,
                base,
                span,
                selection,
                elements(&element.children, index, base),
            ))
        })
        .collect()
}
