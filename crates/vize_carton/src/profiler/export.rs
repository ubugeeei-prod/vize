//! Machine-readable profile export.
//!
//! Serializes the profiler's span, counter, and allocation data into the
//! stable JSON shape committed at
//! `davinci-road/plan/profile-export.schema.json` (the `--profile-json` CLI
//! contract). The export is size-budgeted following the
//! `vize_doctor::ai_context` conventions: hard entry limits applied in
//! deterministic order, with every omission accounted for in an explicit
//! truncation record — never silently.

use std::time::Duration;

use serde::Serialize;

use crate::String;

use super::allocation::AllocationSnapshot;
use super::attribution::SpanAttribution;
use super::core::Profiler;
use super::metrics::Metrics;

/// Current `schema_version` stamped into every export.
pub const PROFILE_EXPORT_SCHEMA_VERSION: u64 = 1;

/// Hard limits applied while constructing a [`ProfileExport`].
///
/// Entries are ranked deterministically (spans by total wall time, counters
/// by key) before the limits apply, so the hottest data always survives. The
/// applied budget is recorded in the export next to the truncation counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileExportBudget {
    /// Maximum span entries. Defaults to 512.
    pub max_spans: u64,
    /// Maximum counter entries. Defaults to 256.
    pub max_counters: u64,
}

impl Default for ProfileExportBudget {
    fn default() -> Self {
        Self {
            max_spans: 512,
            max_counters: 256,
        }
    }
}

/// Explicit accounting for entries omitted by [`ProfileExportBudget`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct ProfileExportTruncation {
    /// Lower-ranked span entries excluded by `max_spans`.
    pub dropped_spans: u64,
    /// Counter entries excluded by `max_counters`.
    pub dropped_counters: u64,
}

/// Inputs for [`Profiler::export_report`].
#[derive(Debug, Clone, Copy)]
pub struct ProfileExportOptions {
    /// Producing CLI subcommand (for example `"build"`).
    pub command: &'static str,
    /// Process-wide allocation window totals; `None` when the profiling
    /// allocator is not installed as the global allocator.
    pub allocation: Option<AllocationSnapshot>,
    /// Entry limits for the export.
    pub budget: ProfileExportBudget,
}

/// Half-open byte range in the wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileExportSpanRange {
    /// Inclusive start byte offset.
    pub start: u32,
    /// Exclusive end byte offset.
    pub end: u32,
}

/// Structured source attribution in the wire format.
///
/// Only present fields are serialized; an entirely absent attribution object
/// means the span is a plain dotted-key aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileExportAttribution {
    /// Pipeline stage name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<&'static str>,
    /// Pass name within the stage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<&'static str>,
    /// Producer-scoped source file id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<u32>,
    /// SFC block kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<&'static str>,
    /// Attributed source byte range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<ProfileExportSpanRange>,
}

impl ProfileExportAttribution {
    fn from_attribution(attribution: SpanAttribution) -> Option<Self> {
        if attribution.is_empty() {
            return None;
        }
        Some(Self {
            stage: attribution.stage,
            pass: attribution.pass,
            file_id: attribution.file_id,
            block: attribution.block,
            span: attribution.span.map(|span| ProfileExportSpanRange {
                start: span.start,
                end: span.end,
            }),
        })
    }
}

/// Wall-clock aggregates for one span entry, in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileExportWallNs {
    /// Total wall time across all calls, nested child spans included.
    pub total: u64,
    /// Total wall time excluding nested child spans.
    #[serde(rename = "self")]
    pub self_ns: u64,
    /// Minimum single-call wall time.
    pub min: u64,
    /// Maximum single-call wall time.
    pub max: u64,
    /// Approximate p50 single-call wall time (histogram bucket upper bound).
    pub p50: u64,
    /// Approximate p95 single-call wall time (histogram bucket upper bound).
    pub p95: u64,
    /// Approximate p99 single-call wall time (histogram bucket upper bound).
    pub p99: u64,
}

/// Allocation aggregates for one span entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileExportAllocCounts {
    /// Allocation-like calls on the span's thread, child spans included.
    pub calls: u64,
    /// Bytes requested by those calls.
    pub bytes: u64,
    /// Allocation-like calls excluding nested child spans.
    pub self_calls: u64,
    /// Requested bytes excluding nested child spans.
    pub self_bytes: u64,
}

/// One span entry: a dotted key, optional attribution, and its aggregates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileExportSpan {
    /// Dotted operation key (for example `"atelier.dom.template.parse"`).
    pub key: &'static str,
    /// Number of calls aggregated into this entry.
    pub count: u64,
    /// Wall-clock aggregates in nanoseconds.
    pub wall_ns: ProfileExportWallNs,
    /// Allocation aggregates; `null` when the profiling allocator is not
    /// installed as the global allocator.
    pub alloc: Option<ProfileExportAllocCounts>,
    /// Structured source attribution; absent for plain dotted-key spans.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<ProfileExportAttribution>,
}

/// One non-duration counter entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileExportCounter {
    /// Dotted counter key (for example `"io.read.bytes"`).
    pub key: &'static str,
    /// Number of samples recorded.
    pub samples: u64,
    /// Sum of all recorded samples.
    pub total: u64,
    /// Smallest sample, or zero when no samples were recorded.
    pub min: u64,
    /// Largest sample.
    pub max: u64,
}

/// Process-wide allocation totals over the profile window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileExportAllocation {
    /// Allocation-like calls (`alloc` + `alloc_zeroed` + successful `realloc`).
    pub calls: u64,
    /// Bytes requested through allocation-like calls.
    pub requested_bytes: u64,
    /// Bytes released or replaced during the window.
    pub released_bytes: u64,
    /// Failed allocation-like calls.
    pub failures: u64,
}

/// Versioned, deterministic machine-readable profile report.
///
/// Shape is committed at `davinci-road/plan/profile-export.schema.json`;
/// bump [`PROFILE_EXPORT_SCHEMA_VERSION`] with any incompatible change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProfileExport {
    /// Export format version.
    pub schema_version: u64,
    /// Producing tool. Always `"vize"`.
    pub tool: &'static str,
    /// Producing tool version.
    pub tool_version: &'static str,
    /// Producing CLI subcommand.
    pub command: &'static str,
    /// Entry limits that were applied while building the export.
    pub budget: ProfileExportBudget,
    /// Explicit accounting for budget-omitted entries.
    pub truncation: ProfileExportTruncation,
    /// Span entries, ranked by total wall time descending (ties broken by
    /// key, then attribution) and capped at `budget.max_spans`.
    pub spans: Vec<ProfileExportSpan>,
    /// Counter entries, sorted by key and capped at `budget.max_counters`.
    pub counters: Vec<ProfileExportCounter>,
    /// Process-wide allocation totals; `null` when the profiling allocator is
    /// not installed as the global allocator.
    pub allocation: Option<ProfileExportAllocation>,
}

impl ProfileExport {
    /// Serialize to pretty-printed JSON with a trailing newline.
    pub fn to_json(&self) -> String {
        // Derived `Serialize` on plain structs with string keys cannot fail.
        let mut text =
            serde_json::to_string_pretty(self).expect("profile export serialization is infallible");
        text.push('\n');
        text.into()
    }
}

impl Profiler {
    /// Build the machine-readable export from the collected profile data.
    ///
    /// Deterministic for a given data set: spans are ranked by total wall
    /// time descending with full tiebreaks, counters by key, and the budget
    /// drops only the tail of each ranking, recording how much was dropped.
    pub fn export_report(&self, options: &ProfileExportOptions) -> ProfileExport {
        let mut ranked_spans = self.span_snapshot();
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

        let mut ranked_counters = self.counter_snapshot();
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
