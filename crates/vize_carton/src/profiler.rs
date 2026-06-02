//! Lightweight profiling utilities for performance monitoring.
//!
//! Provides simple timing and metrics collection for tracking
//! type checking and compilation performance.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

const PROFILER_SHARDS: usize = 32;
const PROFILE_HISTOGRAM_BUCKETS: usize = 48;

thread_local! {
    static PROFILE_STACK: RefCell<std::vec::Vec<ProfileFrame>> = const { RefCell::new(std::vec::Vec::new()) };
    static ALLOCATION_TRACKING_SUPPRESSION: Cell<u32> = const { Cell::new(0) };
}

static ALLOCATION_TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);
static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOC_ZEROED_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOC_FAILURES: AtomicU64 = AtomicU64::new(0);
static ALLOC_ZEROED_FAILURES: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static ALLOC_ZEROED_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static DEALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static REALLOC_FAILURES: AtomicU64 = AtomicU64::new(0);
static REALLOC_OLD_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_NEW_BYTES: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct AllocationTrackingPause;

impl Drop for AllocationTrackingPause {
    fn drop(&mut self) {
        ALLOCATION_TRACKING_SUPPRESSION.with(|depth| {
            depth.set(depth.get().saturating_sub(1));
        });
    }
}

#[inline]
fn pause_allocation_tracking() -> AllocationTrackingPause {
    ALLOCATION_TRACKING_SUPPRESSION.with(|depth| {
        depth.set(depth.get().saturating_add(1));
    });
    AllocationTrackingPause
}

#[inline]
fn allocation_tracking_is_suppressed() -> bool {
    ALLOCATION_TRACKING_SUPPRESSION
        .try_with(|depth| depth.get() > 0)
        .unwrap_or(false)
}

#[inline]
fn allocation_tracking_is_enabled() -> bool {
    ALLOCATION_TRACKING_ENABLED.load(Ordering::Relaxed) && !allocation_tracking_is_suppressed()
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

/// Profiling metrics for a single operation.
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Number of times this operation was called
    pub count: u64,
    /// Total duration across all calls
    pub total_duration: Duration,
    /// Total duration excluding nested child spans
    pub self_duration: Duration,
    /// Total nested child span duration
    pub child_duration: Duration,
    /// Minimum duration
    pub min_duration: Duration,
    /// Maximum duration
    pub max_duration: Duration,
    /// Minimum self duration
    pub min_self_duration: Duration,
    /// Maximum self duration
    pub max_self_duration: Duration,
    histogram: [u64; PROFILE_HISTOGRAM_BUCKETS],
    samples_over_1ms: u64,
    samples_over_10ms: u64,
    samples_over_100ms: u64,
}

impl Metrics {
    /// Create new metrics.
    pub fn new() -> Self {
        Self {
            count: 0,
            total_duration: Duration::ZERO,
            self_duration: Duration::ZERO,
            child_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            min_self_duration: Duration::MAX,
            max_self_duration: Duration::ZERO,
            histogram: [0; PROFILE_HISTOGRAM_BUCKETS],
            samples_over_1ms: 0,
            samples_over_10ms: 0,
            samples_over_100ms: 0,
        }
    }

    /// Record a duration.
    pub fn record(&mut self, duration: Duration) {
        self.record_with_child(duration, Duration::ZERO);
    }

    /// Record a duration and the already-accounted nested child span duration.
    pub fn record_with_child(&mut self, duration: Duration, child_duration: Duration) {
        let self_duration = duration.saturating_sub(child_duration);

        self.count += 1;
        self.total_duration += duration;
        self.self_duration += self_duration;
        self.child_duration += child_duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
        self.min_self_duration = self.min_self_duration.min(self_duration);
        self.max_self_duration = self.max_self_duration.max(self_duration);
        self.histogram[duration_bucket(duration)] += 1;

        if duration >= Duration::from_millis(1) {
            self.samples_over_1ms += 1;
        }
        if duration >= Duration::from_millis(10) {
            self.samples_over_10ms += 1;
        }
        if duration >= Duration::from_millis(100) {
            self.samples_over_100ms += 1;
        }
    }

    /// Get the average duration.
    pub fn average(&self) -> Duration {
        average_duration(self.total_duration, self.count)
    }

    /// Get the average self duration.
    pub fn self_average(&self) -> Duration {
        average_duration(self.self_duration, self.count)
    }

    /// Estimate a percentile from the logarithmic duration histogram.
    pub fn percentile(&self, percentile: f64) -> Duration {
        if self.count == 0 {
            return Duration::ZERO;
        }

        let target = ((self.count as f64) * percentile.clamp(0.0, 1.0)).ceil() as u64;
        let target = target.max(1);
        let mut seen = 0u64;
        for (index, count) in self.histogram.iter().enumerate() {
            seen += count;
            if seen >= target {
                return bucket_upper_bound(index);
            }
        }
        bucket_upper_bound(PROFILE_HISTOGRAM_BUCKETS - 1)
    }

    /// Number of samples at or above one millisecond.
    pub fn samples_over_1ms(&self) -> u64 {
        self.samples_over_1ms
    }

    /// Number of samples at or above ten milliseconds.
    pub fn samples_over_10ms(&self) -> u64 {
        self.samples_over_10ms
    }

    /// Number of samples at or above one hundred milliseconds.
    pub fn samples_over_100ms(&self) -> u64 {
        self.samples_over_100ms
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Monotonic counter metrics for non-duration profile signals.
#[derive(Debug, Clone)]
pub struct CounterMetrics {
    /// Number of samples recorded for this counter.
    pub samples: u64,
    /// Sum of all recorded values.
    pub total: u64,
    /// Smallest recorded value.
    pub min: u64,
    /// Largest recorded value.
    pub max: u64,
}

impl CounterMetrics {
    /// Create new counter metrics.
    pub fn new() -> Self {
        Self {
            samples: 0,
            total: 0,
            min: u64::MAX,
            max: 0,
        }
    }

    /// Record a counter sample.
    pub fn record(&mut self, value: u64) {
        self.samples = self.samples.saturating_add(1);
        self.total = self.total.saturating_add(value);
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    /// Get the average recorded value.
    pub fn average(&self) -> f64 {
        if self.samples == 0 {
            0.0
        } else {
            self.total as f64 / self.samples as f64
        }
    }
}

impl Default for CounterMetrics {
    fn default() -> Self {
        Self::new()
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
    metrics: [RwLock<FxHashMap<&'static str, Metrics>>; PROFILER_SHARDS],
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
    fn shard_index(name: &str) -> usize {
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

/// A summary of non-duration profile counters.
#[derive(Debug)]
pub struct CounterSummary {
    /// Counter entries sorted by name.
    pub entries: Vec<CounterEntry>,
}

impl CounterSummary {
    /// Get a counter total by name.
    pub fn total(&self, name: &str) -> u64 {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.total)
            .unwrap_or(0)
    }

    /// Sum counter totals matching a prefix and suffix.
    pub fn total_matching(&self, prefix: &str, suffix: &str) -> u64 {
        self.entries
            .iter()
            .filter(|entry| entry.name.starts_with(prefix) && entry.name.ends_with(suffix))
            .map(|entry| entry.total)
            .sum()
    }
}

/// A single non-duration counter entry.
#[derive(Debug)]
pub struct CounterEntry {
    /// Counter name.
    pub name: &'static str,
    /// Number of samples recorded.
    pub samples: u64,
    /// Sum of all recorded samples.
    pub total: u64,
    /// Average value per sample.
    pub average: f64,
    /// Smallest sample.
    pub min: u64,
    /// Largest sample.
    pub max: u64,
}

impl std::fmt::Display for ProfileSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Profile Summary:")?;
        writeln!(
            f,
            "{:<30} {:>8} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
            "Operation", "Count", "Total ms", "Self ms", "Avg ms", "P95 ms", "Min ms", "Max ms"
        )?;
        writeln!(f, "{}", "-".repeat(114))?;

        for entry in &self.entries {
            writeln!(
                f,
                "{:<30} {:>8} {:>12.3} {:>12.3} {:>12.3} {:>12.3} {:>12.3} {:>12.3}",
                entry.name,
                entry.count,
                duration_ms(entry.total),
                duration_ms(entry.self_total),
                duration_ms(entry.average),
                duration_ms(entry.p95),
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
    /// Total self duration excluding nested child spans
    pub self_total: Duration,
    /// Total nested child duration
    pub child_total: Duration,
    /// Average duration
    pub average: Duration,
    /// Average self duration
    pub self_average: Duration,
    /// Minimum duration
    pub min: Duration,
    /// Maximum duration
    pub max: Duration,
    /// Minimum self duration
    pub self_min: Duration,
    /// Maximum self duration
    pub self_max: Duration,
    /// Approximate p50 duration
    pub p50: Duration,
    /// Approximate p95 duration
    pub p95: Duration,
    /// Approximate p99 duration
    pub p99: Duration,
    /// Samples at or above 1ms
    pub samples_over_1ms: u64,
    /// Samples at or above 10ms
    pub samples_over_10ms: u64,
    /// Samples at or above 100ms
    pub samples_over_100ms: u64,
}

/// Allocation counters captured for a profile window.
#[derive(Debug, Clone, Copy, Default)]
pub struct AllocationSnapshot {
    /// Successful `alloc` calls.
    pub alloc_calls: u64,
    /// Successful `alloc_zeroed` calls.
    pub alloc_zeroed_calls: u64,
    /// Failed `alloc` calls.
    pub alloc_failures: u64,
    /// Failed `alloc_zeroed` calls.
    pub alloc_zeroed_failures: u64,
    /// Bytes requested through successful `alloc` calls.
    pub alloc_bytes: u64,
    /// Bytes requested through successful `alloc_zeroed` calls.
    pub alloc_zeroed_bytes: u64,
    /// `dealloc` calls.
    pub dealloc_calls: u64,
    /// Bytes released through `dealloc`.
    pub dealloc_bytes: u64,
    /// Successful `realloc` calls.
    pub realloc_calls: u64,
    /// Failed `realloc` calls.
    pub realloc_failures: u64,
    /// Old layout bytes passed to successful `realloc` calls.
    pub realloc_old_bytes: u64,
    /// New size bytes requested by successful `realloc` calls.
    pub realloc_new_bytes: u64,
}

impl AllocationSnapshot {
    /// Allocation-like calls that requested new storage.
    pub fn allocation_calls(&self) -> u64 {
        self.alloc_calls
            .saturating_add(self.alloc_zeroed_calls)
            .saturating_add(self.realloc_calls)
    }

    /// Total allocation failures.
    pub fn allocation_failures(&self) -> u64 {
        self.alloc_failures
            .saturating_add(self.alloc_zeroed_failures)
            .saturating_add(self.realloc_failures)
    }

    /// Bytes requested through allocation-like calls.
    pub fn requested_bytes(&self) -> u64 {
        self.alloc_bytes
            .saturating_add(self.alloc_zeroed_bytes)
            .saturating_add(self.realloc_new_bytes)
    }

    /// Bytes released or replaced in this profile window.
    pub fn released_bytes(&self) -> u64 {
        self.dealloc_bytes.saturating_add(self.realloc_old_bytes)
    }

    /// Approximate heap delta during this profile window.
    pub fn net_bytes(&self) -> i128 {
        i128::from(self.requested_bytes()) - i128::from(self.released_bytes())
    }

    /// Average requested bytes per allocation-like call.
    pub fn requested_bytes_per_call(&self) -> f64 {
        let calls = self.allocation_calls();
        if calls == 0 {
            0.0
        } else {
            self.requested_bytes() as f64 / calls as f64
        }
    }
}

/// Global allocator wrapper that records allocation pressure while profiling is enabled.
#[derive(Debug)]
pub struct ProfilingAllocator<A = System> {
    inner: A,
}

impl ProfilingAllocator<System> {
    /// Create a profiling allocator backed by [`System`].
    pub const fn new() -> Self {
        Self { inner: System }
    }
}

impl Default for ProfilingAllocator<System> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> ProfilingAllocator<A> {
    /// Wrap an existing allocator.
    pub const fn from_allocator(inner: A) -> Self {
        Self { inner }
    }
}

// SAFETY: Every method delegates to the wrapped allocator with the original
// layout and pointer arguments, then updates lock-free counters only after the
// allocator call has returned.
unsafe impl<A: GlobalAlloc> GlobalAlloc for ProfilingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: This forwards the caller-provided layout to the wrapped allocator.
        let ptr = unsafe { self.inner.alloc(layout) };
        if allocation_tracking_is_enabled() {
            if ptr.is_null() {
                ALLOC_FAILURES.fetch_add(1, Ordering::Relaxed);
            } else {
                ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
                ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            }
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: This forwards the caller-provided layout to the wrapped allocator.
        let ptr = unsafe { self.inner.alloc_zeroed(layout) };
        if allocation_tracking_is_enabled() {
            if ptr.is_null() {
                ALLOC_ZEROED_FAILURES.fetch_add(1, Ordering::Relaxed);
            } else {
                ALLOC_ZEROED_CALLS.fetch_add(1, Ordering::Relaxed);
                ALLOC_ZEROED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if allocation_tracking_is_enabled() {
            DEALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
            DEALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        // SAFETY: This forwards the caller-provided pointer and layout to the wrapped allocator.
        unsafe { self.inner.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: This forwards the caller-provided pointer, layout, and new size.
        let new_ptr = unsafe { self.inner.realloc(ptr, layout, new_size) };
        if allocation_tracking_is_enabled() {
            if new_ptr.is_null() {
                REALLOC_FAILURES.fetch_add(1, Ordering::Relaxed);
            } else {
                REALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
                REALLOC_OLD_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
                REALLOC_NEW_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
            }
        }
        new_ptr
    }
}

/// Reset global allocation counters.
pub fn reset_allocation_counters() {
    ALLOC_CALLS.store(0, Ordering::Relaxed);
    ALLOC_ZEROED_CALLS.store(0, Ordering::Relaxed);
    ALLOC_FAILURES.store(0, Ordering::Relaxed);
    ALLOC_ZEROED_FAILURES.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    ALLOC_ZEROED_BYTES.store(0, Ordering::Relaxed);
    DEALLOC_CALLS.store(0, Ordering::Relaxed);
    DEALLOC_BYTES.store(0, Ordering::Relaxed);
    REALLOC_CALLS.store(0, Ordering::Relaxed);
    REALLOC_FAILURES.store(0, Ordering::Relaxed);
    REALLOC_OLD_BYTES.store(0, Ordering::Relaxed);
    REALLOC_NEW_BYTES.store(0, Ordering::Relaxed);
}

/// Capture allocation counters for the current profile window.
pub fn allocation_snapshot() -> AllocationSnapshot {
    AllocationSnapshot {
        alloc_calls: ALLOC_CALLS.load(Ordering::Relaxed),
        alloc_zeroed_calls: ALLOC_ZEROED_CALLS.load(Ordering::Relaxed),
        alloc_failures: ALLOC_FAILURES.load(Ordering::Relaxed),
        alloc_zeroed_failures: ALLOC_ZEROED_FAILURES.load(Ordering::Relaxed),
        alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
        alloc_zeroed_bytes: ALLOC_ZEROED_BYTES.load(Ordering::Relaxed),
        dealloc_calls: DEALLOC_CALLS.load(Ordering::Relaxed),
        dealloc_bytes: DEALLOC_BYTES.load(Ordering::Relaxed),
        realloc_calls: REALLOC_CALLS.load(Ordering::Relaxed),
        realloc_failures: REALLOC_FAILURES.load(Ordering::Relaxed),
        realloc_old_bytes: REALLOC_OLD_BYTES.load(Ordering::Relaxed),
        realloc_new_bytes: REALLOC_NEW_BYTES.load(Ordering::Relaxed),
    }
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
        // Keep disabled profiling cheap enough to leave at fine-grained call
        // sites: one relaxed atomic check, then the original block executes.
        if profiler.is_enabled() {
            let _profile_guard = profiler.global_span(name);
            $block
        } else {
            $block
        }
    }};
}

#[inline]
fn average_duration(duration: Duration, count: u64) -> Duration {
    if count == 0 {
        Duration::ZERO
    } else {
        let nanos = duration.as_nanos() / u128::from(count);
        Duration::from_nanos(nanos.try_into().unwrap_or(u64::MAX))
    }
}

#[inline]
fn duration_bucket(duration: Duration) -> usize {
    let mut upper_micros = 1u128;
    let micros = duration.as_micros();
    let mut bucket = 0usize;
    while bucket + 1 < PROFILE_HISTOGRAM_BUCKETS && micros > upper_micros {
        upper_micros <<= 1;
        bucket += 1;
    }
    bucket
}

#[inline]
fn bucket_upper_bound(bucket: usize) -> Duration {
    if bucket >= 63 {
        return Duration::MAX;
    }
    Duration::from_micros(1u64 << bucket)
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
}
