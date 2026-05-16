//! Lightweight profiling utilities for performance monitoring.
//!
//! Provides simple timing and metrics collection for tracking
//! type checking and compilation performance.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

const PROFILER_SHARDS: usize = 32;

/// A lightweight timer for measuring durations.
#[derive(Debug)]
pub struct Timer {
    start: Instant,
    name: &'static str,
}

impl Timer {
    /// Start a new timer.
    #[inline]
    pub fn start(name: &'static str) -> Self {
        Self {
            start: Instant::now(),
            name,
        }
    }

    /// Get the elapsed time without stopping.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop the timer and return the elapsed time.
    #[inline]
    pub fn stop(self) -> Duration {
        self.elapsed()
    }

    /// Stop and record to a profiler.
    #[inline]
    pub fn record(self, profiler: &Profiler) {
        profiler.record(self.name, self.elapsed());
    }
}

/// Profiling metrics for a single operation.
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Number of times this operation was called
    pub count: u64,
    /// Total duration across all calls
    pub total_duration: Duration,
    /// Minimum duration
    pub min_duration: Duration,
    /// Maximum duration
    pub max_duration: Duration,
}

impl Metrics {
    /// Create new metrics.
    pub fn new() -> Self {
        Self {
            count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
        }
    }

    /// Record a duration.
    pub fn record(&mut self, duration: Duration) {
        self.count += 1;
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
    }

    /// Get the average duration.
    pub fn average(&self) -> Duration {
        if self.count == 0 {
            Duration::ZERO
        } else {
            let nanos = self.total_duration.as_nanos() / u128::from(self.count);
            Duration::from_nanos(nanos.try_into().unwrap_or(u64::MAX))
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance profiler for collecting metrics.
#[derive(Debug)]
pub struct Profiler {
    /// Metrics by operation name, split into shards to keep parallel profile runs from
    /// funnelling every span through the same lock.
    metrics: [RwLock<FxHashMap<&'static str, Metrics>>; PROFILER_SHARDS],
    /// Whether profiling is enabled
    enabled: AtomicBool,
}

impl Profiler {
    /// Create a new profiler.
    pub fn new() -> Self {
        Self {
            metrics: std::array::from_fn(|_| RwLock::new(FxHashMap::default())),
            enabled: AtomicBool::new(false),
        }
    }

    /// Create an enabled profiler.
    pub fn enabled() -> Self {
        let p = Self::new();
        p.enable();
        p
    }

    /// Enable profiling.
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    /// Disable profiling.
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    /// Check if profiling is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Start a timer for the given operation.
    #[inline]
    pub fn timer(&self, name: &'static str) -> Option<Timer> {
        if self.is_enabled() {
            Some(Timer::start(name))
        } else {
            None
        }
    }

    /// Record a duration for the given operation.
    pub fn record(&self, name: &'static str, duration: Duration) {
        if !self.is_enabled() {
            return;
        }

        self.record_enabled(name, duration);
    }

    /// Record a duration after the caller has already checked that profiling is enabled.
    #[doc(hidden)]
    pub fn record_enabled(&self, name: &'static str, duration: Duration) {
        let mut metrics = self.metrics_write(Self::shard_index(name));
        metrics.entry(name).or_default().record(duration);
    }

    /// Get metrics for the given operation.
    pub fn get(&self, name: &str) -> Option<Metrics> {
        self.metrics_read(Self::shard_index(name))
            .get(name)
            .cloned()
    }

    /// Get all metrics.
    pub fn all(&self) -> FxHashMap<&'static str, Metrics> {
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
        for shard in &self.metrics {
            shard
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }
    }

    /// Generate a summary report.
    pub fn summary(&self) -> ProfileSummary {
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
                average: m.average(),
                min: m.min_duration,
                max: m.max_duration,
            }));
        }

        // Sort by total time descending
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.total));

        ProfileSummary { entries }
    }

    #[inline]
    fn metrics_read(&self, shard: usize) -> RwLockReadGuard<'_, FxHashMap<&'static str, Metrics>> {
        self.metrics[shard]
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[inline]
    fn metrics_write(
        &self,
        shard: usize,
    ) -> RwLockWriteGuard<'_, FxHashMap<&'static str, Metrics>> {
        self.metrics[shard]
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[inline]
    fn shard_index(name: &str) -> usize {
        debug_assert!(PROFILER_SHARDS.is_power_of_two());

        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for byte in name.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        (hash as usize) & (PROFILER_SHARDS - 1)
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// A summary of profiling data.
#[derive(Debug)]
pub struct ProfileSummary {
    /// Entries sorted by total time
    pub entries: Vec<ProfileEntry>,
}

impl ProfileSummary {
    /// Check if any operation exceeded the threshold.
    pub fn has_slow_operations(&self, threshold: Duration) -> bool {
        self.entries.iter().any(|e| e.average > threshold)
    }

    /// Get slow operations.
    pub fn slow_operations(&self, threshold: Duration) -> Vec<&ProfileEntry> {
        self.entries
            .iter()
            .filter(|e| e.average > threshold)
            .collect()
    }
}

impl std::fmt::Display for ProfileSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Profile Summary:")?;
        writeln!(
            f,
            "{:<30} {:>8} {:>12} {:>12} {:>12} {:>12}",
            "Operation", "Count", "Total ms", "Avg ms", "Min ms", "Max ms"
        )?;
        writeln!(f, "{}", "-".repeat(88))?;

        for entry in &self.entries {
            writeln!(
                f,
                "{:<30} {:>8} {:>12.3} {:>12.3} {:>12.3} {:>12.3}",
                entry.name,
                entry.count,
                duration_ms(entry.total),
                duration_ms(entry.average),
                duration_ms(entry.min),
                duration_ms(entry.max)
            )?;
        }

        Ok(())
    }
}

#[inline]
fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// A single entry in the profile summary.
#[derive(Debug)]
pub struct ProfileEntry {
    /// Operation name
    pub name: &'static str,
    /// Number of calls
    pub count: u64,
    /// Total duration
    pub total: Duration,
    /// Average duration
    pub average: Duration,
    /// Minimum duration
    pub min: Duration,
    /// Maximum duration
    pub max: Duration,
}

/// Global profiler instance.
static GLOBAL_PROFILER: once_cell::sync::Lazy<Profiler> = once_cell::sync::Lazy::new(Profiler::new);

/// Get the global profiler.
#[inline]
pub fn global_profiler() -> &'static Profiler {
    &GLOBAL_PROFILER
}

/// Macro for profiling a block of code.
#[macro_export]
macro_rules! profile {
    ($name:expr, $block:expr) => {{
        let name: &'static str = $name;
        let profiler = $crate::profiler::global_profiler();
        if profiler.is_enabled() {
            let timer = $crate::profiler::Timer::start(name);
            let result = $block;
            profiler.record_enabled(name, timer.elapsed());
            result
        } else {
            $block
        }
    }};
}

/// Cache statistics.
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: AtomicU64,
    /// Number of cache misses
    pub misses: AtomicU64,
    /// Total entries in cache
    pub entries: AtomicU64,
}

impl CacheStats {
    /// Create new cache stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit.
    #[inline]
    pub fn hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss.
    #[inline]
    pub fn miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Update entry count.
    #[inline]
    pub fn set_entries(&self, count: u64) {
        self.entries.store(count, Ordering::Relaxed);
    }

    /// Get the hit rate (0.0 - 1.0).
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Reset statistics.
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {} hits, {} misses ({:.1}% hit rate), {} entries",
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            self.hit_rate() * 100.0,
            self.entries.load(Ordering::Relaxed)
        )
    }
}

#[cfg(test)]
mod tests {
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
        };

        assert_eq!(
            metrics.average(),
            Duration::from_nanos(
                (Duration::from_secs(10).as_nanos() / u128::from(metrics.count)) as u64
            )
        );
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
}
