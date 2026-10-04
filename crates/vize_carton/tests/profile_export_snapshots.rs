//! Owned inputs share the actual host exporter, preserving complete telemetry.

use core::time::Duration;

use vize_carton::profile_export::{
    ProfileExportBudget, ProfileExportOptions, export_report, export_report_from_snapshots,
};
use vize_l0::profiler::{AllocationSnapshot, CounterMetrics, Metrics, Profiler, SpanAttribution};

fn options() -> ProfileExportOptions {
    ProfileExportOptions {
        command: "analyze-sfc",
        allocation: None,
        budget: ProfileExportBudget::default(),
    }
}

fn metrics(samples: &[u64]) -> Metrics {
    let mut result = Metrics::new();
    for nanos in samples {
        result.record(Duration::from_nanos(*nanos));
    }
    result
}

fn counter(samples: &[u64]) -> CounterMetrics {
    let mut result = CounterMetrics::new();
    for sample in samples {
        result.record(*sample);
    }
    result
}

#[test]
fn owned_snapshots_preserve_every_supplied_telemetry_field() {
    // A controlled owned readback: nonzero allocation metadata must survive,
    // independently of whether this test installs an instrumented allocator.
    let mut span = metrics(&[30, 10]);
    span.alloc_calls = 7;
    span.alloc_bytes = 99;
    span.self_alloc_calls = 5;
    span.self_alloc_bytes = 75;
    let attribution = SpanAttribution::new()
        .with_stage("s2")
        .with_pass("fold")
        .with_file_id(7)
        .with_block("template")
        .with_span(5, 9);
    let export = export_report_from_snapshots(
        vec![("davinci.snapshot.span", attribution, span)],
        vec![("io.read.bytes", counter(&[7, 9]))],
        &ProfileExportOptions {
            allocation: Some(AllocationSnapshot {
                alloc_calls: 2,
                alloc_zeroed_calls: 3,
                realloc_calls: 5,
                alloc_bytes: 17,
                alloc_zeroed_bytes: 23,
                realloc_new_bytes: 41,
                dealloc_bytes: 11,
                realloc_old_bytes: 13,
                alloc_failures: 1,
                alloc_zeroed_failures: 2,
                realloc_failures: 3,
                ..AllocationSnapshot::default()
            }),
            ..options()
        },
    );
    assert_eq!(
        serde_json::to_value(&export).unwrap(),
        serde_json::json!({
            "schema_version": 1,
            "tool": "vize",
            "tool_version": env!("CARGO_PKG_VERSION"),
            "command": "analyze-sfc",
            "budget": { "max_spans": 512, "max_counters": 256 },
            "truncation": { "dropped_spans": 0, "dropped_counters": 0 },
            "spans": [{
                "key": "davinci.snapshot.span", "count": 2,
                "wall_ns": {
                    "total": 40, "self": 40, "min": 10, "max": 30,
                    "p50": 1000, "p95": 1000, "p99": 1000,
                },
                "alloc": { "calls": 7, "bytes": 99, "self_calls": 5, "self_bytes": 75 },
                "attribution": {
                    "stage": "s2", "pass": "fold", "file_id": 7,
                    "block": "template", "span": { "start": 5, "end": 9 },
                },
            }],
            "counters": [{
                "key": "io.read.bytes", "samples": 2, "total": 16, "min": 7, "max": 9,
            }],
            "allocation": {
                "calls": 10, "requested_bytes": 81, "released_bytes": 24, "failures": 6,
            },
        })
    );
    assert!(export.to_json().ends_with("\n"));
}

#[test]
fn native_and_owned_inputs_have_exact_wire_equality_after_source_disposal() {
    let profiler = Profiler::enabled();
    let template = SpanAttribution::new()
        .with_stage("s2")
        .with_block("template");
    profiler.record("davinci.snapshot.b", Duration::from_nanos(7000));
    profiler.record("davinci.snapshot.a", Duration::from_nanos(7000));
    profiler.record_attributed("davinci.snapshot.a", template, Duration::from_nanos(3000));
    profiler.record_attributed("davinci.snapshot.a", template, Duration::from_nanos(9000));
    profiler.record_counter("io.read.bytes", 7);
    profiler.record_counter("io.read.bytes", 9);
    profiler.record_counter("io.read.calls", 1);
    let native = export_report(&profiler, &options());
    let spans = profiler.span_snapshot();
    let counters = profiler.counter_snapshot();
    profiler.clear();
    profiler.disable();
    drop(profiler);

    let owned = export_report_from_snapshots(spans, counters, &options());
    assert_eq!(owned, native);
    assert_eq!(owned.to_json(), native.to_json());
    assert_eq!(
        owned
            .spans
            .iter()
            .map(|span| (span.key, span.count, span.attribution))
            .collect::<Vec<_>>(),
        vec![
            (
                "davinci.snapshot.a",
                2,
                Some(vize_carton::profile_export::ProfileExportAttribution {
                    stage: Some("s2"),
                    pass: None,
                    file_id: None,
                    block: Some("template"),
                    span: None,
                })
            ),
            ("davinci.snapshot.a", 1, None),
            ("davinci.snapshot.b", 1, None),
        ]
    );
    assert_eq!(owned.allocation, None);
    assert!(owned.spans.iter().all(|span| span.alloc.is_none()));
}

#[test]
fn owned_inputs_preserve_full_tiebreaks_and_explicit_truncation() {
    let early = SpanAttribution::new().with_stage("s1");
    let late = SpanAttribution::new().with_stage("s2");
    let export = export_report_from_snapshots(
        vec![
            ("davinci.snapshot.a", late, metrics(&[5000])),
            (
                "davinci.snapshot.b",
                SpanAttribution::EMPTY,
                metrics(&[5000]),
            ),
            ("davinci.snapshot.a", early, metrics(&[5000])),
        ],
        vec![("io.z", counter(&[2])), ("io.a", counter(&[3]))],
        &ProfileExportOptions {
            budget: ProfileExportBudget {
                max_spans: 2,
                max_counters: 1,
            },
            ..options()
        },
    );
    assert_eq!(
        serde_json::to_value(export).unwrap(),
        serde_json::json!({
            "schema_version": 1, "tool": "vize", "tool_version": env!("CARGO_PKG_VERSION"),
            "command": "analyze-sfc",
            "budget": { "max_spans": 2, "max_counters": 1 },
            "truncation": { "dropped_spans": 1, "dropped_counters": 1 },
            "spans": [
                { "key": "davinci.snapshot.a", "count": 1,
                  "wall_ns": { "total": 5000, "self": 5000, "min": 5000, "max": 5000,
                               "p50": 8000, "p95": 8000, "p99": 8000 },
                  "alloc": null, "attribution": { "stage": "s1" } },
                { "key": "davinci.snapshot.a", "count": 1,
                  "wall_ns": { "total": 5000, "self": 5000, "min": 5000, "max": 5000,
                               "p50": 8000, "p95": 8000, "p99": 8000 },
                  "alloc": null, "attribution": { "stage": "s2" } },
            ],
            "counters": [{ "key": "io.a", "samples": 1, "total": 3, "min": 3, "max": 3 }],
            "allocation": null,
        })
    );
}

#[test]
fn owned_inputs_keep_empty_unavailable_and_duration_saturation_contracts() {
    let empty = export_report_from_snapshots(Vec::new(), Vec::new(), &options());
    assert_eq!(
        serde_json::to_value(empty).unwrap(),
        serde_json::json!({
            "schema_version": 1, "tool": "vize", "tool_version": env!("CARGO_PKG_VERSION"),
            "command": "analyze-sfc",
            "budget": { "max_spans": 512, "max_counters": 256 },
            "truncation": { "dropped_spans": 0, "dropped_counters": 0 },
            "spans": [], "counters": [], "allocation": null,
        })
    );
    let mut huge = Metrics::new();
    huge.record(Duration::MAX);
    let huge = export_report_from_snapshots(
        vec![("davinci.snapshot.huge", SpanAttribution::EMPTY, huge)],
        Vec::new(),
        &options(),
    );
    assert_eq!(
        serde_json::to_value(&huge.spans).unwrap(),
        serde_json::json!([{
            "key": "davinci.snapshot.huge", "count": 1,
            "wall_ns": { "total": u64::MAX, "self": u64::MAX, "min": u64::MAX, "max": u64::MAX,
                         "p50": 140737488355328000_u64, "p95": 140737488355328000_u64,
                         "p99": 140737488355328000_u64 },
            "alloc": null,
        }])
    );
}
