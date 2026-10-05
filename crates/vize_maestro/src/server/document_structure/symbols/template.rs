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
    let symbols = elements(&root.children, &block.content, index, block.loc.start);
    (!symbols.is_empty()).then_some(symbols)
}

fn elements(
    nodes: &[TemplateChildNode<'_>],
    source: &str,
    index: &LineIndex<'_>,
    base: usize,
) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for node in nodes {
        let TemplateChildNode::Element(element) = node else {
            continue;
        };
        let children = elements(&element.children, source, index, base);
        let span = oxc_span::Span::new(element.loc.span.start, element.loc.span.end);
        let selection =
            oxc_span::Span::new(span.start + 1, span.start + 1 + element.tag.len() as u32);
        // HTML tree repair can insert an implicit tbody/tr. It is not an
        // authored symbol: retain its real descendants without a fake tag span.
        if span.end as usize > source.len()
            || selection.end > span.end
            || source.as_bytes().get(span.start as usize) != Some(&b'<')
            || source.get(selection.start as usize..selection.end as usize) != Some(element.tag)
        {
            symbols.extend(children);
            continue;
        }
        let kind = if element.tag_type == ElementType::Component {
            SymbolKind::CLASS
        } else {
            SymbolKind::OBJECT
        };
        symbols.push(super::symbol(
            element.tag,
            kind,
            index,
            base,
            span,
            selection,
            children,
        ));
    }
    symbols
}
