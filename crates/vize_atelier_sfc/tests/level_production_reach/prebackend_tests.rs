use std::collections::BTreeMap;

use vize_atelier_sfc::{SfcError, SfcParseOptions, parse_sfc};
use vize_l0::profiler::global_profiler;

use super::parity::parity_failures;
use super::shapes::{Shape, compile};
use super::tally::{Lane, SURFACE_SEMANTICS, Tally, classify};
use crate::Sweep;

#[test]
fn original_vapor_parser_rejections_keep_their_identity_and_complete_diagnostics() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    let cases = [
        (
            "level-html-nesting/Page.vue",
            include_str!("../../../../tests/_fixtures/level-html-nesting/Page.vue"),
            "Vapor template compilation errors: [\"Invalid end tag.\"]",
        ),
        (
            "level-ts40-projection/parse-recovery.vue",
            include_str!("../../../../tests/_fixtures/level-ts40-projection/parse-recovery.vue"),
            "Vapor template compilation errors: [\"Element is missing end tag.\"]",
        ),
    ];
    for (name, source, message) in cases {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
        let selected = compile(&descriptor, name, Shape::Vapor).unwrap();
        let counters = profiler.counter_summary();
        profiler.disable();
        profiler.clear();
        assert_eq!(classify(Shape::Vapor, &counters), Ok(Lane::Unrecorded));
        let errors = vec![SfcError {
            code: Some("VAPOR_TEMPLATE_ERROR".into()),
            message: message.into(),
            loc: Some(descriptor.template.as_ref().unwrap().loc.clone()),
        }];
        assert_eq!(
            serde_json::to_value(&selected.errors).unwrap(),
            serde_json::to_value(&errors).unwrap(),
            "{name}"
        );
        let mut actual = Tally::default();
        crate::measure(&descriptor, name, Shape::Vapor, &mut actual);
        let expected = Tally {
            sfc_errors: BTreeMap::from([("VAPOR_TEMPLATE_ERROR".to_owned(), 1)]),
            prebackend_rejections: vec![(name.to_owned(), errors)],
            ..Default::default()
        };
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"), "{name}");
    }
}

#[test]
fn authored_vapor_expression_errors_are_prebackend_diagnostics() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    let name = "invalid-expression.vue";
    let source = "<script setup>const count = 1</script><template>{{ count as }}</template>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let selected = compile(&descriptor, name, Shape::Vapor).unwrap();
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();
    assert_eq!(classify(Shape::Vapor, &counters), Ok(Lane::Unrecorded));
    assert_eq!(selected.errors.len(), 1);
    assert_eq!(
        selected.errors[0].code.as_deref(),
        Some("VAPOR_TEMPLATE_ERROR")
    );
    assert_eq!(
        selected.errors[0].message,
        "Vapor template compilation errors: [\"Error parsing JavaScript expression: invalid expression\"]"
    );

    let mut tally = Tally::default();
    crate::measure(&descriptor, name, Shape::Vapor, &mut tally);
    assert_eq!(tally.templates, 0);
    assert_eq!(tally.rejected, 0);
    assert_eq!(
        tally.sfc_errors,
        BTreeMap::from([("VAPOR_TEMPLATE_ERROR".to_owned(), 1)])
    );
    assert_eq!(tally.prebackend_rejections.len(), 1);
}

#[test]
fn recorded_surface_semantics_controls_are_not_parser_rejections() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    let cases = [
        (
            "vue-language-tools/upstream/test-workspace/tsc/#5492/main.vue",
            include_str!(
                "../../../../tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#5492/main.vue"
            ),
        ),
        (
            "vue-language-tools/upstream/test-workspace/tsc/_failed_fragment_props/main.vue",
            include_str!(
                "../../../../tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/_failed_fragment_props/main.vue"
            ),
        ),
    ];
    for (name, source) in cases {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
        let selected = compile(&descriptor, name, Shape::Vapor).unwrap();
        let counters = profiler.counter_summary();
        profiler.disable();
        profiler.clear();
        assert_eq!(
            classify(Shape::Vapor, &counters),
            Ok(Lane::Legacy(SURFACE_SEMANTICS.to_owned())),
            "{name}"
        );
        assert_eq!(selected.errors.len(), 0, "{name}");
        let mut tally = Tally::default();
        crate::measure(&descriptor, name, Shape::Vapor, &mut tally);
        assert_eq!(tally.templates, 1, "{name}");
        assert_eq!(
            tally.legacy,
            BTreeMap::from([(SURFACE_SEMANTICS.to_owned(), 1)]),
            "{name}"
        );
        assert_eq!(tally.accepted, 0, "{name}");
        assert_eq!(tally.unrecorded, 0, "{name}");
        assert_eq!(tally.sfc_errors, BTreeMap::new(), "{name}");
        assert_eq!(tally.prebackend_rejections.len(), 0, "{name}");
        assert_eq!(tally.refusals.total(), 1, "{name}");
    }
}

#[test]
fn an_unexplained_vapor_selection_still_fails_with_its_input_name() {
    let mut tally = Tally::default();
    tally.note("unexplained.vue", Lane::Unrecorded);
    let sweep = Sweep {
        files: 1,
        unreadable: 0,
        parse_errors: 0,
        without_template: 0,
        tallies: BTreeMap::from([("vapor", (Shape::Vapor, tally))]),
    };
    assert_eq!(
        parity_failures(&sweep),
        vec![
            "vapor: 1 Vapor template compiles recorded no selection counter: [\"unexplained.vue\"]"
                .to_owned()
        ]
    );
}
