use vize_atelier_dom::differential::take_emit_refusal;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::Span;
use vize_l1_to_l2::{EmitError, UnsupportedReason};

use super::diff::divergence;
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
fn dom_compile_notes_l2_errors_only_when_l2_refuses() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    let rejected = EmitError::unsupported(UnsupportedReason::PrefixExpressionRejected);
    let cases = [
        (
            r#"<template><Widget><template #actions="v-slot:actions"><button>Run</button></template></Widget></template>"#,
            Lane::Legacy(EMIT_REFUSED.to_owned()),
            Some(rejected),
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

/// The SFC selector used to keep these on the compatibility lane without
/// running L2: any `&` in a template expression, and scoped styles without
/// `:slotted` on a source the fast path cannot take. Only a possible
/// character reference still needs the shipped lane, which decodes it.

#[test]
fn shared_pipeline_runs_l2_for_expression_ampersands_and_scoped_slot_outlets() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    let cases = [
        (
            "<script setup>const a = 1, b = 2</script><template><p v-if=\"a && b\">{{ a && b }}</p></template>",
            None,
        ),
        (
            "<script setup>const a = 1</script><template><div><i /><slot :a=\"a\" /><slot name=\"x\" /></div></template><style scoped>div { color: red; }</style>",
            None,
        ),
        (
            "<script setup>const a = 1, b = 2</script><template><div><br><slot v-if=\"a && b\" /></div></template><style scoped>div { color: red; }</style>",
            None,
        ),
        (
            "<script setup>const a = 1, b = 2</script><template><p :title=\"a &amp;&amp; b\">{{ a }}</p></template>",
            Some(None),
        ),
        (
            "<script setup>const a = 1</script><template><span :title=\"'&lt;b&gt;'\">{{ a &#x26;&#x26; a }}</span></template>",
            Some(None),
        ),
        (
            "<script setup>const a = 1, b = 2</script><template><p :title=\"a&&b\" @click=\"a & b\">{{ a &&b }}</p></template>",
            Some(None),
        ),
        // L2 runs before the shipped transform reports malformed slot
        // parameters, so the shared pipeline must refuse them itself.
        (
            "<script setup>const a = 1, b = 2</script><template><Widget v-if=\"a && b\"><template #actions=\"v-slot:actions\"><button>Run</button></template></Widget></template>",
            Some(Some(UnsupportedReason::PrefixExpressionRejected)),
        ),
    ];
    // `None`: L2 emits. `Some(None)`: the selector keeps the shipped lane.
    // `Some(Some(reason))`: L2 runs and refuses with `reason`.
    for (source, refused) in cases {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        for shape in [Shape::DomInline, Shape::DomModule] {
            let _ = take_emit_refusal();
            let profiler = vize_l0::profiler::global_profiler();
            profiler.clear();
            profiler.enable();
            let selected = compile(&descriptor, "Gate.vue", shape).unwrap();
            let counters = profiler.counter_summary();
            profiler.disable();
            profiler.clear();
            let lane = match refused {
                Some(_) => Lane::Legacy(EMIT_REFUSED.to_owned()),
                None => Lane::Accepted,
            };
            assert_eq!(classify(shape, &counters), Ok(lane), "{shape:?}: {source}");
            let reason = take_emit_refusal().and_then(|error| match error {
                EmitError::Unsupported(refusal) => Some(refusal.reason),
                EmitError::Diagnostics => None,
            });
            assert_eq!(reason, refused.flatten(), "{shape:?}: {source}");
            let legacy = vize_atelier_dom::differential::with_legacy_lane(|| {
                compile(&descriptor, "Gate.vue", shape)
            });
            assert_eq!(divergence(&selected, &legacy), None, "{shape:?}: {source}");
        }
    }
}
