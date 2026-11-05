//! Protocol-valid normalization for overlapping authored projections.

use std::collections::BTreeSet;
use std::ops::Range;

use super::{ContentMapperSpanKind, SpanCandidate, ranges_overlap};

pub(super) fn normalize(mut candidates: Vec<SpanCandidate>) -> Vec<SpanCandidate> {
    candidates.sort_by_key(|candidate| {
        (
            candidate.generated.len(),
            candidate.original.len(),
            candidate.generated.start,
        )
    });

    let mut accepted: Vec<SpanCandidate> = Vec::new();
    let mut generated = RangeIndex::default();
    let mut original = RangeIndex::default();
    for candidate in candidates {
        if generated.overlapping(&candidate.generated).next().is_some() {
            continue;
        }
        let overlaps = original
            .overlapping(&candidate.original)
            .collect::<Vec<_>>();
        // An unsplit candidate already has its final shape. Keep it in the output
        // instead of allocating and immediately dropping a singleton Vec.
        if overlaps.is_empty() || overlaps.iter().all(|span| *span == candidate.original) {
            generated.insert(&candidate.generated);
            original.insert(&candidate.original);
            accepted.push(candidate);
            continue;
        }
        for piece in split_original_overlaps(candidate, &overlaps) {
            generated.insert(&piece.generated);
            original.insert(&piece.original);
            accepted.push(piece);
        }
    }
    accepted
}

/// Accepted ranges bucketed by length class. A probe scans only the starts
/// that a range of each class could reach from, not every accepted span.
#[derive(Default)]
struct RangeIndex {
    classes: Vec<BTreeSet<(usize, usize)>>,
}

impl RangeIndex {
    fn insert(&mut self, range: &Range<usize>) {
        let class = range.len().bit_width() as usize;
        if self.classes.len() <= class {
            self.classes.resize_with(class + 1, BTreeSet::new);
        }
        if let Some(ranges) = self.classes.get_mut(class) {
            ranges.insert((range.start, range.end));
        }
    }

    /// Every indexed range overlapping `probe`, in no particular order.
    fn overlapping<'a>(
        &'a self,
        probe: &'a Range<usize>,
    ) -> impl Iterator<Item = Range<usize>> + 'a {
        self.classes
            .iter()
            .enumerate()
            .flat_map(move |(class, ranges)| {
                let reach = 1_usize
                    .checked_shl(class as u32)
                    .map_or(usize::MAX, |bound| bound - 1);
                let low = probe.start.saturating_sub(reach);
                (low < probe.end)
                    .then(|| ranges.range((low, 0)..(probe.end, 0)))
                    .into_iter()
                    .flatten()
            })
            .map(|&(start, end)| start..end)
            .filter(|range| ranges_overlap(probe, range))
    }
}

fn split_original_overlaps(
    candidate: SpanCandidate,
    overlaps: &[Range<usize>],
) -> Vec<SpanCandidate> {
    if overlaps.is_empty() || overlaps.iter().all(|span| *span == candidate.original) {
        return vec![candidate];
    }
    if candidate.kind != ContentMapperSpanKind::Verbatim {
        return Vec::new();
    }

    let mut boundaries = vec![candidate.original.start, candidate.original.end];
    for span in overlaps {
        boundaries.push(span.start.max(candidate.original.start));
        boundaries.push(span.end.min(candidate.original.end));
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    boundaries
        .windows(2)
        .filter_map(|window| {
            let &[start, end] = window else {
                return None;
            };
            let original = start..end;
            let valid = overlaps
                .iter()
                .all(|span| !ranges_overlap(&original, span) || *span == original);
            valid.then(|| {
                let offset = original.start - candidate.original.start;
                SpanCandidate {
                    generated: candidate.generated.start + offset
                        ..candidate.generated.start + offset + original.len(),
                    original,
                    kind: ContentMapperSpanKind::Verbatim,
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The quadratic scan the index replaced, kept as the oracle.
    fn reference(mut candidates: Vec<SpanCandidate>) -> Vec<SpanCandidate> {
        candidates.sort_by_key(|candidate| {
            (
                candidate.generated.len(),
                candidate.original.len(),
                candidate.generated.start,
            )
        });
        let mut accepted: Vec<SpanCandidate> = Vec::new();
        for candidate in candidates {
            if accepted
                .iter()
                .any(|span| ranges_overlap(&candidate.generated, &span.generated))
            {
                continue;
            }
            let overlaps = accepted
                .iter()
                .filter(|span| ranges_overlap(&candidate.original, &span.original))
                .map(|span| span.original.clone())
                .collect::<Vec<_>>();
            accepted.extend(split_original_overlaps(candidate, &overlaps));
        }
        accepted
    }

    #[test]
    fn non_splitting_spans_preserve_duplicate_originals_and_split_neighbors() {
        let span = |generated, original, kind| SpanCandidate {
            generated,
            original,
            kind,
        };
        let spans = normalize(vec![
            span(50..60, 8..18, ContentMapperSpanKind::Verbatim),
            span(40..45, 12..17, ContentMapperSpanKind::Atom),
            span(20..24, 30..34, ContentMapperSpanKind::Verbatim),
            span(0..4, 100..104, ContentMapperSpanKind::Verbatim),
            span(10..13, 10..13, ContentMapperSpanKind::Alias),
            span(0..3, 10..13, ContentMapperSpanKind::Atom),
        ]);
        assert_eq!(
            spans
                .into_iter()
                .map(|span| (span.generated, span.original, span.kind))
                .collect::<Vec<_>>(),
            vec![
                (0..3, 10..13, ContentMapperSpanKind::Atom),
                (10..13, 10..13, ContentMapperSpanKind::Alias),
                (20..24, 30..34, ContentMapperSpanKind::Verbatim),
                (50..52, 8..10, ContentMapperSpanKind::Verbatim),
                (52..55, 10..13, ContentMapperSpanKind::Verbatim),
                (55..60, 13..18, ContentMapperSpanKind::Verbatim),
            ],
        );
    }

    #[test]
    fn indexed_normalization_matches_the_quadratic_scan() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % bound) as usize
        };
        let kinds = [
            ContentMapperSpanKind::Verbatim,
            ContentMapperSpanKind::Atom,
            ContentMapperSpanKind::Alias,
        ];
        for round in 0..400 {
            let spread = [16, 64, 400, 5000][round % 4];
            let candidates = (0..next(160) + 1)
                .map(|_| {
                    let generated = next(spread as u64);
                    let original = next(spread as u64 / 2 + 1);
                    let scale = [3, 12, 90][next(3)];
                    let length = 1 + next(scale);
                    let original_length = if next(4) == 0 { 1 + next(40) } else { length };
                    SpanCandidate {
                        generated: generated..generated + length,
                        original: original..original + original_length,
                        kind: kinds[next(3)],
                    }
                })
                .collect::<Vec<_>>();
            let key = |spans: Vec<SpanCandidate>| {
                spans
                    .into_iter()
                    .map(|span| (span.generated, span.original, span.kind))
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                key(normalize(candidates.clone())),
                key(reference(candidates)),
                "round {round}"
            );
        }
    }

    #[test]
    fn splits_a_broad_verbatim_span_around_a_duplicate_symbol_projection() {
        let accepted = SpanCandidate {
            generated: 100..105,
            original: 10..15,
            kind: ContentMapperSpanKind::Verbatim,
        };
        let broad = SpanCandidate {
            generated: 200..225,
            original: 0..25,
            kind: ContentMapperSpanKind::Verbatim,
        };

        let normalized = normalize(vec![broad, accepted]);
        assert_eq!(normalized.len(), 4);
        assert_eq!(
            normalized
                .iter()
                .filter(|span| span.original == (10..15))
                .count(),
            2
        );
        assert!(normalized.iter().all(|left| normalized.iter().all(|right| {
            left.original == right.original || !ranges_overlap(&left.original, &right.original)
        })));
    }
}
