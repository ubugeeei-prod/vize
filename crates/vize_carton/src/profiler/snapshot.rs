//! Metric readback: lookups, snapshots for the export, and summaries.
//!
//! Split out of `core.rs`; these are `impl Profiler` methods reading the same
//! sharded stores the recording paths write.

use rustc_hash::FxHashMap;

use super::allocation::pause_allocation_tracking;
use super::attribution::SpanAttribution;
use super::core::Profiler;
use super::metrics::{CounterMetrics, Metrics};
use super::report::{CounterEntry, CounterSummary, ProfileEntry, ProfileSummary};

impl Profiler {
    /// Get metrics for the given operation.
    pub fn get(&self, name: &str) -> Option<Metrics> {
        self.metrics_read(Self::shard_index(name))
            .get(name)
            .cloned()
    }

    /// Get all metrics.
    pub fn all(&self) -> FxHashMap<&'static str, Metrics> {
        let _allocation_tracking = pause_allocation_tracking();
        let mut all = FxHashMap::default();
        for shard in &self.metrics {
            let metrics = shard
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            all.extend(
                metrics
                    .iter()
                    .map(|(name, metrics)| (*name, metrics.clone())),
            );
        }
        all
    }

    /// Clear all metrics.
    pub fn clear(&self) {
        let _allocation_tracking = pause_allocation_tracking();
        for shard in &self.metrics {
            shard
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }
        for shard in &self.attributed {
            shard
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }
        for shard in &self.counters {
            shard
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }
    }

    /// Snapshot every span bucket — plain dotted keys first, then attributed
    /// keys — for the machine-readable export.
    pub(super) fn span_snapshot(&self) -> Vec<(&'static str, SpanAttribution, Metrics)> {
        let _allocation_tracking = pause_allocation_tracking();
        let mut spans = Vec::new();
        for shard in &self.metrics {
            let metrics = shard
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            spans.reserve(metrics.len());
            spans.extend(
                metrics
                    .iter()
                    .map(|(name, metrics)| (*name, SpanAttribution::EMPTY, metrics.clone())),
            );
        }
        for shard in &self.attributed {
            let metrics = shard
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            spans.reserve(metrics.len());
            spans.extend(
                metrics
                    .iter()
                    .map(|(key, metrics)| (key.name, key.attribution, metrics.clone())),
            );
        }
        spans
    }

    /// Snapshot every counter for the machine-readable export.
    pub(super) fn counter_snapshot(&self) -> Vec<(&'static str, CounterMetrics)> {
        let _allocation_tracking = pause_allocation_tracking();
        let mut counters = Vec::new();
        for shard in &self.counters {
            let entries = shard
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            counters.reserve(entries.len());
            counters.extend(
                entries
                    .iter()
                    .map(|(name, counter)| (*name, counter.clone())),
            );
        }
        counters
    }

    /// Generate a summary report.
    pub fn summary(&self) -> ProfileSummary {
        let _allocation_tracking = pause_allocation_tracking();
        let mut entries = Vec::new();
        for shard in &self.metrics {
            let metrics = shard
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            entries.reserve(metrics.len());
            entries.extend(metrics.iter().map(|(name, m)| ProfileEntry {
                name,
                count: m.count,
                total: m.total_duration,
                self_total: m.self_duration,
                child_total: m.child_duration,
                average: m.average(),
                self_average: m.self_average(),
                min: m.min_duration,
                max: m.max_duration,
                self_min: m.min_self_duration,
                self_max: m.max_self_duration,
                p50: m.percentile(0.50),
                p95: m.percentile(0.95),
                p99: m.percentile(0.99),
                samples_over_1ms: m.samples_over_1ms(),
                samples_over_10ms: m.samples_over_10ms(),
                samples_over_100ms: m.samples_over_100ms(),
            }));
        }

        // Sort by total time descending
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.total));

        ProfileSummary { entries }
    }

    /// Generate a counter summary report.
    pub fn counter_summary(&self) -> CounterSummary {
        let _allocation_tracking = pause_allocation_tracking();
        let mut entries = Vec::new();
        for shard in &self.counters {
            let counters = shard
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            entries.reserve(counters.len());
            entries.extend(counters.iter().map(|(name, counter)| CounterEntry {
                name,
                samples: counter.samples,
                total: counter.total,
                average: counter.average(),
                min: if counter.samples == 0 { 0 } else { counter.min },
                max: counter.max,
            }));
        }

        entries.sort_by(|left, right| left.name.cmp(right.name));

        CounterSummary { entries }
    }
}
