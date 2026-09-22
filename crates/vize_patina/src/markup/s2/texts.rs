//! Span index of merged text/interpolation facts.
//!
//! [`SideTable`] stores [`TextParts`] under the compound op's page-order
//! [`NodeId`]. The facade only has the op's span, and scanning the table per
//! interpolation is quadratic when a walk reruns. One index per [`S2Markup`]
//! makes the lookup a binary search.

use vize_davinci::id::NodeId;
use vize_davinci::side_table::SideTable;
use vize_s0::{Allocator, Span};
use vize_s1_to_s2::lower::TextParts;

/// One compound run, addressed by the span its parts tile.
#[derive(Clone, Copy)]
pub(super) struct TextSpan {
    start: u32,
    end: u32,
    id: NodeId,
}

pub(super) const EMPTY: &[TextSpan] = &[];

/// Page-order ids keyed by the compound span `(first.start, last.end)`.
pub(super) fn index<'a>(allocator: &'a Allocator, texts: &SideTable<TextParts>) -> &'a [TextSpan] {
    let mut entries = std::vec::Vec::with_capacity(texts.len());
    for (id, parts) in texts.iter() {
        let (Some(first), Some(last)) = (parts.parts.first(), parts.parts.last()) else {
            continue;
        };
        entries.push(TextSpan {
            start: first.span.start,
            end: last.span.end,
            id,
        });
    }
    if entries.is_empty() {
        return EMPTY;
    }
    entries.sort_unstable_by_key(|entry| (entry.start, entry.end, entry.id.index()));
    allocator.as_oxc().alloc_slice_copy(&entries)
}

pub(super) fn lookup<'a>(
    index: &[TextSpan],
    texts: &'a SideTable<TextParts>,
    span: Span,
) -> Option<&'a TextParts> {
    let found = index
        .binary_search_by_key(&(span.start, span.end), |entry| (entry.start, entry.end))
        .ok()?;
    texts.get(index[found].id)
}

#[cfg(test)]
mod tests {
    use super::super::S2Template;
    use vize_s0::Allocator;
    use vize_s1_to_s2::lower::TextParts;
    use vize_s2::op::Op;

    fn brute(
        texts: &vize_davinci::side_table::SideTable<TextParts>,
        span: vize_s0::Span,
    ) -> Option<&TextParts> {
        texts.iter().map(|(_, parts)| parts).find(|parts| {
            matches!(
                (parts.parts.first(), parts.parts.last()),
                (Some(first), Some(last))
                    if first.span.start == span.start && last.span.end == span.end
            )
        })
    }

    fn walk_interpolations<'a>(ops: &'a [Op<'a>], visit: &mut impl FnMut(vize_s0::Span)) {
        for op in ops {
            match op {
                Op::Interpolation(interpolation) => visit(interpolation.span),
                Op::Element(element) => walk_interpolations(&element.children.ops, visit),
                Op::Component(component) => walk_interpolations(&component.children.ops, visit),
                Op::Slot(slot) => walk_interpolations(&slot.fallback.ops, visit),
                Op::If(if_op) => {
                    for branch in &if_op.branches {
                        walk_interpolations(&branch.region.ops, visit);
                    }
                }
                Op::For(for_op) => walk_interpolations(&for_op.region.ops, visit),
                Op::Text(_) | Op::Comment(_) => {}
            }
        }
    }

    #[test]
    fn merged_runs_resolve_by_span_without_scanning() {
        let source = "<p>Hello {{ name }}, you have {{ count }}</p><p>{{ only }}</p>\
                      <div>{{ a }}-{{ b }}</div>";
        let allocator = Allocator::new();
        let lowered = S2Template::lower(&allocator, source);
        let markup = lowered.markup();
        let mut compounds = 0;
        let mut plain = 0;
        walk_interpolations(&markup.root.ops, &mut |span| {
            let indexed = markup.text_parts(span);
            let scanned = brute(&lowered.lowered().texts, span);
            assert_eq!(
                indexed.map(|parts| parts as *const TextParts),
                scanned.map(|parts| parts as *const TextParts),
            );
            match indexed {
                Some(parts) => {
                    assert!(parts.parts.len() >= 2);
                    compounds += 1;
                }
                None => plain += 1,
            }
        });
        assert_eq!(compounds, 2);
        assert_eq!(plain, 1);
    }
}
