use std::ops::Range;

use serde::Deserialize;
use vize_carton::String;

use super::{ContentMapperSpanKind, SpanCandidate, normalize, ranges_overlap};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    generated: String,
    #[serde(default)]
    legacy_domain: bool,
    candidates: Vec<Row>,
    expected: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    generated: [usize; 2],
    original: [usize; 2],
    kind: Kind,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Verbatim,
    Atom,
    Alias,
}

impl Row {
    fn candidate(&self) -> SpanCandidate {
        let [generated_start, generated_end] = self.generated;
        let [original_start, original_end] = self.original;
        SpanCandidate {
            generated: generated_start..generated_end,
            original: original_start..original_end,
            kind: match self.kind {
                Kind::Verbatim => ContentMapperSpanKind::Verbatim,
                Kind::Atom => ContentMapperSpanKind::Atom,
                Kind::Alias => ContentMapperSpanKind::Alias,
            },
        }
    }
}

fn whole(rows: &[SpanCandidate]) -> Vec<(Range<usize>, Range<usize>, ContentMapperSpanKind)> {
    rows.iter()
        .map(|row| {
            let SpanCandidate {
                generated,
                original,
                kind,
            } = row;
            (generated.clone(), original.clone(), *kind)
        })
        .collect()
}

#[test]
fn physical_source_vectors_keep_every_ordered_span_field() {
    let fixture: Fixture = serde_json::from_str(include_str!("fixtures.json")).unwrap();
    assert_eq!(fixture.schema, "vize.content-mapper-span-normalize.v1");
    assert_eq!(fixture.cases.len(), 10);
    for case in fixture.cases {
        let candidates = case
            .candidates
            .iter()
            .map(Row::candidate)
            .collect::<Vec<_>>();
        let expected = case.expected.iter().map(Row::candidate).collect::<Vec<_>>();
        if !case.legacy_domain {
            for row in candidates.iter().chain(&expected) {
                let generated = case.generated.get(row.generated.clone()).unwrap();
                let original = case.source.get(row.original.clone()).unwrap();
                assert!(!generated.is_empty() && !original.is_empty(), "{}", case.id);
                if row.kind == ContentMapperSpanKind::Verbatim {
                    assert_eq!(generated, original, "{}", case.id);
                }
            }
        }
        assert_eq!(
            whole(&legacy(candidates.clone())),
            whole(&expected),
            "{}",
            case.id
        );
        assert_eq!(
            whole(&normalize(candidates)),
            whole(&expected),
            "{}",
            case.id
        );
    }
}

#[test]
fn indexed_selection_matches_legacy_for_nested_duplicate_and_permuted_ranges() {
    let mut state = 0x7692_8341_u64;
    for round in 0..192 {
        let mut candidates = Vec::new();
        for index in 0..96 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let start = (state >> 32) as usize % 128;
            let length = 1 + (state as usize % 24);
            let original_start = (state >> 16) as usize % 128;
            let kind = match (round + index) % 3 {
                0 => ContentMapperSpanKind::Verbatim,
                1 => ContentMapperSpanKind::Atom,
                _ => ContentMapperSpanKind::Alias,
            };
            let row = SpanCandidate {
                generated: start..start + length,
                original: original_start..original_start + length,
                kind,
            };
            if index % 11 == 0 {
                candidates.push(row.clone());
            }
            candidates.push(row);
        }
        for rows in [candidates.clone(), candidates.into_iter().rev().collect()] {
            assert_eq!(
                whole(&normalize(rows.clone())),
                whole(&legacy(rows)),
                "round {round}"
            );
        }
    }
}

#[test]
fn dense_disjoint_and_repeated_original_populations_preserve_all_rows() {
    for repeated_original in [false, true] {
        let candidates = (0..1024)
            .rev()
            .map(|index| SpanCandidate {
                generated: index * 4..index * 4 + 2,
                original: if repeated_original {
                    0..2
                } else {
                    index * 4..index * 4 + 2
                },
                kind: ContentMapperSpanKind::Verbatim,
            })
            .collect::<Vec<_>>();
        let expected = legacy(candidates.clone());
        assert_eq!(expected.len(), 1024);
        assert_eq!(whole(&normalize(candidates)), whole(&expected));
    }
}

#[test]
fn unsupported_range_shapes_keep_the_entire_legacy_call() {
    for candidate in [
        SpanCandidate {
            generated: 8..8,
            original: 2..2,
            kind: ContentMapperSpanKind::Atom,
        },
        SpanCandidate {
            generated: Range { start: 8, end: 6 },
            original: 2..4,
            kind: ContentMapperSpanKind::Alias,
        },
        SpanCandidate {
            generated: 8..12,
            original: 2..9,
            kind: ContentMapperSpanKind::Verbatim,
        },
    ] {
        let rows = vec![
            SpanCandidate {
                generated: 30..32,
                original: 4..6,
                kind: ContentMapperSpanKind::Verbatim,
            },
            candidate,
            SpanCandidate {
                generated: 40..60,
                original: 0..20,
                kind: ContentMapperSpanKind::Verbatim,
            },
        ];
        assert_eq!(whole(&normalize(rows.clone())), whole(&legacy(rows)));
    }
}

// Literal pre-change algorithm: the comparison oracle does not use either
// indexed helper or the refactored production fallback's splitting helper.
fn legacy(mut candidates: Vec<SpanCandidate>) -> Vec<SpanCandidate> {
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
        accepted.extend(legacy_split(candidate, &accepted));
    }
    accepted
}

fn legacy_split(candidate: SpanCandidate, accepted: &[SpanCandidate]) -> Vec<SpanCandidate> {
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
            let &[start, end] = window else {
                return None;
            };
            let original = start..end;
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
