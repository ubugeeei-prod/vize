//! Shared profile-counter pins for P2-12b S2 DOM tests.

use davinci_harness::fixtures::LADDER;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct S2DomEmitCount {
    pub emit_walks: u64,
    pub emit_visits: u64,
    pub transform_walks: u64,
}

/// fixture -> S2 DOM emit walks, emit op visits, transform walks.
///
/// Vue 3 DOM emission folds preserving fact products before codegen, leaving
/// no pass-manager transform walks in the profiled source-map-free build path.
/// Vue 2 legacy sugar still reports through the pass manager when selected.
const S2_DOM_EMIT_COUNTS: [(&str, S2DomEmitCount); 6] = [
    (
        "small",
        S2DomEmitCount {
            emit_walks: 1,
            emit_visits: 5,
            transform_walks: 0,
        },
    ),
    (
        "medium",
        S2DomEmitCount {
            emit_walks: 1,
            emit_visits: 33,
            transform_walks: 0,
        },
    ),
    (
        "large",
        S2DomEmitCount {
            emit_walks: 1,
            emit_visits: 54,
            transform_walks: 0,
        },
    ),
    (
        "stress-deep",
        S2DomEmitCount {
            emit_walks: 1,
            emit_visits: 72,
            transform_walks: 0,
        },
    ),
    (
        "stress-wide",
        S2DomEmitCount {
            emit_walks: 1,
            emit_visits: 2,
            transform_walks: 0,
        },
    ),
    (
        "stress-interp",
        S2DomEmitCount {
            emit_walks: 1,
            emit_visits: 201,
            transform_walks: 0,
        },
    ),
];

pub fn assert_s2_dom_emit_counts_cover_ladder() {
    let pinned: Vec<&str> = S2_DOM_EMIT_COUNTS.iter().map(|(name, _)| *name).collect();
    let ladder: Vec<&str> = LADDER.iter().map(|fixture| fixture.name).collect();
    assert_eq!(
        pinned, ladder,
        "S2 DOM profile pins must match the ladder exactly, in order"
    );
}

pub fn s2_dom_emit_count(fixture: &str) -> S2DomEmitCount {
    S2_DOM_EMIT_COUNTS
        .iter()
        .find(|(name, _)| *name == fixture)
        .map(|(_, count)| *count)
        .unwrap_or_else(|| panic!("{fixture} has no pinned S2 DOM emit count"))
}
