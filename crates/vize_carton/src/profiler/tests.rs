use super::{CacheStats, Metrics, Profiler, Timer};
use std::sync::Arc;
use std::time::Duration;

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
