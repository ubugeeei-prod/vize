//! P1-7 legacy re-parse floor: the SSR lane of the expr-reparse baseline.
//!
//! One fused compile per ladder fixture, counting the surviving legacy oxc
//! expression re-parses via the P0-3 probe — the same sweep that produced
//! `davinci-road/plan/expr-reparse-baseline.md`, pinned exactly at the
//! post-P1-7 floor (the pre-P1-7 counts are the baseline doc's table). Any
//! change means a retained-consumption gate or a legacy site moved:
//! re-derive the floor deliberately and update the baseline doc's post-P1-7
//! section with it.
//!
//! The probe is process-global and monotone, so this file holds a single
//! `#[test]` (its own binary — no concurrent recorder), mirroring
//! `vize_armature/tests/davinci_expr_parses.rs`. Integration binaries
//! compile the library without `cfg(test)`, so the P1-7 differential
//! comparators are disarmed here and the deltas are the production floor.

use davinci_harness::fixtures::{LADDER, template_block};
use vize_atelier_core::expr_parse_probe;
use vize_atelier_ssr::compile_ssr;
use vize_s0::Allocator;

/// fixture name → surviving legacy re-parses per fused SSR compile.
const FLOOR: [(&str, u64); 6] = [
    ("small", 0),
    ("medium", 0),
    ("large", 27),
    ("stress-deep", 0),
    ("stress-wide", 0),
    ("stress-interp", 0),
];

#[test]
fn ssr_legacy_reparse_floor_holds() {
    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");
        let allocator = Allocator::new();
        let before = expr_parse_probe::expr_parse_count();
        let _compiled = compile_ssr(&allocator, template);
        let parses = expr_parse_probe::expr_parse_count() - before;
        let floor = FLOOR
            .iter()
            .find(|(name, _)| *name == fixture.name)
            .unwrap_or_else(|| panic!("ladder fixture {} has no pinned floor", fixture.name))
            .1;
        assert_eq!(
            parses, floor,
            "ssr {}: surviving legacy re-parses moved from the pinned P1-7 floor",
            fixture.name
        );
    }
}
