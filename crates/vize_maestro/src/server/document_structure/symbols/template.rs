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
    let (symbols, _) = elements(&root.children, &block.content, index, block.loc.start);
    (!symbols.is_empty()).then_some(symbols)
}

fn elements(
    nodes: &[TemplateChildNode<'_>],
    source: &str,
    index: &LineIndex<'_>,
    base: usize,
) -> (Vec<DocumentSymbol>, usize) {
    let mut symbols = Vec::new();
    let mut end = 0;
    for node in nodes {
        end = end.max(node.loc().span.end as usize);
        let TemplateChildNode::Element(element) = node else {
            continue;
        };
        let (children, child_end) = elements(&element.children, source, index, base);
        // Legacy element loc covers its opening tag. Its original children
        // bound the content; accept only the adjacent authored closing tag.
        let content_end = (element.loc.span.end as usize).max(child_end);
        let element_end = if element.is_self_closing || vize_l0::is_void_tag(element.tag) {
            content_end
        } else {
            closing_end(source, element.tag, content_end)
        };
        end = end.max(element_end);
        let span = oxc_span::Span::new(element.loc.span.start, element_end as u32);
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
    (symbols, end)
}

pub(super) fn closing_end(source: &str, tag: &str, content_end: usize) -> usize {
    let Some(tail) = source.get(content_end..) else {
        return content_end;
    };
    let tail = tail.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let Some(close) = tail.strip_prefix("</") else {
        return content_end;
    };
    if !close
        .get(..tag.len())
        .is_some_and(|name| name.eq_ignore_ascii_case(tag))
    {
        return content_end;
    }
    let Some(after_name) = close.get(tag.len()..) else {
        return content_end;
    };
    let after_name = after_name.trim_start_matches(|c: char| c.is_ascii_whitespace());
    if after_name.starts_with('>') {
        source.len() - after_name.len() + 1
    } else {
        content_end
    }
}
