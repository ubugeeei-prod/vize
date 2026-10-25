//! Observe the real generation path and parser/allocation costs on one thread.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "qualification asserts by panicking"
)]

use serde_json::{Value, json};
use std::{alloc::System, hint::black_box, path::Path};
use vize_canon::{
    generate_vue_content_mapper_transform,
    sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc},
};
use vize_l0::{
    profile,
    profiler::{ProfilingAllocator, global_profiler},
};

#[global_allocator]
static ALLOCATOR: ProfilingAllocator<System> = ProfilingAllocator::from_allocator(System);

const FORM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/content-mapper/diagnostic-ownership-10607-10618/Form.art.vue.txt"
));
const ITERATIONS: u64 = 32;

fn generate(source: &str, disabled: bool) -> Value {
    if disabled {
        let mut options = SfcTypeCheckOptions::new("src/Form.vue").with_virtual_ts();
        options.check_props = false;
        let result = profile!(
            "qualification.component_input.generate",
            type_check_sfc(source, &options)
        );
        let mut value = serde_json::to_value(result).unwrap();
        value.as_object_mut().unwrap().remove("analysis_time_ms");
        value
    } else {
        let result = profile!(
            "qualification.component_input.generate",
            generate_vue_content_mapper_transform(Path::new("src/Form.vue"), source)
        );
        serde_json::to_value(result.unwrap()).unwrap()
    }
}

#[test]
fn actual_mapper_generation_reports_gated_parse_and_allocation_costs() {
    let ordinary = FORM
        .replace("<art>", "<template>")
        .replace("</art>", "</template>")
        .replace("<variant name=\"0\">", "")
        .replace("<variant name=\"1\">", "")
        .replace("<variant name=\"2\">", "")
        .replace("</variant>", "");
    let expected = ordinary.replace(
        "  <GenericForm :initial-state=\"{ role: null",
        "  <!-- @vue-expect-error -->\n  <GenericForm :initial-state=\"{ role: null",
    );
    let arrow = expected.replace(":initial-state=\"{ role: null, tags: [] }\" :schema=\"schema\"", ":initial-state=\"{ role: null, tags: [] }\" :schema=\"schema\" :validate-form-field-on=\"(field) => !!field\"");
    let unsafe_read = expected.replace(
        ":initial-state=\"{ role: null, tags: [] }\"",
        ":initial-state=\"schema.value\"",
    );
    let cases = [
        ("art-ordinary", FORM, false, 0, 0),
        ("vue-ordinary", ordinary.as_str(), false, 0, 0),
        ("vue-pure-expect", expected.as_str(), false, 1, 2),
        ("vue-arrow-expect", arrow.as_str(), false, 1, 3),
        ("vue-member-unmoved", unsafe_read.as_str(), false, 0, 1),
        ("checker-props-disabled", expected.as_str(), true, 0, 0),
    ];
    let profiler = global_profiler();
    let mut rows = Vec::new();
    for (name, source, disabled, setup_parses, candidate_parses) in cases {
        let baseline = generate(source, disabled);
        for _ in 0..8 {
            black_box(generate(source, disabled));
        }
        profiler.clear();
        profiler.enable();
        let mut last = Value::Null;
        for _ in 0..ITERATIONS {
            last = generate(black_box(source), disabled);
        }
        profiler.disable();
        assert_eq!(
            last, baseline,
            "profiling must retain the complete generated packet: {name}"
        );
        for (key, expected_count) in [
            (
                "canon.virtual_ts.root_expect.parse_setup",
                setup_parses * ITERATIONS,
            ),
            (
                "canon.virtual_ts.root_expect.parse_candidate",
                candidate_parses * ITERATIONS,
            ),
        ] {
            assert_eq!(
                profiler.get(key).map_or(0, |metrics| metrics.count),
                expected_count,
                "actual gated parser calls: {name} {key}"
            );
        }
        let mut spans = profiler.span_snapshot().into_iter().filter(|(name, _, _)| name.starts_with("canon.virtual_ts.root_expect.") || *name == "qualification.component_input.generate").map(|(name, _, m)| json!({"name": name, "calls": m.count, "nanoseconds": m.total_duration.as_nanos(), "allocationCalls": m.alloc_calls, "requestedBytes": m.alloc_bytes, "selfAllocationCalls": m.self_alloc_calls, "selfRequestedBytes": m.self_alloc_bytes})).collect::<Vec<_>>();
        spans.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        rows.push(json!({"case": name, "input": source, "iterations": ITERATIONS, "setupParsesPerGeneration": setup_parses, "candidateParsesPerGeneration": candidate_parses, "packet": last, "spans": spans}));
    }
    if let Some(path) = std::env::var_os("VIZE_INPUT_OWNERSHIP_PROFILE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}
