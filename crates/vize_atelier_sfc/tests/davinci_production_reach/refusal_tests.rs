use vize_atelier_dom::differential::take_emit_refusal;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::Span;
use vize_l1_to_l2::{EmitError, UnsupportedReason};

use super::refusal::{DIAGNOSTICS, RefusalCensus, SELECTOR, bucket};
use super::shapes::{Shape, compile};
use super::tally::{EMIT_REFUSED, Lane, classify};

#[test]
fn census_groups_refusals_by_stable_bucket_largest_first() {
    let mut census = RefusalCensus::with_sample_limit(2);
    let gap = EmitError::unsupported_at(UnsupportedReason::HoistConstantGap, Span::new(7, 9));
    census.note("a.vue", Some(gap));
    census.note("b.vue", Some(gap));
    census.note("beyond-the-limit.vue", Some(gap));
    census.note("c.vue", Some(EmitError::Diagnostics));
    census.note("d.vue", None);

    assert_eq!(census.total(), 5);
    assert_eq!(census.count("hoist_constant_gap"), 3);
    assert_eq!(census.count(DIAGNOSTICS), 1);
    assert_eq!(census.count(SELECTOR), 1);
    let lines = census.lines("committed", "dom_inline");
    assert_eq!(
        lines.first().map(String::as_str),
        Some(
            "davinci production reach refusals: scope=committed shape=dom_inline total=5 \
             reasons=[(\"hoist_constant_gap\", 3), (\"diagnostics\", 1), (\"selector\", 1)]"
        )
    );
    assert_eq!(
        lines.get(1).map(String::as_str),
        Some(
            "davinci production reach refusal samples: scope=committed shape=dom_inline \
             reason=hoist_constant_gap files=[\"a.vue@7\", \"b.vue@7\"]"
        )
    );
    assert_eq!(lines.len(), 4);
    assert!(
        RefusalCensus::with_sample_limit(2)
            .lines("committed", "dom_inline")
            .is_empty()
    );
}

#[test]
fn every_unsupported_reason_keeps_its_own_bucket() {
    for reason in UnsupportedReason::ALL {
        let refusal = EmitError::unsupported(*reason);
        assert_eq!(bucket(Some(refusal)), (reason.code(), None));
    }
}

#[test]
fn dom_compile_notes_l2_errors_and_leaves_selector_refusals_unnoted() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    let rejected = EmitError::unsupported(UnsupportedReason::PrefixExpressionRejected);
    let cases = [
        (
            r#"<template><Widget><template #actions="v-slot:actions"><button>Run</button></template></Widget></template>"#,
            Lane::Legacy(EMIT_REFUSED.to_owned()),
            Some(rejected),
        ),
        (
            "<script setup>const a = 1, b = 2</script><template><p v-if=\"a && b\">x</p></template>",
            Lane::Legacy(EMIT_REFUSED.to_owned()),
            None,
        ),
        ("<template><p>static</p></template>", Lane::Accepted, None),
    ];
    for (source, lane, refusal) in cases {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        for shape in [Shape::DomInline, Shape::DomModule] {
            let _ = take_emit_refusal();
            let profiler = vize_l0::profiler::global_profiler();
            profiler.clear();
            profiler.enable();
            let _ = compile(&descriptor, "Gate.vue", shape);
            let counters = profiler.counter_summary();
            profiler.disable();
            profiler.clear();
            assert_eq!(
                classify(shape, &counters),
                Ok(lane.clone()),
                "{shape:?}: {source}"
            );
            assert_eq!(take_emit_refusal(), refusal, "{shape:?}: {source}");
            assert_eq!(take_emit_refusal(), None, "taken once: {shape:?}");
        }
    }
}
