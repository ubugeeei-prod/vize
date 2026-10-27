//! Observe the real generation path and parser/allocation costs on one thread.

#![expect(clippy::unwrap_used, reason = "qualification asserts by panicking")]

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
    "/../../../../tests/_fixtures/content-mapper/diagnostic-ownership-10607-10618/Form.art.vue.txt"
));
const ITERATIONS: u64 = 32;

fn generate(source: &str, disabled: bool) -> Value {
    if disabled {
        let mut options = SfcTypeCheckOptions::new("src/Form.vue");
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
    let captured_type = arrow.replace(
        "(field) => !!field",
        "(field: typeof schema.value) => !!field",
    );
    let delete_frontier = expected.replace(
        "</template>",
        "<div>{{ delete schema.value }}</div></template>",
    );
    let cases = [
        ("art-ordinary", FORM, false, false, 0, 0),
        ("vue-ordinary", ordinary.as_str(), false, false, 0, 0),
        ("vue-pure-expect", expected.as_str(), false, true, 1, 2),
        ("vue-arrow-expect", arrow.as_str(), false, true, 1, 3),
        (
            "vue-member-unmoved",
            unsafe_read.as_str(),
            false,
            true,
            0,
            1,
        ),
        (
            "vue-type-capture-unmoved",
            captured_type.as_str(),
            false,
            true,
            1,
            3,
        ),
        (
            "vue-delete-frontier-unmoved",
            delete_frontier.as_str(),
            false,
            true,
            0,
            0,
        ),
        (
            "checker-virtual-projection-disabled",
            expected.as_str(),
            true,
            false,
            0,
            0,
        ),
    ];
    let profiler = global_profiler();
    let mut rows = Vec::new();
    for (name, source, disabled, render_flow, setup_parses, candidate_parses) in cases {
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
        let mut spans = profiler.span_snapshot().into_iter().filter(|(name, _, _)| (name.starts_with("canon.virtual_ts.root_expect.") || name.starts_with("canon.virtual_ts.input_ownership.")) || *name == "qualification.component_input.generate").map(|(name, _, m)| json!({"name": name, "calls": m.count, "nanoseconds": m.total_duration.as_nanos(), "allocationCalls": m.alloc_calls, "requestedBytes": m.alloc_bytes, "selfAllocationCalls": m.self_alloc_calls, "selfRequestedBytes": m.self_alloc_bytes})).collect::<Vec<_>>();
        spans.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        rows.push(json!({"case": name, "input": source, "iterations": ITERATIONS, "renderFlowProof": render_flow, "setupParsesPerGeneration": setup_parses, "candidateParsesPerGeneration": candidate_parses, "packet": last, "spans": spans}));
    }
    if let Some(path) = std::env::var_os("VIZE_INPUT_OWNERSHIP_PROFILE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
    // Retain every actual observation before checking the independent laws.
    for row in &rows {
        let flow = row["spans"]
            .as_array()
            .unwrap()
            .iter()
            .find(|span| span["name"] == "canon.virtual_ts.root_expect.render_flow");
        if row["renderFlowProof"].as_bool().unwrap() {
            let flow = flow.expect("one parser-owned render-flow proof per EXPECT generation");
            assert_eq!(flow["calls"], ITERATIONS);
            assert_eq!(flow["allocationCalls"], 0);
            assert_eq!(flow["requestedBytes"], 0);
        } else {
            assert!(
                flow.is_none(),
                "ordinary/disabled projection does no relocation flow walk"
            );
        }
        let retained = row["spans"]
            .as_array()
            .unwrap()
            .iter()
            .find(|span| span["name"] == "canon.virtual_ts.input_ownership.retained_context");
        if row["case"] == "checker-virtual-projection-disabled" {
            assert!(
                retained.is_none(),
                "disabled projection does no ownership walk"
            );
        } else {
            let retained = retained.expect("one retained-AST ownership walk per mapper generation");
            assert_eq!(retained["calls"], ITERATIONS);
            assert_eq!(
                retained["allocationCalls"], 0,
                "these ordinary/callback/EXPECT values require no new owned contextual records"
            );
            assert_eq!(retained["requestedBytes"], 0);
        }
        for (key, expected) in [
            (
                "canon.virtual_ts.root_expect.parse_setup",
                "setupParsesPerGeneration",
            ),
            (
                "canon.virtual_ts.root_expect.parse_candidate",
                "candidateParsesPerGeneration",
            ),
        ] {
            let actual = row["spans"]
                .as_array()
                .unwrap()
                .iter()
                .find(|span| span["name"] == key)
                .map_or(0, |span| span["calls"].as_u64().unwrap());
            assert_eq!(
                actual,
                row[expected].as_u64().unwrap() * ITERATIONS,
                "actual gated parser calls: {} {key}",
                row["case"]
            );
        }
    }
}

#[test]
fn actual_root_callback_generation_reports_roles_and_buffer_costs() {
    const PARENT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../tests/_fixtures/differential/typechecker/callback-render-position-10738/Min.vue.txt"
    ));
    let expected = PARENT.replace("  <Comp", "  <!-- @vue-expect-error -->\n  <Comp");
    let before = PARENT.replace("  <Comp", "  <div>{{ delete state.value }}</div>\n  <Comp");
    let disabled = PARENT.replace("(field: typeof state.value) => !!field", "noop");
    let marker = PARENT.replace(
        "(field: typeof state.value) => !!field",
        "'literal => callback'",
    );
    let profiler = global_profiler();
    let mut rows = Vec::new();
    for (name, source, owners) in [
        ("typed-query-before-delete", PARENT, 1),
        ("typed-query-expect", expected.as_str(), 1),
        ("typed-query-after-delete", before.as_str(), 1),
        ("no-inline-callback", disabled.as_str(), 0),
        ("no-inline-callback-marker-text", marker.as_str(), 0),
    ] {
        let baseline = generate(source, false);
        for _ in 0..8 {
            black_box(generate(source, false));
        }
        profiler.clear();
        profiler.enable();
        let mut last = Value::Null;
        for _ in 0..ITERATIONS {
            last = generate(black_box(source), false);
        }
        profiler.disable();
        assert_eq!(
            last, baseline,
            "profiling retains the complete mapper packet"
        );
        let mut spans = profiler.span_snapshot().into_iter().filter(|(name, _, _)| {
            name.starts_with("canon.virtual_ts.root_callback.")
                || name.starts_with("canon.virtual_ts.root_expect.parse_")
                || matches!(*name, "canon.virtual_ts.prop_check.name_emission" | "canon.virtual_ts.prop_check.value_emission" | "qualification.component_input.generate")
        }).map(|(name, _, metrics)| json!({"name": name, "calls": metrics.count, "nanoseconds": metrics.total_duration.as_nanos(), "allocationCalls": metrics.alloc_calls, "requestedBytes": metrics.alloc_bytes, "selfAllocationCalls": metrics.self_alloc_calls, "selfRequestedBytes": metrics.self_alloc_bytes})).collect::<Vec<_>>();
        spans.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        rows.push(json!({"case": name, "input": source, "iterations": ITERATIONS, "ownersPerGeneration": owners, "generatedBytes": last["text"].as_str().unwrap().len(), "packet": last, "spans": spans}));
    }
    if let Some(path) = std::env::var_os("VIZE_ROOT_CALLBACK_PROFILE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
    for row in rows {
        let spans = row["spans"].as_array().unwrap();
        let calls = |name| {
            spans
                .iter()
                .find(|span| span["name"] == name)
                .map_or(0, |span| span["calls"].as_u64().unwrap())
        };
        assert_eq!(
            calls("canon.virtual_ts.root_callback.owner"),
            row["ownersPerGeneration"].as_u64().unwrap() * ITERATIONS
        );
        for role in [
            "canon.virtual_ts.prop_check.name_emission",
            "canon.virtual_ts.prop_check.value_emission",
        ] {
            assert_eq!(
                calls(role),
                ITERATIONS,
                "the actual one prop, including the ordinary control, emits each role exactly once"
            );
        }
        for name in [
            "canon.virtual_ts.root_callback.input_flow",
            "canon.virtual_ts.root_callback.scope_frontier",
        ] {
            if row["ownersPerGeneration"] == 0 {
                assert_eq!(calls(name), 0, "ordinary values do no callback proof walk");
            } else {
                let span = spans
                    .iter()
                    .find(|span| span["name"] == name)
                    .expect("actual retained proof walk");
                assert_eq!(span["calls"], ITERATIONS);
                assert_eq!(span["allocationCalls"], 0);
                assert_eq!(span["requestedBytes"], 0);
            }
        }
        if row["ownersPerGeneration"] == 0 {
            let collect = spans
                .iter()
                .find(|span| span["name"] == "canon.virtual_ts.root_callback.collect")
                .expect("actual fast path");
            assert_eq!(collect["allocationCalls"], 0);
            assert_eq!(collect["requestedBytes"], 0);
        }
        if row["case"] != "typed-query-expect" {
            assert_eq!(calls("canon.virtual_ts.root_expect.parse_setup"), 0);
            assert_eq!(calls("canon.virtual_ts.root_expect.parse_candidate"), 0);
        }
    }
}
