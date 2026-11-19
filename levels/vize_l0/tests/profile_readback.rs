//! Host-neutral metric readbacks retain the collector's existing buckets.

use core::time::Duration;
use vize_l0::profiler::{CounterMetrics, Metrics, Profiler, SpanAttribution};

#[test]
fn owned_readbacks_keep_plain_attributed_and_counter_buckets_distinct() {
    let profiler = Profiler::enabled();
    let attribution = SpanAttribution::new().with_stage("s2").with_pass("fold");
    profiler.record("shared", Duration::from_millis(2));
    profiler.record_attributed("shared", attribution, Duration::from_millis(3));
    profiler.record_counter("bytes", 10);
    profiler.record_counter("bytes", 20);

    let spans: Vec<(&'static str, SpanAttribution, Metrics)> = profiler.span_snapshot();
    let counters: Vec<(&'static str, CounterMetrics)> = profiler.counter_snapshot();
    profiler.clear();
    drop(profiler);

    assert_eq!(spans.len(), 2);
    // Plain buckets precede attributed buckets, without a host-specific sort.
    assert_eq!(spans[0].0, "shared");
    assert_eq!(spans[0].1, SpanAttribution::EMPTY);
    assert_eq!(spans[0].2.total_duration, Duration::from_millis(2));
    assert_eq!(spans[1].0, "shared");
    assert_eq!(spans[1].1, attribution);
    assert_eq!(spans[1].2.total_duration, Duration::from_millis(3));
    assert_eq!(counters.len(), 1);
    assert_eq!(counters[0].0, "bytes");
    assert_eq!(counters[0].1.samples, 2);
    assert_eq!(counters[0].1.total, 30);
    assert_eq!(counters[0].1.min, 10);
    assert_eq!(counters[0].1.max, 20);
}

#[test]
fn readbacks_of_a_disabled_collector_are_empty() {
    let profiler = Profiler::new();
    profiler.record("disabled", Duration::from_millis(1));
    profiler.record_attributed("disabled", SpanAttribution::EMPTY, Duration::from_millis(1));
    profiler.record_counter("disabled", 1);
    assert!(profiler.span_snapshot().is_empty());
    assert!(profiler.counter_snapshot().is_empty());
}
