//! Untimed, same-thread source census. No server, backend, filesystem project or network.

use super::cost_census::{self, Census};
use super::pending::{InitialDiagnosticsJob, PendingInitialDiagnostics};
use serde_json::{Value, json};
use std::alloc::System;
use std::hint::black_box;
use std::mem::size_of;
use std::time::{Duration, Instant};
use tower_lsp::lsp_types::Url;
use vize_l0::profiler::{ProfilingAllocator, global_profiler, micro};

#[global_allocator]
static ALLOCATOR: ProfilingAllocator<System> = ProfilingAllocator::from_allocator(System);

fn span(label: &str) -> Value {
    let entries = micro::snapshot();
    let matches: Vec<_> = entries
        .iter()
        .filter(|entry| entry.name == label && entry.kind == "span")
        .collect();
    assert_eq!(matches.len(), 1, "one exact source callsite per label");
    let site = matches[0];
    let metrics = global_profiler().get(site.key).unwrap();
    json!({
        "site": site, "calls": metrics.count,
        "allocation_like_calls": metrics.alloc_calls,
        "requested_bytes": metrics.alloc_bytes,
        "self_allocation_like_calls": metrics.self_alloc_calls,
        "self_requested_bytes": metrics.self_alloc_bytes,
        "scope": "this synchronous test thread and its synchronous callees; not RSS or child processes",
    })
}

fn fixture(count: usize, now: Instant) -> PendingInitialDiagnostics {
    let mut pending = PendingInitialDiagnostics::default();
    for index in 0..count {
        let uri = Url::parse(&format!("file:///scheduler/Component{index:05}.vue")).unwrap();
        pending.insert(uri, 1, now);
    }
    pending
}

fn drain(pending: &mut PendingInitialDiagnostics, now: Instant) {
    loop {
        let job = vize_l0::micro_span!("scheduler.cost.take_sync", pending.take_sync());
        let Some((uri, job)) = job else { break };
        pending.complete_sync(&uri, job.sequence);
        drop(black_box((uri, job)));
    }
    loop {
        let preparation = vize_l0::micro_span!(
            "scheduler.cost.take_preparation",
            pending.take_preparation()
        );
        if let Some(job) = preparation {
            drop(black_box(job));
            continue;
        }
        let deadline =
            vize_l0::micro_span!("scheduler.cost.next_not_before", pending.next_not_before());
        if deadline.is_none() {
            break;
        }
        assert!(deadline.unwrap() <= now);
        let job = vize_l0::micro_span!("scheduler.cost.take_ready", pending.take_ready(now));
        assert!(job.is_some());
        drop(black_box(job));
    }
}

fn assert_census(count: u64, rows: &[Census; 4]) {
    let triangular = count * (count + 1) / 2;
    let expected = [
        (count + 1, count * (count + 1), triangular),
        (2 * count + 1, count * count + triangular, triangular),
        (count + 1, triangular, triangular),
        (count, triangular, triangular),
    ];
    for (actual, (calls, visited, eligible)) in rows.iter().zip(expected) {
        assert_eq!(actual.calls, calls);
        assert_eq!(actual.visited_live_items, visited);
        assert_eq!(actual.eligible_live_items, eligible);
        assert_eq!(actual.selected, count);
        assert_eq!(actual.len_max as u64, count);
    }
    assert_eq!(
        rows.iter().map(|row| row.visited_live_items).sum::<u64>(),
        2 * count * count + count + 3 * triangular,
    );
}

#[test]
fn scheduler_cost_carrier() {
    let profiler = global_profiler();
    // Warm profiler/TLS state on this thread before the actual calibrators.
    profiler.enable();
    vize_l0::micro_span!("scheduler.cost.warmup", black_box(1_usize));
    profiler.disable();
    profiler.clear();
    profiler.enable();
    vize_l0::micro_span!("scheduler.cost.empty", black_box(1_usize));
    profiler.disable();
    let empty = span("scheduler.cost.empty");
    assert_eq!(empty["allocation_like_calls"], 0);
    assert_eq!(empty["requested_bytes"], 0);
    profiler.clear();
    profiler.enable();
    drop(vize_l0::micro_span!(
        "scheduler.cost.known1024",
        black_box(Vec::<u8>::with_capacity(1024))
    ));
    profiler.disable();
    let calibration = span("scheduler.cost.known1024");
    assert_eq!(calibration["allocation_like_calls"], 1);
    assert_eq!(calibration["requested_bytes"], 1024);

    let mut cases = Vec::new();
    for count in [1_usize, 10, 100, 2500, 10000] {
        profiler.clear();
        cost_census::reset();
        let now = Instant::now();
        profiler.enable();
        let mut pending = vize_l0::micro_span!("scheduler.cost.fixture", fixture(count, now));
        profiler.disable();
        let fixture_profile = span("scheduler.cost.fixture");
        let before = json!({
            "jobs": pending.jobs.len(), "capacity_api": pending.jobs.capacity(),
            "uri_utf8_bytes": pending.jobs.keys().map(|uri| uri.as_str().len()).sum::<usize>(),
            "payload": "sync=None and continuation=None: excludes real configuration/diagnostic payload clones",
        });
        profiler.enable();
        vize_l0::micro_span!(
            "scheduler.cost.workload",
            drain(&mut pending, now + Duration::from_secs(2))
        );
        profiler.disable();
        let counts = cost_census::snapshot();
        assert_census(count as u64, &counts);
        assert!(pending.jobs.is_empty());
        let after = json!({"jobs": pending.jobs.len(), "capacity_api": pending.jobs.capacity()});
        cases.push(json!({
            "unique_uris": count, "before": before, "after": after,
            "live_entry_census": counts,
            "fixture_profile": fixture_profile,
            "workload_profile": span("scheduler.cost.workload"),
            "selector_profiles": [
                span("scheduler.cost.take_sync"), span("scheduler.cost.take_preparation"),
                span("scheduler.cost.next_not_before"), span("scheduler.cost.take_ready"),
            ],
        }));
        drop(pending);
    }
    let mut pending = PendingInitialDiagnostics::default();
    let uri = Url::parse("file:///scheduler/Repeated.vue").unwrap();
    let now = Instant::now();
    let mut capacity_max = 0;
    for version in 1..=10000 {
        pending.insert(uri.clone(), version, now);
        assert_eq!(pending.jobs.len(), 1);
        capacity_max = capacity_max.max(pending.jobs.capacity());
    }
    assert_eq!(pending.jobs[&uri].version, 10000);
    let report = json!({
        "source_base": "c1dc7965dc51f91345db41a85f6cd5ca8866432c",
        "instrumented_source": option_env!("VIZE_SCHEDULER_COST_SOURCE"),
        "kind": "actual untimed Rust selector calls/live-item filters; not a full LSP or cold-latency benchmark",
        "schedule": "all sync first, then preparation-first native worker selection; no concurrent edits/importer retirements/retries",
        "empty_calibration": empty, "known1024_calibration": calibration,
        "layout_bytes": {
            "job": size_of::<InitialDiagnosticsJob>(), "url": size_of::<Url>(),
            "map_entry": size_of::<(Url, InitialDiagnosticsJob)>(),
            "pending": size_of::<PendingInitialDiagnostics>(),
        },
        "memory_limits": {
            "capacity_is": "HashMap public minimum storable-entry capacity, NOT buckets or allocated bytes",
            "pending_allocated_bytes": null, "peak_live_bytes": null, "rss_bytes": null,
            "source_operations_excluded": "internal empty/control bucket visits and actual hasher calls/bytes were not instrumented",
        },
        "cases": cases,
        "repeated_same_uri": {"edits": 10000, "jobs": pending.jobs.len(), "capacity_max": capacity_max},
    });
    let output = std::env::var("VIZE_SCHEDULER_COST_EVIDENCE").unwrap();
    std::fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}
