//! The existing deterministic span/counter ranking and wire assembly.

use core::time::Duration;

use vize_l0::profiler::{CounterMetrics, Metrics, SpanAttribution};

use super::{
    PROFILE_EXPORT_SCHEMA_VERSION, ProfileExport, ProfileExportAllocCounts,
    ProfileExportAllocation, ProfileExportAttribution, ProfileExportCounter, ProfileExportOptions,
    ProfileExportSpan, ProfileExportTruncation, ProfileExportWallNs,
};

pub(super) fn spans(
    mut ranked_spans: Vec<(&'static str, SpanAttribution, Metrics)>,
    options: &ProfileExportOptions,
) -> (Vec<ProfileExportSpan>, u64) {
    ranked_spans.sort_by(
        |(left_name, left_attribution, left), (right_name, right_attribution, right)| {
            right
                .total_duration
                .cmp(&left.total_duration)
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_attribution.cmp(right_attribution))
        },
    );
    let span_limit = usize::try_from(options.budget.max_spans).unwrap_or(usize::MAX);
    let dropped_spans = ranked_spans.len().saturating_sub(span_limit) as u64;
    ranked_spans.truncate(span_limit);
    let spans = ranked_spans
        .into_iter()
        .map(|(name, attribution, metrics)| {
            span_entry(name, attribution, &metrics, options.allocation.is_some())
        })
        .collect();

    (spans, dropped_spans)
}

pub(super) fn counters(
    mut ranked_counters: Vec<(&'static str, CounterMetrics)>,
    options: &ProfileExportOptions,
) -> (Vec<ProfileExportCounter>, u64) {
    ranked_counters.sort_by_key(|(key, _)| *key);
    let counter_limit = usize::try_from(options.budget.max_counters).unwrap_or(usize::MAX);
    let dropped_counters = ranked_counters.len().saturating_sub(counter_limit) as u64;
    ranked_counters.truncate(counter_limit);
    let counters = ranked_counters
        .into_iter()
        .map(|(key, counter)| ProfileExportCounter {
            key,
            samples: counter.samples,
            total: counter.total,
            min: if counter.samples == 0 { 0 } else { counter.min },
            max: counter.max,
        })
        .collect();

    (counters, dropped_counters)
}

pub(super) fn report(
    (spans, dropped_spans): (Vec<ProfileExportSpan>, u64),
    (counters, dropped_counters): (Vec<ProfileExportCounter>, u64),
    options: &ProfileExportOptions,
) -> ProfileExport {
    ProfileExport {
        schema_version: PROFILE_EXPORT_SCHEMA_VERSION,
        tool: "vize",
        tool_version: env!("CARGO_PKG_VERSION"),
        command: options.command,
        budget: options.budget,
        truncation: ProfileExportTruncation {
            dropped_spans,
            dropped_counters,
        },
        spans,
        counters,
        allocation: options.allocation.map(|snapshot| ProfileExportAllocation {
            calls: snapshot.allocation_calls(),
            requested_bytes: snapshot.requested_bytes(),
            released_bytes: snapshot.released_bytes(),
            failures: snapshot.allocation_failures(),
        }),
    }
}

fn span_entry(
    key: &'static str,
    attribution: SpanAttribution,
    metrics: &Metrics,
    allocation_tracked: bool,
) -> ProfileExportSpan {
    ProfileExportSpan {
        key,
        count: metrics.count,
        wall_ns: ProfileExportWallNs {
            total: duration_ns(metrics.total_duration),
            self_ns: duration_ns(metrics.self_duration),
            min: duration_ns(metrics.min_duration),
            max: duration_ns(metrics.max_duration),
            p50: duration_ns(metrics.percentile(0.50)),
            p95: duration_ns(metrics.percentile(0.95)),
            p99: duration_ns(metrics.percentile(0.99)),
        },
        alloc: allocation_tracked.then_some(ProfileExportAllocCounts {
            calls: metrics.alloc_calls,
            bytes: metrics.alloc_bytes,
            self_calls: metrics.self_alloc_calls,
            self_bytes: metrics.self_alloc_bytes,
        }),
        attribution: ProfileExportAttribution::from_attribution(attribution),
    }
}

#[inline]
fn duration_ns(duration: Duration) -> u64 {
    duration.as_nanos().try_into().unwrap_or(u64::MAX)
}
