//! P2-12a pre-S2 traversal baseline and the SSR production-selector floor.
//!
//! One fused compile per ladder fixture, diffing
//! `vize_atelier_core::walk_probe` around it: the template-node visits and
//! stage tree-walks the shipped pipeline makes. `BASELINE` is the pre-S2
//! sweep that produced `docs/davinci/plan/walk-baseline.md` and filled
//! `budgets.toml [traversal]`; it stays fixed as the "before" budget.
//! `PRODUCTION_FLOOR` pins the SSR selector after P3-8 routes admitted
//! templates through the S4 string plan, which never walks the legacy tree
//! for code generation. Any change means a stage started or stopped walking
//! the legacy tree - re-derive the floor deliberately (`--nocapture` prints
//! every row and its per-stage breakdown).
//!
//! The same run also pins Davinci's side of the tie: the walks measured here
//! must not exceed `vize_atelier_core::legacy_plan::SSR.group_count()`, so the
//! pass-manager plan that describes the legacy pipeline remains the upper
//! bound. The plans live in `vize_davinci` and are read from the
//! dev-dependencies: a published crate cannot depend on an unpublished one.
//!
//! The probe is process-global and monotone, so this file holds a single
//! `#[test]` in its own binary - the `davinci_expr_reparse_floor.rs` shape.

use davinci_harness::fixtures::{LADDER, template_block};
use std::fmt::Write as _;
use vize_atelier_core::walk_probe::{WALK_STAGES, WalkCounts};
use vize_atelier_ssr::compile_ssr;
use vize_davinci::legacy_plan;
use vize_s0::{Allocator, String};

/// fixture name -> (stage tree-walks, template-node visits) per fused compile.
const BASELINE: [(&str, u64, u64); 6] = [
    ("small", 2, 16),
    ("medium", 2, 118),
    ("large", 2, 106),
    ("stress-deep", 2, 144),
    ("stress-wide", 2, 4),
    ("stress-interp", 2, 2002),
];

/// fixture name -> (stage tree-walks, template-node visits) after the SSR
/// production selector emits admitted templates from the S4 string plan.
const PRODUCTION_FLOOR: [(&str, u64, u64); 6] = [
    ("small", 1, 8),
    ("medium", 1, 33),
    ("large", 1, 57),
    ("stress-deep", 1, 72),
    ("stress-wide", 1, 2),
    ("stress-interp", 1, 1001),
];

#[test]
fn ssr_walk_baseline_holds() {
    let mut measured: Vec<(&str, u64, u64)> = Vec::new();

    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");
        let allocator = Allocator::new();
        let before = WalkCounts::snapshot();
        let _compiled = compile_ssr(&allocator, template);
        let delta = WalkCounts::snapshot().since(before);

        let mut breakdown = String::default();
        for stage in WALK_STAGES.iter().filter(|stage| {
            delta.visits[**stage as usize] != 0 || delta.walks[**stage as usize] != 0
        }) {
            let _ = write!(
                &mut breakdown,
                " {}={}/{}",
                stage.as_str(),
                delta.walks[*stage as usize],
                delta.visits[*stage as usize]
            );
        }
        println!(
            "davinci.walk ssr {} walks={} visits={}{}",
            fixture.name,
            delta.total_walks(),
            delta.total_visits(),
            breakdown
        );

        // The legacy plan is the upper bound for the production selector.
        assert!(
            delta.total_walks() as usize <= legacy_plan::SSR.group_count(),
            "ssr {}: the measured walks exceed legacy_plan::SSR",
            fixture.name
        );

        measured.push((fixture.name, delta.total_walks(), delta.total_visits()));
    }

    let expected = rows_for_ladder(&PRODUCTION_FLOOR, "production floor");
    assert_eq!(
        measured, expected,
        "ssr: the S4 production traversal floor moved from its pin"
    );
    let baseline = rows_for_ladder(&BASELINE, "pre-S2 baseline");
    for ((fixture, walks, visits), (_, baseline_walks, baseline_visits)) in
        measured.iter().zip(baseline.iter())
    {
        assert!(
            walks <= baseline_walks && visits <= baseline_visits,
            "ssr {fixture}: production traversal exceeded the pre-S2 baseline"
        );
    }
}

fn rows_for_ladder(
    rows: &[(&'static str, u64, u64)],
    label: &str,
) -> Vec<(&'static str, u64, u64)> {
    LADDER
        .iter()
        .map(|fixture| {
            *rows
                .iter()
                .find(|(name, _, _)| *name == fixture.name)
                .unwrap_or_else(|| panic!("ladder fixture {} has no pinned {label}", fixture.name))
        })
        .collect()
}
