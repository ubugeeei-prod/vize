//! Index source spans already emitted as authored expressions. The interval
//! union avoids rescanning every mapping for each unresolved reference.

use crate::virtual_ts::VizeMapping;
use std::ops::Range;
use vize_croquis::{Croquis, TemplateExpressionKind};

pub(super) fn interpolations(summary: &Croquis, offset: u32) -> Vec<Range<usize>> {
    let mut ranges: Vec<_> = summary
        .template_expressions
        .iter()
        .filter(|expression| expression.kind == TemplateExpressionKind::Interpolation)
        .map(|expression| (offset + expression.start) as usize..(offset + expression.end) as usize)
        .collect();
    ranges.sort_unstable_by_key(|range| range.start);
    ranges
}

pub(super) fn collect(mappings: &[VizeMapping]) -> Vec<Range<usize>> {
    let mut spans: Vec<_> = mappings
        .iter()
        .flat_map(|mapping| {
            std::iter::once(mapping.src_range.clone())
                .chain(mapping.sub_spans.iter().map(|span| span.src_range.clone()))
        })
        .filter(|range| !range.is_empty())
        .collect();
    spans.sort_unstable_by_key(|range| (range.start, range.end));
    let mut union: Vec<Range<usize>> = Vec::with_capacity(spans.len());
    for span in spans {
        if let Some(last) = union.last_mut()
            && span.start <= last.end
        {
            last.end = last.end.max(span.end);
        } else {
            union.push(span);
        }
    }
    union
}

pub(super) fn contains(spans: &[Range<usize>], offset: usize) -> bool {
    spans
        .get(spans.partition_point(|span| span.end <= offset))
        .is_some_and(|span| span.contains(&offset))
}
