//! P0-11 profile-export contract tests.
//!
//! This binary installs [`ProfilingAllocator`] as its global allocator so
//! span guards observe real allocation deltas, then proves two things:
//!
//! 1. Per-span allocation counts are exact: spans measure allocation-like
//!    calls made on their own thread, child spans are attributed to the
//!    parent's child share, and the plain/attributed `profile!` arms land in
//!    disjoint buckets.
//! 2. The serialized export satisfies the committed schema
//!    `davinci-road/plan/profile-export.schema.json`, checked by the strict
//!    shared strict JSON Schema subset validator: a schema keyword the
//!    validator does not implement is an error, never a silently skipped
//!    check.
//!
//! Only `per_span_allocation_counts_and_schema_validation` touches the global
//! profiler; per-span allocation counters are thread-local, so the pure
//! validator tests below can run concurrently without corrupting it.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::alloc::System;
use std::fs;
use std::path::PathBuf;

use davinci_test_support::schema as schema_check;
use vize_carton::profile;
use vize_carton::profiler::{
    ProfileExportBudget, ProfileExportOptions, ProfilingAllocator, SpanAttribution,
    allocation_snapshot, global_profiler,
};

#[global_allocator]
static GLOBAL: ProfilingAllocator<System> = ProfilingAllocator::new();

/// Schema location relative to the workspace root.
const SCHEMA_SUBPATH: &str = "davinci-road/plan/profile-export.schema.json";

const INNER_ATTRIBUTION: SpanAttribution = SpanAttribution::new()
    .with_stage("s1")
    .with_pass("fold_constants")
    .with_file_id(3)
    .with_block("template")
    .with_span(10, 42);

#[test]
fn per_span_allocation_counts_and_schema_validation() {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();

    let outer_buffer = profile!("davinci.p0_11.outer", {
        let inner_buffer = profile!(
            "davinci.p0_11.inner",
            attr: INNER_ATTRIBUTION,
            std::hint::black_box(Vec::<u8>::with_capacity(4096))
        );
        drop(inner_buffer);
        std::hint::black_box(Vec::<u8>::with_capacity(1024))
    });
    drop(outer_buffer);

    let export = profiler.export_report(&ProfileExportOptions {
        command: "build",
        allocation: Some(allocation_snapshot()),
        budget: ProfileExportBudget::default(),
    });
    profiler.disable();

    // The outer span wraps the inner one, so it always ranks first.
    assert_eq!(export.spans.len(), 2);

    let outer = &export.spans[0];
    assert_eq!(outer.key, "davinci.p0_11.outer");
    assert_eq!(outer.count, 1);
    assert_eq!(outer.attribution, None);
    let outer_alloc = outer.alloc.unwrap();
    assert_eq!(outer_alloc.calls, 2);
    assert_eq!(outer_alloc.bytes, 5120);
    assert_eq!(outer_alloc.self_calls, 1);
    assert_eq!(outer_alloc.self_bytes, 1024);

    let inner = &export.spans[1];
    assert_eq!(inner.key, "davinci.p0_11.inner");
    assert_eq!(inner.count, 1);
    let inner_attribution = inner.attribution.unwrap();
    assert_eq!(inner_attribution.stage, Some("s1"));
    assert_eq!(inner_attribution.pass, Some("fold_constants"));
    assert_eq!(inner_attribution.file_id, Some(3));
    assert_eq!(inner_attribution.block, Some("template"));
    let inner_span = inner_attribution.span.unwrap();
    assert_eq!(inner_span.start, 10);
    assert_eq!(inner_span.end, 42);
    let inner_alloc = inner.alloc.unwrap();
    assert_eq!(inner_alloc.calls, 1);
    assert_eq!(inner_alloc.bytes, 4096);
    assert_eq!(inner_alloc.self_calls, 1);
    assert_eq!(inner_alloc.self_bytes, 4096);

    // The plain-key space never absorbs the attributed bucket.
    assert!(profiler.get("davinci.p0_11.inner").is_none());
    assert_eq!(profiler.get("davinci.p0_11.outer").unwrap().count, 1);

    // The serialized report satisfies the committed schema.
    let json: serde_json::Value =
        serde_json::from_str(&export.to_json()).expect("export must be valid JSON");
    schema_check::validate(&load_schema(), &json, "$").expect("export must satisfy the schema");
}

#[test]
fn schema_rejects_missing_required_property() {
    let schema = load_schema();
    let mut value = minimal_valid_export();
    value
        .as_object_mut()
        .expect("export is an object")
        .remove("allocation");
    let error = schema_check::validate(&schema, &value, "$").expect_err("missing key must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$`: missing required property `allocation`"
    );
}

#[test]
fn schema_rejects_wrong_type_and_unexpected_property() {
    let schema = load_schema();

    let mut value = minimal_valid_export();
    value["spans"] = serde_json::Value::from("not-an-array");
    let error = schema_check::validate(&schema, &value, "$").expect_err("wrong type must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.spans`: expected array, found string"
    );

    let mut value = minimal_valid_export();
    value
        .as_object_mut()
        .expect("export is an object")
        .insert("extra".into(), serde_json::Value::from(1));
    let error = schema_check::validate(&schema, &value, "$").expect_err("extra key must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$`: unexpected property `extra`"
    );
}

#[test]
fn schema_rejects_const_mismatch_and_bad_span_key() {
    let schema = load_schema();

    let mut value = minimal_valid_export();
    value["schema_version"] = serde_json::Value::from(2);
    let error = schema_check::validate(&schema, &value, "$").expect_err("const must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.schema_version`: value does not equal const `1`"
    );

    let mut value = minimal_valid_export();
    value["spans"][0]["key"] = serde_json::Value::from("bad key");
    let error = schema_check::validate(&schema, &value, "$").expect_err("pattern must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.spans[0].key`: string does not match pattern `^[A-Za-z0-9._-]+$`"
    );
}

#[test]
fn validator_rejects_unimplemented_keywords() {
    let schema: serde_json::Value =
        serde_json::from_str("{\"type\": \"object\", \"maxProperties\": 3}")
            .expect("literal schema parses");
    let instance = serde_json::json!({});
    let error =
        schema_check::validate(&schema, &instance, "$").expect_err("unknown keyword must fail");
    assert_eq!(
        error.message().as_str(),
        "schema keyword `maxProperties` at `$` is not implemented by this validator"
    );
}

/// A hand-written export with one span exercising every optional field.
fn minimal_valid_export() -> serde_json::Value {
    let value = serde_json::json!({
        "schema_version": 1,
        "tool": "vize",
        "tool_version": "0.0.0-test",
        "command": "build",
        "budget": { "max_spans": 512, "max_counters": 256 },
        "truncation": { "dropped_spans": 0, "dropped_counters": 0 },
        "spans": [
            {
                "key": "davinci.p0_11.demo",
                "count": 1,
                "wall_ns": {
                    "total": 1000, "self": 1000, "min": 1000, "max": 1000,
                    "p50": 1000, "p95": 1000, "p99": 1000
                },
                "alloc": null,
                "attribution": { "stage": "s1", "span": { "start": 0, "end": 4 } }
            }
        ],
        "counters": [
            { "key": "io.read.bytes", "samples": 1, "total": 10, "min": 10, "max": 10 }
        ],
        "allocation": null
    });
    schema_check::validate(&load_schema(), &value, "$").expect("fixture must satisfy the schema");
    value
}

/// Load the committed schema by walking up to the `[workspace]` Cargo.toml.
///
/// Cargo runs test binaries with the package directory as the working
/// directory, so this resolves deterministically without configuration.
fn load_schema() -> serde_json::Value {
    let start = std::env::current_dir().expect("test binary has a working directory");
    let mut dir: &std::path::Path = &start;
    let root: PathBuf = loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let text = fs::read_to_string(&manifest).expect("manifest must be readable");
            if text.lines().any(|line| line.trim() == "[workspace]") {
                break dir.to_path_buf();
            }
        }
        dir = dir
            .parent()
            .expect("a [workspace] Cargo.toml must exist above the test binary");
    };
    let schema_path = root.join(SCHEMA_SUBPATH);
    let text = fs::read_to_string(&schema_path).expect("committed schema must be readable");
    serde_json::from_str(&text).expect("committed schema must be valid JSON")
}
