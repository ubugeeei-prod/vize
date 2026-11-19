use alloc::vec::Vec;

use vize_l0::{Span, String};

use super::SpanLink;
use crate::write::rewrite_spans::rewritten_identifier_spans;

/// Links for an emitted expression authored at `span`: the whole expression
/// plus every identifier the context rewrite scoped (named, see
/// [`rewritten_identifier_spans`]). When an identifier starts the expression,
/// the whole-expression link keeps its range but yields its segment to the
/// identifier. An expression without an authored span (the empty stub
/// location of a synthesized node) has no links.
pub fn expression_links(code: &str, span: Span, source_text: &str) -> Vec<SpanLink> {
    if span.start >= span.end {
        return Vec::new();
    }
    let authored = source_text.get(span.start as usize..span.end as usize);
    let identifiers = authored
        .and_then(|authored| rewritten_identifier_spans(authored, code))
        .unwrap_or_default();
    let covers_start = identifiers
        .first()
        .is_some_and(|identifier| identifier.emitted == 0 && identifier.authored == 0);
    let whole = SpanLink {
        generated: Span::new(0, code.len() as u32),
        authored: span,
        name: None,
        segment: !covers_start,
    };
    let named = identifiers.into_iter().map(|identifier| {
        let start = span.start + identifier.authored as u32;
        let emitted = identifier.emitted as u32;
        SpanLink {
            generated: Span::new(emitted, emitted + identifier.emitted_len() as u32),
            authored: Span::new(start, start + identifier.name.len() as u32),
            name: Some(String::new(identifier.name)),
            segment: true,
        }
    });
    core::iter::once(whole).chain(named).collect()
}
