//! Profiler core: timers, nested span guards, and the sharded metric store.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

use super::allocation::{
    ALLOCATION_TRACKING_ENABLED, pause_allocation_tracking, reset_allocation_counters,
};
use super::metrics::{CounterMetrics, Metrics};
use super::report::{CounterEntry, CounterSummary, ProfileEntry, ProfileSummary};

const PROFILER_SHARDS: usize = 32;

thread_local! {
    static PROFILE_STACK: RefCell<std::vec::Vec<ProfileFrame>> = const { RefCell::new(std::vec::Vec::new()) };
}

#[derive(Debug)]
struct ProfileFrame {
    name: &'static str,
    start: Instant,
    child_duration: Duration,
}

/// RAII guard for nested global profiling spans.
#[derive(Debug)]
pub struct ProfileGuard {
    profiler: &'static Profiler,
    active: bool,
}

impl ProfileGuard {
    #[inline]
    fn start(profiler: &'static Profiler, name: &'static str) -> Self {
        let _allocation_tracking = pause_allocation_tracking();
        PROFILE_STACK.with(|stack| {
            stack.borrow_mut().push(ProfileFrame {
                name,
                start: Instant::now(),
                child_duration: Duration::ZERO,
            });
        });
        Self {
            profiler,
            active: true,
        }
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        PROFILE_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let Some(frame) = stack.pop() else {
                return;
            };

            let duration = frame.start.elapsed();
            if let Some(parent) = stack.last_mut() {
                parent.child_duration += duration;
            }
            self.profiler
                .record_sample_enabled(frame.name, duration, frame.child_duration);
        });
    }
}

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

/// Performance profiler for collecting metrics.
///
/// Disabled profiling sits directly on several CLI/LSP hot paths, so the fast
/// path is just one relaxed atomic load in the `profile!` macro. When enabled,
/// samples are sharded by operation name to keep parallel file processing from
/// contending on one global lock, and profiler-internal allocation accounting is
/// paused so the measurement machinery does not count itself.
#[derive(Debug)]
pub struct Profiler {
    /// Metrics by operation name, split into shards to keep parallel profile runs from
    /// funnelling every span through the same lock.
    pub(super) metrics: [RwLock<FxHashMap<&'static str, Metrics>>; PROFILER_SHARDS],
    /// Non-duration counters by name.
    counters: [RwLock<FxHashMap<&'static str, CounterMetrics>>; PROFILER_SHARDS],
    /// Whether profiling is enabled
    enabled: AtomicBool,
}

impl Profiler {
    /// Create a new profiler.
    pub fn new() -> Self {
        Self {
            metrics: std::array::from_fn(|_| RwLock::new(FxHashMap::default())),
            counters: std::array::from_fn(|_| RwLock::new(FxHashMap::default())),
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
        reset_allocation_counters();
        ALLOCATION_TRACKING_ENABLED.store(true, Ordering::Relaxed);
        self.enabled.store(true, Ordering::Relaxed);
    }

    /// Disable profiling.
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
        ALLOCATION_TRACKING_ENABLED.store(false, Ordering::Relaxed);
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
        self.record_sample_enabled(name, duration, Duration::ZERO);
    }

    /// Start a nested profiling span on the global profiler.
    #[inline]
    pub fn global_span(&'static self, name: &'static str) -> Option<ProfileGuard> {
        if self.is_enabled() {
            Some(ProfileGuard::start(self, name))
        } else {
            None
        }
    }

    /// Record a duration and child duration after the caller has already checked profiling.
    ///
    /// `ProfileGuard::drop` uses this path after the macro has checked
    /// `is_enabled()`, avoiding another atomic load for every nested span.
    #[doc(hidden)]
    pub fn record_sample_enabled(
        &self,
        name: &'static str,
        duration: Duration,
        child_duration: Duration,
    ) {
        let _allocation_tracking = pause_allocation_tracking();
        let mut metrics = self.metrics_write(Self::shard_index(name));
        metrics
            .entry(name)
            .or_default()
            .record_with_child(duration, child_duration);
    }

    /// Record a non-duration counter sample.
    pub fn record_counter(&self, name: &'static str, value: u64) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled(name, value);
    }

    /// Record a counter after the caller has already checked profiling.
    #[doc(hidden)]
    pub fn record_counter_enabled(&self, name: &'static str, value: u64) {
        let _allocation_tracking = pause_allocation_tracking();
        let mut counters = self.counters_write(Self::shard_index(name));
        counters.entry(name).or_default().record(value);
    }

    /// Record a successful `std::fs::read_to_string` call.
    pub fn record_fs_read_to_string(&self, bytes: usize) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("io.read.calls", 1);
        self.record_counter_enabled("io.read.bytes", bytes as u64);
        self.record_counter_enabled("syscall.fs.read_to_string.calls", 1);
    }

    /// Record a failed `std::fs::read_to_string` call.
    pub fn record_fs_read_to_string_failure(&self) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("io.read.calls", 1);
        self.record_counter_enabled("io.read.failures", 1);
        self.record_counter_enabled("syscall.fs.read_to_string.calls", 1);
        self.record_counter_enabled("syscall.fs.read_to_string.failures", 1);
    }

    /// Record a successful `std::fs::write` call.
    pub fn record_fs_write(&self, bytes: usize) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("io.write.calls", 1);
        self.record_counter_enabled("io.write.attempted_bytes", bytes as u64);
        self.record_counter_enabled("io.write.bytes", bytes as u64);
        self.record_counter_enabled("syscall.fs.write.calls", 1);
    }

    /// Record a failed `std::fs::write` call.
    pub fn record_fs_write_failure(&self, bytes: usize) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("io.write.calls", 1);
        self.record_counter_enabled("io.write.attempted_bytes", bytes as u64);
        self.record_counter_enabled("io.write.failures", 1);
        self.record_counter_enabled("syscall.fs.write.calls", 1);
        self.record_counter_enabled("syscall.fs.write.failures", 1);
    }

    /// Record a successful `std::fs::create_dir_all` call.
    pub fn record_fs_create_dir_all(&self) {
        if self.is_enabled() {
            self.record_counter_enabled("syscall.fs.create_dir_all.calls", 1);
        }
    }

    /// Record a failed `std::fs::create_dir_all` call.
    pub fn record_fs_create_dir_all_failure(&self) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("syscall.fs.create_dir_all.calls", 1);
        self.record_counter_enabled("syscall.fs.create_dir_all.failures", 1);
    }

    /// Record a successful `std::fs::remove_dir_all` call.
    pub fn record_fs_remove_dir_all(&self) {
        if self.is_enabled() {
            self.record_counter_enabled("syscall.fs.remove_dir_all.calls", 1);
        }
    }

    /// Record a failed `std::fs::remove_dir_all` call.
    pub fn record_fs_remove_dir_all_failure(&self) {
        if !self.is_enabled() {
            return;
        }

        self.record_counter_enabled("syscall.fs.remove_dir_all.calls", 1);
        self.record_counter_enabled("syscall.fs.remove_dir_all.failures", 1);
    }

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
        for shard in &self.counters {
            shard
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear();
        }
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
    fn counters_write(
        &self,
        shard: usize,
    ) -> RwLockWriteGuard<'_, FxHashMap<&'static str, CounterMetrics>> {
        self.counters[shard]
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[inline]
    pub(super) fn shard_index(name: &str) -> usize {
        debug_assert!(PROFILER_SHARDS.is_power_of_two());

        // FNV-1a over static operation names is cheaper than building a
        // hasher per sample, and the power-of-two mask keeps sharding branchless.
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

/// Global profiler instance.
static GLOBAL_PROFILER: once_cell::sync::Lazy<Profiler> = once_cell::sync::Lazy::new(Profiler::new);

/// Get the global profiler.
#[inline]
pub fn global_profiler() -> &'static Profiler {
    &GLOBAL_PROFILER
}
