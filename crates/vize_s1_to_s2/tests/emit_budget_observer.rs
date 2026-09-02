//! P2-12b groundwork: S2 DOM emission exposes a walk budget without
//! changing the shipped-compatible render output.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use davinci_harness::fixtures::{LADDER, template_block};
use vize_s0::Allocator;
use vize_s1_to_s2::{emit_dom_source, emit_dom_source_observed};

/// fixture name -> P2-12a DOM template-node visits.
const DOM_BASELINE_VISITS: [(&str, u32); 6] = [
    ("small", 11),
    ("medium", 62),
    ("large", 86),
    ("stress-deep", 134),
    ("stress-wide", 3),
    ("stress-interp", 1102),
];

#[test]
fn observed_dom_emit_keeps_output_and_walk_budget() {
    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");
        let observed_allocator = Allocator::new();
        let plain_allocator = Allocator::new();
        let observed = emit_dom_source_observed(&observed_allocator, template)
            .unwrap_or_else(|error| panic!("{} observed emit failed: {error:?}", fixture.name));
        let plain = emit_dom_source(&plain_allocator, template)
            .unwrap_or_else(|error| panic!("{} plain emit failed: {error:?}", fixture.name));
        let baseline_visits = DOM_BASELINE_VISITS
            .iter()
            .find(|(name, _)| *name == fixture.name)
            .map(|(_, visits)| *visits)
            .unwrap_or_else(|| panic!("{} has no pinned DOM baseline", fixture.name));

        assert_eq!(
            observed.emit.assembled(),
            plain.assembled(),
            "{} observed emit must not change render output",
            fixture.name
        );
        assert_eq!(
            observed.budget.transform.walks, 6,
            "{} transform walks",
            fixture.name
        );
        assert_eq!(
            observed.budget.transform.passes, 6,
            "{} transform passes",
            fixture.name
        );
        assert_eq!(
            observed.budget.transform.pipelines, 1,
            "{} transform pipelines",
            fixture.name
        );
        assert_eq!(
            observed.budget.transform.failures, 0,
            "{} transform failures",
            fixture.name
        );
        assert_eq!(observed.budget.emit_walks, 1, "{} emit walks", fixture.name);
        assert!(
            observed.budget.emit_visits > 0,
            "{} emit must visit at least one op",
            fixture.name
        );
        assert!(
            observed.budget.emit_visits <= baseline_visits,
            "{} emit visits {} exceed P2-12a baseline {}",
            fixture.name,
            observed.budget.emit_visits,
            baseline_visits
        );
        println!(
            "davinci.s2_dom.walk {} emit_walks={} emit_visits={} transform_walks={} transform_passes={} baseline_visits={}",
            fixture.name,
            observed.budget.emit_walks,
            observed.budget.emit_visits,
            observed.budget.transform.walks,
            observed.budget.transform.passes,
            baseline_visits
        );
    }
}
