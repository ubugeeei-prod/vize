//! Protocol-valid normalization for overlapping authored projections.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use super::{ContentMapperSpanKind, SpanCandidate, ranges_overlap};

pub(super) fn normalize(candidates: Vec<SpanCandidate>) -> Vec<SpanCandidate> {
    // A split verbatim piece takes its generated length from its original. When
    // both lengths agree, as every projected verbatim span's do, pieces stay
    // inside their candidate and accepted ranges stay disjoint.
    if candidates.iter().all(|candidate| {
        candidate.kind != ContentMapperSpanKind::Verbatim
            || candidate.generated.len() == candidate.original.len()
    }) {
        normalize_with::<DisjointIndex>(candidates)
    } else {
        normalize_with::<LengthClassIndex>(candidates)
    }
}

trait RangeIndex: Default {
    fn insert(&mut self, range: &Range<usize>);
    fn overlaps(&self, probe: &Range<usize>) -> bool;
    /// Append every indexed range overlapping `probe`, each once, unordered.
    fn collect_overlapping(&self, probe: &Range<usize>, out: &mut Vec<Range<usize>>);
}

fn normalize_with<Index: RangeIndex>(mut candidates: Vec<SpanCandidate>) -> Vec<SpanCandidate> {
    candidates.sort_by_key(|candidate| {
        (
            candidate.generated.len(),
            candidate.original.len(),
            candidate.generated.start,
        )
    });

    let mut accepted: Vec<SpanCandidate> = Vec::with_capacity(candidates.len());
    let mut generated = Index::default();
    let mut original = Index::default();
    let mut overlaps = Vec::new();
    for candidate in candidates {
        if generated.overlaps(&candidate.generated) {
            continue;
        }
        overlaps.clear();
        original.collect_overlapping(&candidate.original, &mut overlaps);
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

/// Accepted ranges, which are pairwise non-overlapping or equal: a generated
/// range is accepted only when it overlaps none, and an original only when every
/// range it overlaps is the same range (split pieces are disjoint). Distinct
/// non-empty ranges are therefore disjoint, so their ends ascend with their
/// starts and every overlap of a probe is one contiguous run ending at the last
/// start before the probe's end. An empty range `p..p` still overlaps a probe
/// that strictly contains `p` under [`ranges_overlap`], so empties are kept
/// apart by position.
#[derive(Default)]
struct DisjointIndex {
    ends: BTreeMap<usize, usize>,
    empties: BTreeSet<usize>,
}

impl RangeIndex for DisjointIndex {
    fn insert(&mut self, range: &Range<usize>) {
        if range.is_empty() {
            self.empties.insert(range.start);
        } else {
            self.ends.insert(range.start, range.end);
        }
    }

    fn overlaps(&self, probe: &Range<usize>) -> bool {
        self.ends
            .range(..probe.end)
            .next_back()
            .is_some_and(|(_, &end)| end > probe.start)
            || self.empty_overlap(probe).next().is_some()
    }

    fn collect_overlapping(&self, probe: &Range<usize>, out: &mut Vec<Range<usize>>) {
        for (&start, &end) in self.ends.range(..probe.end).rev() {
            if end <= probe.start {
                break;
            }
            out.push(start..end);
        }
        out.extend(self.empty_overlap(probe).map(|&at| at..at));
    }
}

impl DisjointIndex {
    fn empty_overlap<'a>(&'a self, probe: &Range<usize>) -> impl Iterator<Item = &'a usize> {
        let low = probe.start.saturating_add(1);
        (low < probe.end)
            .then(|| self.empties.range(low..probe.end))
            .into_iter()
            .flatten()
    }
}

/// Accepted ranges bucketed by length class, for inputs whose split pieces may
/// overlap. A probe scans only the starts each class could reach from.
#[derive(Default)]
struct LengthClassIndex {
    classes: Vec<BTreeSet<(usize, usize)>>,
}

impl LengthClassIndex {
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

impl RangeIndex for LengthClassIndex {
    fn insert(&mut self, range: &Range<usize>) {
        let class = range.len().bit_width() as usize;
        if self.classes.len() <= class {
            self.classes.resize_with(class + 1, BTreeSet::new);
        }
        if let Some(ranges) = self.classes.get_mut(class) {
            ranges.insert((range.start, range.end));
        }
    }

    fn overlaps(&self, probe: &Range<usize>) -> bool {
        self.overlapping(probe).next().is_some()
    }

    fn collect_overlapping(&self, probe: &Range<usize>, out: &mut Vec<Range<usize>>) {
        out.extend(self.overlapping(probe));
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
        oracle_rounds(false);
        oracle_rounds(true);
    }

    // `projected` keeps verbatim spans equal in length, as projections do.
    fn oracle_rounds(projected: bool) {
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
        for round in 0..2000 {
            let spread = [16, 64, 400, 5000][round % 4];
            let candidates = (0..next(160) + 1)
                .map(|_| {
                    let generated = next(spread as u64);
                    let original = next(spread as u64 / 2 + 1);
                    let scale = [3, 12, 90][next(3)];
                    // Empty ranges overlap a probe that strictly contains them.
                    let length = if next(9) == 0 { 0 } else { 1 + next(scale) };
                    let kind = kinds[next(3)];
                    let original_length = match next(9) {
                        _ if projected && kind == ContentMapperSpanKind::Verbatim => length,
                        0 => 0,
                        1 | 2 => 1 + next(40),
                        _ => length,
                    };
                    SpanCandidate {
                        generated: generated..generated + length,
                        original: original..original + original_length,
                        kind,
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
                "round {round}, projected {projected}"
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
