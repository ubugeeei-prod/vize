//! Protocol-valid normalization for overlapping authored projections.

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
    for candidate in candidates {
        if accepted
            .iter()
            .any(|span| ranges_overlap(&candidate.generated, &span.generated))
        {
            continue;
        }
        accepted.extend(split_original_overlaps(candidate, &accepted));
    }
    accepted
}

fn split_original_overlaps(
    candidate: SpanCandidate,
    accepted: &[SpanCandidate],
) -> Vec<SpanCandidate> {
    let overlaps = accepted
        .iter()
        .filter(|span| ranges_overlap(&candidate.original, &span.original))
        .collect::<Vec<_>>();
    if overlaps.is_empty()
        || overlaps
            .iter()
            .all(|span| span.original == candidate.original)
    {
        return vec![candidate];
    }
    if candidate.kind != ContentMapperSpanKind::Verbatim {
        return Vec::new();
    }

    let mut boundaries = vec![candidate.original.start, candidate.original.end];
    for span in &overlaps {
        boundaries.push(span.original.start.max(candidate.original.start));
        boundaries.push(span.original.end.min(candidate.original.end));
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    boundaries
        .windows(2)
        .filter_map(|window| {
            let original = window[0]..window[1];
            let valid = overlaps.iter().all(|span| {
                !ranges_overlap(&original, &span.original) || span.original == original
            });
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
