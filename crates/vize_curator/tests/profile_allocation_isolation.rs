//! Actual allocation tracking survives the duration-only host export.
//!
//! A standalone executable keeps libtest's reporting thread out of the
//! process-wide allocation window. The host chooses its real System allocator;
//! L0 only wraps that supplied allocator. No clock is read by this control.

use core::time::Duration;
use std::alloc::System;
use std::hint::black_box;

use vize_curator::inspector::{LadderStep, ladder_profile};
use vize_l0::profiler::{
    ProfilingAllocator, SpanAttribution, allocation_snapshot, global_profiler,
};

#[global_allocator]
static ALLOCATOR: ProfilingAllocator<System> = ProfilingAllocator::from_allocator(System);

const CASE: &str = "local_profile_preserves_active_native_allocation_and_counter_session";

fn main() -> Result<(), &'static str> {
    let mut args = std::env::args().skip(1);
    let (mut list, mut ignored, mut exact) = (false, false, false);
    let mut filter = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" => list = true,
            "--ignored" => ignored = true,
            "--exact" => exact = true,
            "--nocapture" => {}
            "--format" if args.next().as_deref() == Some("terse") => {}
            value if !value.starts_with('-') && filter.is_none() => filter = Some(arg),
            _ => return Err("unsupported harness argument"),
        }
    }
    let selected = filter.as_ref().is_none_or(|filter| {
        if exact {
            filter == CASE
        } else {
            CASE.contains(filter)
        }
    });
    if ignored || !selected {
        return Ok(());
    }
    if list {
        println!("{CASE}: test");
        return Ok(());
    }
    local_profile_preserves_active_native_allocation_and_counter_session();
    Ok(())
}

fn local_profile_preserves_active_native_allocation_and_counter_session() {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    profiler.record("native.host.span", Duration::from_nanos(123));
    profiler.record_counter("native.host.counter", 7);
    // A real nonempty native window exposes an accidental reset by a local
    // collector. Keep the buffer alive through both observations.
    let retained = black_box(Vec::<u8>::with_capacity(65_536));
    let before = allocation_snapshot();
    let profile = ladder_profile(
        "analyze-sfc",
        &[
            step("s1", "parse", 10),
            step("s1", "parse", 30),
            step("s2", "lower", 7),
        ],
        &[step("s2", "lower", 7)],
    );
    let after = allocation_snapshot();
    // No JSON construction, assertion or output between these snapshots:
    // exactly one genuine alloc proves the tracking gate remains enabled.
    let probe_before = allocation_snapshot();
    let probe = black_box(Vec::<u8>::with_capacity(127));
    let probe_after = allocation_snapshot();
    profiler.record_counter("native.host.counter", 11);
    let native_spans = profiler.span_snapshot();
    let native_counters = profiler.counter_snapshot();
    let enabled = profiler.is_enabled();
    profiler.disable();

    assert!(enabled);
    assert!(before.requested_bytes() >= 65_536);
    assert!(after.requested_bytes() >= before.requested_bytes());
    assert!(after.allocation_calls() >= before.allocation_calls());
    assert_eq!(probe_after.alloc_calls - probe_before.alloc_calls, 1);
    assert_eq!(probe_after.alloc_bytes - probe_before.alloc_bytes, 127);
    assert_eq!(
        probe_after.allocation_calls() - probe_before.allocation_calls(),
        1
    );
    assert_eq!(
        probe_after.requested_bytes() - probe_before.requested_bytes(),
        127
    );
    assert_eq!(retained.capacity(), 65_536);
    assert_eq!(probe.capacity(), 127);

    assert_eq!(native_spans.len(), 1);
    for (key, attribution, metrics) in native_spans {
        assert_eq!(key, "native.host.span");
        assert_eq!(attribution, SpanAttribution::EMPTY);
        assert_eq!(
            serde_json::json!({
                "count": metrics.count,
                "total": metrics.total_duration.as_nanos(),
                "self": metrics.self_duration.as_nanos(),
                "child": metrics.child_duration.as_nanos(),
                "min": metrics.min_duration.as_nanos(),
                "max": metrics.max_duration.as_nanos(),
                "self_min": metrics.min_self_duration.as_nanos(),
                "self_max": metrics.max_self_duration.as_nanos(),
                "alloc_calls": metrics.alloc_calls,
                "alloc_bytes": metrics.alloc_bytes,
                "self_alloc_calls": metrics.self_alloc_calls,
                "self_alloc_bytes": metrics.self_alloc_bytes,
                "p50": metrics.percentile(0.50).as_nanos(),
                "p95": metrics.percentile(0.95).as_nanos(),
                "p99": metrics.percentile(0.99).as_nanos(),
                "samples_over_1ms": metrics.samples_over_1ms(),
                "samples_over_10ms": metrics.samples_over_10ms(),
                "samples_over_100ms": metrics.samples_over_100ms(),
            }),
            serde_json::json!({
                "count": 1, "total": 123, "self": 123, "child": 0,
                "min": 123, "max": 123, "self_min": 123, "self_max": 123,
                "alloc_calls": 0, "alloc_bytes": 0, "self_alloc_calls": 0, "self_alloc_bytes": 0,
                "p50": 1000, "p95": 1000, "p99": 1000,
                "samples_over_1ms": 0, "samples_over_10ms": 0, "samples_over_100ms": 0,
            })
        );
    }
    assert_eq!(
        native_counters
            .into_iter()
            .map(|(key, metrics)| (
                key,
                metrics.samples,
                metrics.total,
                metrics.min,
                metrics.max
            ))
            .collect::<Vec<_>>(),
        vec![("native.host.counter", 2, 18, 7, 11)]
    );
    // Repeated identities coalesce with the original Metrics arithmetic;
    // walk/step ties retain the exporter's key ordering and unavailable fields.
    assert_eq!(
        profile,
        serde_json::json!({
            "schema_version": 1, "tool": "vize", "tool_version": env!("CARGO_PKG_VERSION"),
            "command": "analyze-sfc",
            "budget": { "max_spans": 512, "max_counters": 256 },
            "truncation": { "dropped_spans": 0, "dropped_counters": 0 },
            "spans": [
                { "key": "davinci.spolvero.step", "count": 2,
                  "wall_ns": { "total": 40, "self": 40, "min": 10, "max": 30,
                               "p50": 1000, "p95": 1000, "p99": 1000 },
                  "alloc": null, "attribution": { "stage": "s1", "pass": "parse", "block": "template" } },
                { "key": "davinci.pass.walk", "count": 1,
                  "wall_ns": { "total": 7, "self": 7, "min": 7, "max": 7,
                               "p50": 1000, "p95": 1000, "p99": 1000 },
                  "alloc": null, "attribution": { "stage": "s2", "pass": "lower", "block": "template" } },
                { "key": "davinci.spolvero.step", "count": 1,
                  "wall_ns": { "total": 7, "self": 7, "min": 7, "max": 7,
                               "p50": 1000, "p95": 1000, "p99": 1000 },
                  "alloc": null, "attribution": { "stage": "s2", "pass": "lower", "block": "template" } },
            ],
            "counters": [], "allocation": null,
        })
    );
}

fn step(stage: &'static str, pass: &'static str, nanos: u64) -> LadderStep {
    LadderStep { stage, pass, nanos }
}
