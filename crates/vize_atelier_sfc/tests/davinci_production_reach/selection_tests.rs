use std::collections::BTreeMap;

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::profiler::{CounterEntry, CounterSummary, global_profiler};

use super::parity::parity_failures;
use super::shapes::{Shape, compile, explicit_vapor_source};
use super::tally::{Lane, Tally, classify_route};
use crate::Sweep;

fn vapor_counter() -> CounterSummary {
    CounterSummary {
        entries: vec![CounterEntry {
            name: "davinci.s3_vapor.accepted",
            samples: 1,
            total: 1,
            average: 1.0,
            min: 1,
            max: 1,
        }],
    }
}

fn dom_failures(lane: Lane) -> Vec<String> {
    let mut tally = Tally::default();
    tally.record(lane);
    let sweep = Sweep {
        files: 1,
        unreadable: 0,
        parse_errors: 0,
        without_template: 0,
        tallies: BTreeMap::from([("dom_inline", (Shape::DomInline, tally))]),
    };
    parity_failures(&sweep)
}

#[test]
fn source_vapor_route_requires_a_real_vapor_selection() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    for source in [
        "<script setup vapor>const x = 1</script><template>{{ x }}</template>",
        "<script vapor>export default {}</script><template>hi</template>",
    ] {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        assert!(explicit_vapor_source(&descriptor));
        for shape in [Shape::DomInline, Shape::DomModule] {
            let profiler = global_profiler();
            profiler.clear();
            profiler.enable();
            let output = compile(&descriptor, "explicit-vapor.vue", shape);
            let counters = profiler.counter_summary();
            profiler.disable();
            profiler.clear();
            output.expect("explicit Vapor source compiles");
            assert_eq!(
                classify_route(shape, &counters, true),
                Ok(Lane::RoutedVapor),
                "{shape:?}: {source}"
            );
        }
    }
    assert!(dom_failures(Lane::RoutedVapor).is_empty());
}

#[test]
fn missing_dom_selection_still_fails_for_dom_and_unproven_vapor_routes() {
    let descriptor = parse_sfc("<template>hi</template>", SfcParseOptions::default()).unwrap();
    assert!(!explicit_vapor_source(&descriptor));

    let expected =
        vec!["dom_inline: 1 DOM template compiles recorded no selection counter: []".to_owned()];
    let no_counters = CounterSummary { entries: vec![] };
    assert_eq!(
        dom_failures(classify_route(Shape::DomInline, &no_counters, false).unwrap()),
        expected
    );
    assert_eq!(
        dom_failures(classify_route(Shape::DomInline, &no_counters, true).unwrap()),
        expected
    );
    assert_eq!(
        dom_failures(classify_route(Shape::DomInline, &vapor_counter(), false).unwrap()),
        expected
    );
}

#[test]
fn script_sfc_template_error_is_accounted_before_dom_selection() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    let source = include_str!("../fixtures/production-reach/script-duplicate-class.vue");
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    for shape in [Shape::DomInline, Shape::DomModule] {
        let selected = compile(&descriptor, "ScriptDuplicateClass.vue", shape)
            .expect("normal-script SFC returns its template error in the result");
        assert_eq!(selected.errors.len(), 1, "{shape:?}");
        assert_eq!(selected.errors[0].code.as_deref(), Some("TEMPLATE_ERROR"));
        let mut tally = Tally::default();
        crate::measure(&descriptor, "ScriptDuplicateClass.vue", shape, &mut tally);
        assert_eq!(tally.sfc_errors.get("TEMPLATE_ERROR"), Some(&1));
        assert_eq!(tally.templates, 0, "{shape:?}");
        assert_eq!(tally.unrecorded, 0, "{shape:?}");
        assert!(tally.divergences.is_empty(), "{shape:?}");
    }
}
