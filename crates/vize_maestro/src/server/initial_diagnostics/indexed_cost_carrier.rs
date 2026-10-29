//! Untimed, same-thread source census with a real test-only project payload.
//! No LSP server, native backend, child process, or network is started.

use super::pending::{DiagnosticsJobCause, InitialDiagnosticsJob, PendingInitialDiagnostics};
use crate::server::{diagnostic_publishing::SyncDiagnostics, state::ServerState};
use davinci_harness::alloc::{
    CountingAllocator, mark_installed, measure_returning, stats as allocation_stats,
};
use serde_json::{Value, json};
use std::alloc::System;
use std::hint::black_box;
use std::mem::size_of;
use std::time::{Duration, Instant};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};
use vize_l0::profiler::{
    ProfilingAllocator, allocation_snapshot, global_profiler, micro, reset_allocation_counters,
    set_allocation_tracking_enabled,
};

#[global_allocator]
static ALLOCATOR: CountingAllocator<ProfilingAllocator<System>> =
    CountingAllocator::from_allocator(ProfilingAllocator::from_allocator(System));

#[derive(Clone, Copy)]
struct AllocationReceipt {
    allocation_like_calls: u64,
    deallocation_calls: u64,
    reallocation_calls: u64,
    requested_bytes: u64,
    released_bytes: u64,
    live_before_bytes: u64,
    live_after_bytes: u64,
    live_delta_bytes: i64,
    peak_bytes_over_start: u64,
}

impl AllocationReceipt {
    fn json(self) -> Value {
        json!({
            "allocation_like_calls": self.allocation_like_calls,
            "deallocation_calls": self.deallocation_calls,
            "reallocation_calls": self.reallocation_calls,
            "requested_bytes": self.requested_bytes,
            "released_bytes": self.released_bytes,
            "live_before_bytes": self.live_before_bytes,
            "live_after_bytes": self.live_after_bytes,
            "live_delta_bytes": self.live_delta_bytes,
            "peak_bytes_over_start": self.peak_bytes_over_start,
        })
    }
}

fn allocation_window<T>(operation: impl FnOnce() -> T) -> (T, AllocationReceipt) {
    let before = allocation_stats();
    reset_allocation_counters();
    set_allocation_tracking_enabled(true);
    let (value, peak) = measure_returning(operation);
    let counters = allocation_snapshot();
    set_allocation_tracking_enabled(false);
    let after = allocation_stats();
    let peak = peak.expect("the test-only counting allocator is installed");
    let counted_calls = after.calls.saturating_sub(before.calls);
    let requested = counters.requested_bytes();
    let released = counters.released_bytes();
    let net = i128::from(requested) - i128::from(released);
    let live_delta = i128::from(after.live_bytes) - i128::from(before.live_bytes);

    assert_eq!(counters.allocation_failures(), 0);
    assert_eq!(peak.calls, counters.allocation_calls());
    assert_eq!(counted_calls, counters.allocation_calls());
    assert_eq!(
        live_delta, net,
        "both nested allocators observe the same layouts"
    );

    (
        value,
        AllocationReceipt {
            allocation_like_calls: counters.allocation_calls(),
            deallocation_calls: counters.dealloc_calls,
            reallocation_calls: counters.realloc_calls,
            requested_bytes: requested,
            released_bytes: released,
            live_before_bytes: before.live_bytes,
            live_after_bytes: after.live_bytes,
            live_delta_bytes: i64::try_from(live_delta).expect("test heap delta fits i64"),
            peak_bytes_over_start: peak.peak_bytes_over_start,
        },
    )
}

#[derive(Clone, Copy)]
struct PayloadRetentionEvidence {
    diagnostics: usize,
    diagnostic_json_bytes: usize,
    open_sequence: u64,
    edit_sequence: u64,
    fixture_baseline_bytes: u64,
    overlap_live_bytes: u64,
    retained_one_job_bytes: u64,
    baseline_after_drop_bytes: u64,
    retained_map_capacity: usize,
    open_insert: AllocationReceipt,
    take_open: AllocationReceipt,
    replace_with_edit: AllocationReceipt,
    stale_completion: AllocationReceipt,
    drop_open_clone: AllocationReceipt,
    take_edit: AllocationReceipt,
    drop_edit: AllocationReceipt,
    drop_queue: AllocationReceipt,
}

impl PayloadRetentionEvidence {
    fn json(self) -> Value {
        json!({
            "scope": "production-captured open continuation; production SyncDiagnostics type/current stamp with representative nonempty editor diagnostics; deterministic old-taken/new-edit scheduler overlap",
            "diagnostic_payload": {
                "items": self.diagnostics,
                "json_bytes": self.diagnostic_json_bytes,
            },
            "sequences": {"open": self.open_sequence, "edit": self.edit_sequence},
            "live_bytes": {
                "fixture_baseline": self.fixture_baseline_bytes,
                "during_old_taken_new_edit_overlap": self.overlap_live_bytes,
                "retained_with_exactly_one_edit_job": self.retained_one_job_bytes,
                "after_all_queue_payloads_and_storage_drop": self.baseline_after_drop_bytes,
                "returned_to_fixture_baseline": self.fixture_baseline_bytes == self.baseline_after_drop_bytes,
            },
            "retained_map_capacity_with_one_job": self.retained_map_capacity,
            "windows": {
                "insert_open_continuation": self.open_insert.json(),
                "take_open_clone": self.take_open.json(),
                "replace_with_edit_payload": self.replace_with_edit.json(),
                "ignore_stale_open_completion": self.stale_completion.json(),
                "drop_old_taken_clone": self.drop_open_clone.json(),
                "take_current_edit": self.take_edit.json(),
                "drop_current_edit": self.drop_edit.json(),
                "drop_empty_queue_storage": self.drop_queue.json(),
            },
            "allocation_scope": "requested/released bytes come from ProfilingAllocator; live/peak bytes come from the outer CountingAllocator over identical System layouts",
        })
    }
}

fn edit_diagnostics() -> Vec<Diagnostic> {
    vec![
        Diagnostic {
            range: Range::new(Position::new(8, 38), Position::new(8, 44)),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::Number(2345)),
            source: Some("vize/types".into()),
            message:
                "Argument of type '\"clik\"' is not assignable to parameter of type '\"click\"'."
                    .into(),
            ..Default::default()
        },
        Diagnostic {
            range: Range::new(Position::new(9, 48), Position::new(9, 51)),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::Number(2345)),
            source: Some("vize/types".into()),
            message: "Argument of type 'string' is not assignable to parameter of type 'number'."
                .into(),
            ..Default::default()
        },
    ]
}

#[expect(
    clippy::disallowed_types,
    reason = "the carrier retains the same shared project owner as production continuation payloads"
)]
fn payload_retention_evidence() -> PayloadRetentionEvidence {
    let directory = crate::test_support::tempdir().unwrap();
    std::fs::write(
        directory.path().join("vite.config.mjs"),
        "export default {vue:{}};",
    )
    .unwrap();
    let owner = std::sync::Arc::new(ServerState::new());
    owner.set_workspace_root(directory.path().to_path_buf());
    owner.load_workspace_config(directory.path());
    let uri = Url::from_file_path(directory.path().join("App.vue")).unwrap();
    owner
        .documents
        .open(uri.clone(), "<template />".into(), 1, "vue".into());
    let current = owner
        .document_project_state(&uri)
        .unwrap_or_else(|| owner.clone());
    let continuation = current
        .initial_configuration_continuation(&uri, 1)
        .expect("the actual open document captures its configuration continuation");
    assert!(continuation.owns_current_state(&current, &uri, 1));

    let diagnostics = edit_diagnostics();
    let diagnostic_json_bytes = serde_json::to_vec(&diagnostics).unwrap().len();
    let diagnostic_count = diagnostics.len();
    let edit_sync = SyncDiagnostics {
        version: 2,
        stamp: current.corsa_request_stamp(),
        diagnostics,
    };
    let now = Instant::now();
    let mut pending = PendingInitialDiagnostics::default();
    let fixture_baseline = allocation_stats().live_bytes;

    let ((), open_insert) = allocation_window(|| {
        pending.insert_with_continuation(
            uri.clone(),
            1,
            now,
            DiagnosticsJobCause::Open,
            None,
            Some(continuation.clone()),
        );
    });
    assert_eq!(pending.jobs.len(), 1);
    assert!(pending.jobs[&uri].continuation.is_some());
    assert!(pending.jobs[&uri].sync.is_none());

    let (open, take_open) = allocation_window(|| pending.take_sync().unwrap());
    let (open_uri, open_job) = open;
    assert_eq!(open_uri, uri);
    assert_eq!(open_job.cause, DiagnosticsJobCause::Open);
    assert!(open_job.sync_running);
    assert!(
        open_job
            .continuation
            .as_ref()
            .is_some_and(|value| value.owns_current_state(&current, &uri, 1))
    );
    let open_sequence = open_job.sequence;

    let ((), replace_with_edit) = allocation_window(|| {
        pending.insert_with_continuation(
            uri.clone(),
            2,
            now,
            DiagnosticsJobCause::Edit,
            Some(edit_sync.clone()),
            None,
        );
    });
    let edit_sequence = pending.jobs[&uri].sequence;
    assert_ne!(edit_sequence, open_sequence);
    assert_eq!(pending.jobs[&uri].version, 2);
    assert_eq!(pending.jobs[&uri].cause, DiagnosticsJobCause::Edit);
    assert!(pending.jobs[&uri].continuation.is_none());
    assert_eq!(
        pending.jobs[&uri]
            .sync
            .as_ref()
            .map(|value| (value.version, value.diagnostics.len())),
        Some((2, diagnostic_count))
    );
    let overlap_live_bytes = allocation_stats().live_bytes;

    let ((), stale_completion) = allocation_window(|| pending.complete_sync(&uri, open_sequence));
    assert_eq!(pending.jobs[&uri].sequence, edit_sequence);
    assert!(!pending.jobs[&uri].sync_running);

    let ((), drop_open_clone) = allocation_window(|| drop((open_uri, open_job)));
    assert_eq!(pending.jobs.len(), 1);
    let retained_one_job_bytes = allocation_stats().live_bytes;
    let retained_map_capacity = pending.jobs.capacity();
    assert!(overlap_live_bytes > retained_one_job_bytes);
    assert!(retained_one_job_bytes > fixture_baseline);

    let (edit, take_edit) = allocation_window(|| pending.take_ready(now).unwrap());
    assert!(pending.jobs.is_empty());
    assert_eq!(edit.0, uri);
    assert_eq!(edit.1.version, 2);
    assert_eq!(
        edit.1
            .sync
            .as_ref()
            .map(|value| (value.version, value.diagnostics.len())),
        Some((2, diagnostic_count))
    );
    let ((), drop_edit) = allocation_window(|| drop(edit));
    let ((), drop_queue) = allocation_window(|| drop(pending));
    let baseline_after_drop_bytes = allocation_stats().live_bytes;
    assert_eq!(baseline_after_drop_bytes, fixture_baseline);

    PayloadRetentionEvidence {
        diagnostics: diagnostic_count,
        diagnostic_json_bytes,
        open_sequence,
        edit_sequence,
        fixture_baseline_bytes: fixture_baseline,
        overlap_live_bytes,
        retained_one_job_bytes,
        baseline_after_drop_bytes,
        retained_map_capacity,
        open_insert,
        take_open,
        replace_with_edit,
        stale_completion,
        drop_open_clone,
        take_edit,
        drop_edit,
        drop_queue,
    }
}

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

#[test]
fn indexed_scheduler_cost_carrier() {
    mark_installed();
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
        pending.jobs.reset_census();
        profiler.enable();
        vize_l0::micro_span!(
            "scheduler.cost.workload",
            drain(&mut pending, now + Duration::from_secs(2))
        );
        profiler.disable();
        let counts = pending.jobs.census();
        pending.jobs.verify();
        assert!(pending.jobs.is_empty());
        let after = json!({"jobs": pending.jobs.len(), "capacity_api": pending.jobs.capacity()});
        cases.push(json!({
            "unique_uris": count, "before": before, "after": after,
            "heap_source_census": counts,
            "retained_storage": pending.jobs.storage(),
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
    let repeated_same_uri_jobs = pending.jobs.len();
    let repeated_same_uri_capacity = capacity_max;
    drop(pending);
    let payload_retention = payload_retention_evidence();
    let report = json!({
        "source_base": "b34edc288fda5f596eea16183b225cd894fd2c3a",
        "baseline_carrier": "e29fd1527bd0a9b1ded78f7d61985fde1dda9e45",
        "instrumented_source": option_env!("VIZE_SCHEDULER_COST_SOURCE"),
        "kind": "actual untimed Rust indexed selector/heap operations; not a full LSP or cold-latency benchmark",
        "schedule": "original cases drain all sync work first, then preparation-first native selection; payload carrier deterministically overlaps one taken open job with its replacing edit",
        "empty_calibration": empty, "known1024_calibration": calibration,
        "layout_bytes": {
            "job": size_of::<InitialDiagnosticsJob>(), "url": size_of::<Url>(),
            "old_map_entry_layout": size_of::<(Url, InitialDiagnosticsJob)>(),
            "pending": size_of::<PendingInitialDiagnostics>(),
        },
        "memory_limits": {
            "capacity_is": "HashMap public minimum storable-entry capacity, NOT buckets or allocated bytes",
            "pending_allocated_bytes": "exact requested/released and live-byte deltas are in payload_retention.windows",
            "peak_live_bytes": "baseline-subtracted allocator high-water marks are in payload_retention.windows",
            "rss_bytes": null,
            "source_counter_scope": "heap root peeks/key comparisons/insertions/remove lookups/removed entries/swaps are distinct operations; not old HashMap live entry filters",
            "source_operations_excluded": "actual hasher calls/bytes and internal map bucket work were not instrumented",
        },
        "cases": cases,
        "repeated_same_uri": {
            "edits": 10000,
            "jobs": repeated_same_uri_jobs,
            "capacity_max": repeated_same_uri_capacity,
        },
        "payload_retention": payload_retention.json(),
    });
    let output = std::env::var("VIZE_SCHEDULER_COST_EVIDENCE").unwrap();
    std::fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}
