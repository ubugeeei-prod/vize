//! P2-12a pre-S2 traversal baseline: the SSR lane.
//!
//! One fused compile per ladder fixture, diffing
//! `vize_atelier_core::walk_probe` around it: the template-node visits and
//! stage tree-walks today's still-live pipeline makes. This is the sweep that
//! produced `davinci-road/plan/walk-baseline.md` and filled
//! `budgets.toml [traversal]`; the numbers below are that record, pinned
//! exactly. Any change means a stage started or stopped walking the tree -
//! re-derive the baseline deliberately (`--nocapture` prints every row and
//! its per-stage breakdown) and move the budget entry with a reviewed PR, per
//! the ratchet rule at the top of `budgets.toml`.
//!
//! The same run also pins Davinci's side of the tie: the walks measured here
//! must equal `vize_atelier_core::legacy_plan::SSR.group_count()`, so the
//! pass-manager plan that *describes* the shipped pipeline cannot drift from
//! it silently. The plans live in `vize_davinci` and are read from the dev-dependencies:
//! a published crate cannot depend on an unpublished one.
//!
//! The probe is process-global and monotone, so this file holds a single
//! `#[test]` in its own binary - the `davinci_expr_reparse_floor.rs` shape.

use davinci_harness::fixtures::{LADDER, template_block};
use vize_atelier_core::walk_probe::{WALK_STAGES, WalkCounts};
use vize_atelier_ssr::compile_ssr;
use vize_davinci::legacy_plan;
use vize_s0::Allocator;

/// fixture name -> (stage tree-walks, template-node visits) per fused compile.
const BASELINE: [(&str, u64, u64); 6] = [
    ("small", 2, 16),
    ("medium", 2, 118),
    ("large", 2, 106),
    ("stress-deep", 2, 144),
    ("stress-wide", 2, 4),
    ("stress-interp", 2, 2002),
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

        let breakdown = WALK_STAGES
            .iter()
            .filter(|stage| {
                delta.visits[**stage as usize] != 0 || delta.walks[**stage as usize] != 0
            })
            .map(|stage| {
                format!(
                    " {}={}/{}",
                    stage.as_str(),
                    delta.walks[*stage as usize],
                    delta.visits[*stage as usize]
                )
            })
            .collect::<String>();
        println!(
            "davinci.walk ssr {} walks={} visits={}{}",
            fixture.name,
            delta.total_walks(),
            delta.total_visits(),
            breakdown
        );

        // The Davinci plan and the pipeline it describes must agree on the
        // walk count. A plan that drifts from the compiler fails here, which
        // is why the plan is declared as data rather than written as a
        // comment (`vize_davinci::legacy_plan`).
        assert_eq!(
            delta.total_walks() as usize,
            legacy_plan::SSR.group_count(),
            "ssr {}: the measured walks disagree with legacy_plan::SSR",
            fixture.name
        );

        measured.push((fixture.name, delta.total_walks(), delta.total_visits()));
    }

    let expected: Vec<(&str, u64, u64)> = LADDER
        .iter()
        .map(|fixture| {
            *BASELINE
                .iter()
                .find(|(name, _, _)| *name == fixture.name)
                .unwrap_or_else(|| panic!("ladder fixture {} has no pinned baseline", fixture.name))
        })
        .collect();

    assert_eq!(
        measured, expected,
        "ssr: the pre-S2 traversal baseline moved from the pinned P2-12a record"
    );
}
