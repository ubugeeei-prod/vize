//! Join verbatim spans across whitespace that the generated and authored text share.
//!
//! Copied setup lines map one span per line, and every newline between them
//! becomes an unmapped-gap ignore. When both sides hold the same whitespace,
//! one span maps the whole run at the same offsets, so no position moves.
//! A span whose original another span also maps, or a gap that another span
//! maps into, stays split: originals must remain disjoint or equal.

use super::super::{ContentMapperSpan, ContentMapperSpanKind};

pub(in super::super) fn join_verbatim_runs(
    source: &str,
    generated: &str,
    spans: &mut Vec<ContentMapperSpan>,
) {
    let mut originals: Vec<(usize, usize)> =
        spans.iter().map(|span| (span.0[2], span.0[3])).collect();
    originals.sort_unstable();
    let shared = |start: usize, len: usize| {
        let at = originals.partition_point(|&original| original < (start, len));
        originals.get(at + 1) == Some(&(start, len))
    };
    let mapped_inside = |start: usize, end: usize| {
        let at = originals.partition_point(|&(original, _)| original < start);
        originals
            .get(at)
            .is_some_and(|&(original, _)| original < end)
    };
    let mut joined: Vec<ContentMapperSpan> = Vec::with_capacity(spans.len());
    for span in spans.drain(..) {
        if let Some(previous) = joined.last_mut()
            && joinable(source, generated, previous, &span, &shared, &mapped_inside)
        {
            let end = span.0[0] + span.0[1];
            previous.0[1] = end - previous.0[0];
            previous.0[3] = previous.0[1];
            continue;
        }
        joined.push(span);
    }
    *spans = joined;
}

fn joinable(
    source: &str,
    generated: &str,
    previous: &ContentMapperSpan,
    next: &ContentMapperSpan,
    shared: &impl Fn(usize, usize) -> bool,
    mapped_inside: &impl Fn(usize, usize) -> bool,
) -> bool {
    let [
        left_at,
        left_len,
        left_original,
        left_original_len,
        left_kind,
        left_features,
    ] = previous.0;
    let [at, len, original, original_len, kind, features] = next.0;
    let verbatim = ContentMapperSpanKind::Verbatim as usize;
    let (gap_start, original_gap_start) = (left_at + left_len, left_original + left_original_len);
    left_kind == verbatim
        && kind == verbatim
        && left_features == features
        && left_len == left_original_len
        && len == original_len
        && at >= gap_start
        && original >= original_gap_start
        && at - gap_start == original - original_gap_start
        && !shared(original, original_len)
        && !mapped_inside(original_gap_start, original)
        && generated
            .get(gap_start..at)
            .zip(source.get(original_gap_start..original))
            .is_some_and(|(gap, original_gap)| {
                gap == original_gap && gap.bytes().all(|byte| matches!(byte, b' ' | b'\t' | b'\n'))
            })
}
