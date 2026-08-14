use super::{
    CacheStats, Metrics, ProfileExportBudget, ProfileExportOptions, Profiler, SpanAttribution,
    SpanRange, Timer,
};
use std::sync::Arc;
use std::time::Duration;

fn export_options(budget: ProfileExportBudget) -> ProfileExportOptions {
    ProfileExportOptions {
        command: "build",
        allocation: None,
        budget,
    }
}

#[test]
fn test_timer() {
    let timer = Timer::start("test");
    std::thread::sleep(Duration::from_millis(10));
    let elapsed = timer.stop();
    assert!(elapsed >= Duration::from_millis(10));
}

#[test]
fn test_profiler() {
    let profiler = Profiler::enabled();
    profiler.record("test", Duration::from_millis(10));
    profiler.record("test", Duration::from_millis(20));

    let metrics = profiler.get("test").unwrap();
    assert_eq!(metrics.count, 2);
    assert_eq!(metrics.total_duration, Duration::from_millis(30));
    assert_eq!(metrics.min_duration, Duration::from_millis(10));
    assert_eq!(metrics.max_duration, Duration::from_millis(20));
    assert_eq!(metrics.average(), Duration::from_millis(15));
}

#[test]
fn disabled_profiler_ignores_records() {
    let profiler = Profiler::new();
    profiler.record("test", Duration::from_millis(10));

    assert!(profiler.get("test").is_none());
}

#[test]
fn average_handles_counts_larger_than_u32() {
    let metrics = Metrics {
        count: u64::from(u32::MAX) + 2,
        total_duration: Duration::from_secs(10),
        min_duration: Duration::ZERO,
        max_duration: Duration::from_secs(10),
        ..Metrics::new()
    };

    assert_eq!(
        metrics.average(),
        Duration::from_nanos(
            (Duration::from_secs(10).as_nanos() / u128::from(metrics.count)) as u64
        )
    );
}

#[test]
fn metrics_track_self_child_and_tail_counts() {
    let mut metrics = Metrics::new();

    metrics.record_with_child(Duration::from_millis(10), Duration::from_millis(4));
    metrics.record_with_child(Duration::from_micros(500), Duration::from_micros(125));

    assert_eq!(metrics.count, 2);
    assert_eq!(metrics.self_duration, Duration::from_micros(6_375));
    assert_eq!(metrics.child_duration, Duration::from_micros(4_125));
    assert_eq!(metrics.self_average(), Duration::from_nanos(3_187_500));
    assert_eq!(metrics.samples_over_1ms(), 1);
    assert_eq!(metrics.samples_over_10ms(), 1);
    assert_eq!(metrics.samples_over_100ms(), 0);
    assert!(metrics.percentile(0.95) >= Duration::from_millis(10));
}

#[test]
fn profiler_tracks_counters() {
    let profiler = Profiler::enabled();

    profiler.record_counter("io.read.bytes", 10);
    profiler.record_counter("io.read.bytes", 20);
    profiler.record_counter("io.read.calls", 1);

    let summary = profiler.counter_summary();
    profiler.disable();

    assert_eq!(summary.total("io.read.bytes"), 30);
    assert_eq!(summary.total("io.read.calls"), 1);
    assert_eq!(summary.total_matching("io.", ".bytes"), 30);
}

#[test]
#[allow(clippy::disallowed_macros)]
fn profiler_recovers_from_poisoned_metrics_lock() {
    let profiler = Arc::new(Profiler::enabled());
    let cloned = Arc::clone(&profiler);
    let shard = Profiler::shard_index("after_poison");
    let _ = std::thread::spawn(move || {
        let _guard = cloned.metrics[shard].write().unwrap();
        panic!("poison profiler metrics lock");
    })
    .join();

    profiler.record("after_poison", Duration::from_millis(1));

    assert_eq!(profiler.get("after_poison").unwrap().count, 1);
}

#[test]
fn profiler_summarizes_records_across_shards() {
    let profiler = Profiler::enabled();
    for index in 0..128 {
        let name = match index % 4 {
            0 => "profile.shard.a",
            1 => "profile.shard.b",
            2 => "profile.shard.c",
            _ => "profile.shard.d",
        };
        profiler.record(name, Duration::from_micros(index + 1));
    }

    let all = profiler.all();
    assert_eq!(all.len(), 4);

    let summary = profiler.summary();
    assert_eq!(summary.entries.len(), 4);
    assert_eq!(
        summary.entries.iter().map(|entry| entry.count).sum::<u64>(),
        128
    );
}

#[test]
fn profile_summary_display_uses_ms_columns() {
    let profiler = Profiler::enabled();
    profiler.record("tiny", Duration::from_micros(250));

    let report = profiler.summary().to_string();

    assert!(report.contains("Total ms"));
    assert!(report.contains("0.250"));
    assert!(!report.contains("us"));
}

#[test]
fn test_cache_stats() {
    let stats = CacheStats::new();
    stats.hit();
    stats.hit();
    stats.miss();

    assert!((stats.hit_rate() - 0.666).abs() < 0.01);
}

#[test]
fn span_attribution_builders_are_const_and_exact() {
    const ATTRIBUTION: SpanAttribution = SpanAttribution::new()
        .with_stage("s1")
        .with_pass("fold_constants")
        .with_file_id(7)
        .with_block("template")
        .with_span(5, 9);

    assert_eq!(ATTRIBUTION.stage, Some("s1"));
    assert_eq!(ATTRIBUTION.pass, Some("fold_constants"));
    assert_eq!(ATTRIBUTION.file_id, Some(7));
    assert_eq!(ATTRIBUTION.block, Some("template"));
    assert_eq!(ATTRIBUTION.span, Some(SpanRange { start: 5, end: 9 }));
    assert!(!ATTRIBUTION.is_empty());
    assert!(SpanAttribution::EMPTY.is_empty());
    assert_eq!(SpanAttribution::new(), SpanAttribution::EMPTY);
}

#[test]
fn attributed_records_stay_out_of_the_plain_key_space() {
    let profiler = Profiler::enabled();
    let attribution = SpanAttribution::new().with_stage("s1").with_pass("demo");

    profiler.record_attributed("davinci.attr.only", attribution, Duration::from_millis(3));
    profiler.record("davinci.attr.plain", Duration::from_millis(2));

    // Pre-attribution consumers observe exactly the historical key space.
    assert!(profiler.get("davinci.attr.only").is_none());
    assert_eq!(profiler.get("davinci.attr.plain").unwrap().count, 1);
    let all = profiler.all();
    assert_eq!(all.len(), 1);
    let summary = profiler.summary();
    assert_eq!(summary.entries.len(), 1);
    assert_eq!(summary.entries[0].name, "davinci.attr.plain");

    // The export sees both buckets, ranked by total wall time.
    let export = profiler.export_report(&export_options(ProfileExportBudget::default()));
    assert_eq!(export.spans.len(), 2);
    assert_eq!(export.spans[0].key, "davinci.attr.only");
    assert_eq!(
        export.spans[0]
            .attribution
            .map(|attribution| attribution.stage),
        Some(Some("s1"))
    );
    assert_eq!(export.spans[1].key, "davinci.attr.plain");
    assert_eq!(export.spans[1].attribution, None);
}

#[test]
fn attributed_buckets_with_different_attribution_stay_distinct() {
    let profiler = Profiler::enabled();
    let template = SpanAttribution::new()
        .with_pass("fold")
        .with_block("template");
    let script = SpanAttribution::new()
        .with_pass("fold")
        .with_block("script");

    profiler.record_attributed("davinci.attr.pass", template, Duration::from_millis(1));
    profiler.record_attributed("davinci.attr.pass", template, Duration::from_millis(1));
    profiler.record_attributed("davinci.attr.pass", script, Duration::from_millis(1));

    let export = profiler.export_report(&export_options(ProfileExportBudget::default()));
    assert_eq!(export.spans.len(), 2);
    // Equal keys: ranked by total descending, so the two-sample bucket wins.
    assert_eq!(export.spans[0].count, 2);
    assert_eq!(export.spans[0].attribution.unwrap().block, Some("template"));
    assert_eq!(export.spans[1].count, 1);
    assert_eq!(export.spans[1].attribution.unwrap().block, Some("script"));
}

#[test]
fn export_ranks_spans_deterministically_and_truncates_with_accounting() {
    let profiler = Profiler::enabled();
    profiler.record("davinci.rank.b", Duration::from_millis(5));
    profiler.record("davinci.rank.a", Duration::from_millis(5));
    profiler.record("davinci.rank.c", Duration::from_millis(9));
    profiler.record_counter("davinci.rank.counter.b", 2);
    profiler.record_counter("davinci.rank.counter.a", 1);

    let full = profiler.export_report(&export_options(ProfileExportBudget::default()));
    assert_eq!(full.spans.len(), 3);
    assert_eq!(full.spans[0].key, "davinci.rank.c");
    // Equal totals tie-break by key ascending.
    assert_eq!(full.spans[1].key, "davinci.rank.a");
    assert_eq!(full.spans[2].key, "davinci.rank.b");
    assert_eq!(full.counters.len(), 2);
    assert_eq!(full.counters[0].key, "davinci.rank.counter.a");
    assert_eq!(full.counters[1].key, "davinci.rank.counter.b");
    assert_eq!(full.truncation.dropped_spans, 0);
    assert_eq!(full.truncation.dropped_counters, 0);

    let truncated = profiler.export_report(&export_options(ProfileExportBudget {
        max_spans: 1,
        max_counters: 1,
    }));
    assert_eq!(truncated.spans.len(), 1);
    assert_eq!(truncated.spans[0].key, "davinci.rank.c");
    assert_eq!(truncated.counters.len(), 1);
    assert_eq!(truncated.counters[0].key, "davinci.rank.counter.a");
    assert_eq!(truncated.truncation.dropped_spans, 2);
    assert_eq!(truncated.truncation.dropped_counters, 1);
}

#[test]
fn export_json_bytes_are_exact() {
    let profiler = Profiler::enabled();
    profiler.record("davinci.export.plain", Duration::from_millis(2));
    profiler.record_attributed(
        "davinci.export.pass",
        SpanAttribution::new()
            .with_stage("s1")
            .with_pass("fold_constants")
            .with_file_id(7)
            .with_block("template")
            .with_span(5, 9),
        Duration::from_millis(1),
    );
    profiler.record_counter("davinci.export.bytes", 10);

    let export = profiler.export_report(&export_options(ProfileExportBudget::default()));
    let expected = concat!(
        "{\n",
        "  \"schema_version\": 1,\n",
        "  \"tool\": \"vize\",\n",
        "  \"tool_version\": \"",
        env!("CARGO_PKG_VERSION"),
        "\",\n",
        "  \"command\": \"build\",\n",
        "  \"budget\": {\n",
        "    \"max_spans\": 512,\n",
        "    \"max_counters\": 256\n",
        "  },\n",
        "  \"truncation\": {\n",
        "    \"dropped_spans\": 0,\n",
        "    \"dropped_counters\": 0\n",
        "  },\n",
        "  \"spans\": [\n",
        "    {\n",
        "      \"key\": \"davinci.export.plain\",\n",
        "      \"count\": 1,\n",
        "      \"wall_ns\": {\n",
        "        \"total\": 2000000,\n",
        "        \"self\": 2000000,\n",
        "        \"min\": 2000000,\n",
        "        \"max\": 2000000,\n",
        "        \"p50\": 2048000,\n",
        "        \"p95\": 2048000,\n",
        "        \"p99\": 2048000\n",
        "      },\n",
        "      \"alloc\": null\n",
        "    },\n",
        "    {\n",
        "      \"key\": \"davinci.export.pass\",\n",
        "      \"count\": 1,\n",
        "      \"wall_ns\": {\n",
        "        \"total\": 1000000,\n",
        "        \"self\": 1000000,\n",
        "        \"min\": 1000000,\n",
        "        \"max\": 1000000,\n",
        "        \"p50\": 1024000,\n",
        "        \"p95\": 1024000,\n",
        "        \"p99\": 1024000\n",
        "      },\n",
        "      \"alloc\": null,\n",
        "      \"attribution\": {\n",
        "        \"stage\": \"s1\",\n",
        "        \"pass\": \"fold_constants\",\n",
        "        \"file_id\": 7,\n",
        "        \"block\": \"template\",\n",
        "        \"span\": {\n",
        "          \"start\": 5,\n",
        "          \"end\": 9\n",
        "        }\n",
        "      }\n",
        "    }\n",
        "  ],\n",
        "  \"counters\": [\n",
        "    {\n",
        "      \"key\": \"davinci.export.bytes\",\n",
        "      \"samples\": 1,\n",
        "      \"total\": 10,\n",
        "      \"min\": 10,\n",
        "      \"max\": 10\n",
        "    }\n",
        "  ],\n",
        "  \"allocation\": null\n",
        "}\n",
    );
    assert_eq!(export.to_json(), expected);
}

#[test]
fn export_maps_the_global_allocation_window() {
    use super::AllocationSnapshot;

    let profiler = Profiler::enabled();
    profiler.record("davinci.alloc.window", Duration::from_millis(1));

    let snapshot = AllocationSnapshot {
        alloc_calls: 3,
        alloc_zeroed_calls: 1,
        alloc_failures: 1,
        alloc_zeroed_failures: 0,
        alloc_bytes: 96,
        alloc_zeroed_bytes: 32,
        dealloc_calls: 2,
        dealloc_bytes: 64,
        realloc_calls: 1,
        realloc_failures: 0,
        realloc_old_bytes: 16,
        realloc_new_bytes: 48,
    };
    let export = profiler.export_report(&ProfileExportOptions {
        command: "build",
        allocation: Some(snapshot),
        budget: ProfileExportBudget::default(),
    });

    let allocation = export.allocation.unwrap();
    assert_eq!(allocation.calls, 5);
    assert_eq!(allocation.requested_bytes, 176);
    assert_eq!(allocation.released_bytes, 80);
    assert_eq!(allocation.failures, 1);
    // Allocation tracking present: every span carries an alloc object (zeroed
    // here because the samples were recorded without a span guard).
    let span_alloc = export.spans[0].alloc.unwrap();
    assert_eq!(span_alloc.calls, 0);
    assert_eq!(span_alloc.bytes, 0);
    assert_eq!(span_alloc.self_calls, 0);
    assert_eq!(span_alloc.self_bytes, 0);
}
