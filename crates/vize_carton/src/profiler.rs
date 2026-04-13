//! Lightweight profiling utilities for performance monitoring.
//!
//! Provides simple timing and metrics collection for tracking
//! type checking and compilation performance.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

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
            self.total_duration / self.count as u32
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance profiler for collecting metrics.
#[derive(Debug, Default)]
pub struct Profiler {
    /// Metrics by operation name
    metrics: std::sync::RwLock<FxHashMap<&'static str, Metrics>>,
    /// Whether profiling is enabled
    enabled: std::sync::atomic::AtomicBool,
}

impl Profiler {
    /// Create a new profiler.
    pub fn new() -> Self {
        Self::default()
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

        let mut metrics = self.metrics.write().unwrap();
        metrics.entry(name).or_default().record(duration);
    }

    /// Get metrics for the given operation.
    pub fn get(&self, name: &str) -> Option<Metrics> {
        self.metrics.read().unwrap().get(name).cloned()
    }

    /// Get all metrics.
    pub fn all(&self) -> FxHashMap<&'static str, Metrics> {
        self.metrics.read().unwrap().clone()
    }

    /// Clear all metrics.
    pub fn clear(&self) {
        self.metrics.write().unwrap().clear();
    }

    /// Generate a summary report.
    pub fn summary(&self) -> ProfileSummary {
        let metrics = self.metrics.read().unwrap();
        let mut entries: Vec<_> = metrics
            .iter()
            .map(|(name, m)| ProfileEntry {
                name,
                count: m.count,
                total: m.total_duration,
                average: m.average(),
                min: m.min_duration,
                max: m.max_duration,
            })
            .collect();

        // Sort by total time descending
        entries.sort_by(|a, b| b.total.cmp(&a.total));

        ProfileSummary { entries }
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
            "Operation", "Count", "Total", "Average", "Min", "Max"
        )?;
        writeln!(f, "{}", "-".repeat(88))?;

        for entry in &self.entries {
            writeln!(
                f,
                "{:<30} {:>8} {:>12.2?} {:>12.2?} {:>12.2?} {:>12.2?}",
                entry.name, entry.count, entry.total, entry.average, entry.min, entry.max
            )?;
        }

        Ok(())
    }
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
        let _timer = $crate::profiler::global_profiler().timer($name);
        let result = $block;
        if let Some(timer) = _timer {
            timer.record($crate::profiler::global_profiler());
        }
        result
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
    use super::{CacheStats, Profiler, Timer};
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
    fn test_cache_stats() {
        let stats = CacheStats::new();
        stats.hit();
        stats.hit();
        stats.miss();

        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }
}
