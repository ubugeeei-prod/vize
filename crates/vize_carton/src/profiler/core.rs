//! Profiler core: timers, nested span guards, and the sharded metric store.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

use super::allocation::{
    ALLOCATION_TRACKING_ENABLED, ThreadAllocationCounters, current_thread_allocation,
    pause_allocation_tracking, reset_allocation_counters,
};
use super::attribution::{AttributedSpanKey, SpanAttribution};
use super::metrics::{CounterMetrics, Metrics, SpanAllocationDelta};

const PROFILER_SHARDS: usize = 32;

thread_local! {
    static PROFILE_STACK: RefCell<std::vec::Vec<ProfileFrame>> = const { RefCell::new(std::vec::Vec::new()) };
}

#[derive(Debug)]
struct ProfileFrame {
    name: &'static str,
    attribution: SpanAttribution,
    start: Instant,
    child_duration: Duration,
    /// This thread's monotone allocation counters at guard start.
    alloc_start: ThreadAllocationCounters,
    /// Allocation growth already attributed to nested child spans.
    child_alloc: ThreadAllocationCounters,
}

/// RAII guard for nested global profiling spans.
#[derive(Debug)]
pub struct ProfileGuard {
    profiler: &'static Profiler,
    active: bool,
}

impl ProfileGuard {
    #[inline]
    fn start(
        profiler: &'static Profiler,
        name: &'static str,
        attribution: SpanAttribution,
    ) -> Self {
        let _allocation_tracking = pause_allocation_tracking();
        // Read after pausing so the frame push below (suppressed) cannot sit
        // between the counter read and the measured window.
        let alloc_start = current_thread_allocation();
        PROFILE_STACK.with(|stack| {
            stack.borrow_mut().push(ProfileFrame {
                name,
                attribution,
                start: Instant::now(),
                child_duration: Duration::ZERO,
                alloc_start,
                child_alloc: ThreadAllocationCounters::ZERO,
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

        // Snapshot the thread's allocation counters before any profiler
        // bookkeeping below (all of it suppressed) runs.
        let alloc_now = current_thread_allocation();
        PROFILE_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let Some(frame) = stack.pop() else {
                return;
            };

            let duration = frame.start.elapsed();
            let alloc_delta = alloc_now.since(frame.alloc_start);
            if let Some(parent) = stack.last_mut() {
                parent.child_duration += duration;
                parent.child_alloc.calls =
                    parent.child_alloc.calls.saturating_add(alloc_delta.calls);
                parent.child_alloc.bytes =
                    parent.child_alloc.bytes.saturating_add(alloc_delta.bytes);
            }
            self.profiler.record_span_sample_attributed_enabled(
                frame.name,
                frame.attribution,
                duration,
                frame.child_duration,
                SpanAllocationDelta {
                    calls: alloc_delta.calls,
                    bytes: alloc_delta.bytes,
                    child_calls: frame.child_alloc.calls,
                    child_bytes: frame.child_alloc.bytes,
                },
            );
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
    /// Metrics for spans carrying a non-empty [`SpanAttribution`], sharded by
    /// operation name like `metrics`. Kept separate so every pre-attribution
    /// consumer of `metrics`, `get`, `all`, and `summary` observes exactly the
    /// historical key space.
    pub(super) attributed: [RwLock<FxHashMap<AttributedSpanKey, Metrics>>; PROFILER_SHARDS],
    /// Non-duration counters by name.
    pub(super) counters: [RwLock<FxHashMap<&'static str, CounterMetrics>>; PROFILER_SHARDS],
    /// Whether profiling is enabled
    enabled: AtomicBool,
}

impl Profiler {
    /// Create a new profiler.
    pub fn new() -> Self {
        Self {
            metrics: std::array::from_fn(|_| RwLock::new(FxHashMap::default())),
            attributed: std::array::from_fn(|_| RwLock::new(FxHashMap::default())),
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
        self.global_span_attributed(name, SpanAttribution::EMPTY)
    }

    /// Start a nested profiling span carrying structured source attribution.
    ///
    /// An empty attribution records under the plain dotted key, exactly like
    /// [`Profiler::global_span`]; a non-empty one records under its own
    /// `key × attribution` bucket and is never merged into the plain key.
    #[inline]
    pub fn global_span_attributed(
        &'static self,
        name: &'static str,
        attribution: SpanAttribution,
    ) -> Option<ProfileGuard> {
        if self.is_enabled() {
            Some(ProfileGuard::start(self, name, attribution))
        } else {
            None
        }
    }

    /// Record a duration and child duration after the caller has already checked profiling.
    #[doc(hidden)]
    pub fn record_sample_enabled(
        &self,
        name: &'static str,
        duration: Duration,
        child_duration: Duration,
    ) {
        self.record_span_sample_attributed_enabled(
            name,
            SpanAttribution::EMPTY,
            duration,
            child_duration,
            SpanAllocationDelta::ZERO,
        );
    }

    /// Record a duration under an attributed span key.
    ///
    /// Companion to [`Profiler::record`] for callers that measure durations
    /// themselves; span guards from [`Profiler::global_span_attributed`] also
    /// capture per-span allocation deltas, which this direct path cannot.
    pub fn record_attributed(
        &self,
        name: &'static str,
        attribution: SpanAttribution,
        duration: Duration,
    ) {
        if !self.is_enabled() {
            return;
        }

        self.record_span_sample_attributed_enabled(
            name,
            attribution,
            duration,
            Duration::ZERO,
            SpanAllocationDelta::ZERO,
        );
    }

    /// Record one span sample after the caller has already checked profiling.
    ///
    /// `ProfileGuard::drop` uses this path after the macro has checked
    /// `is_enabled()`, avoiding another atomic load for every nested span.
    fn record_span_sample_attributed_enabled(
        &self,
        name: &'static str,
        attribution: SpanAttribution,
        duration: Duration,
        child_duration: Duration,
        allocations: SpanAllocationDelta,
    ) {
        let _allocation_tracking = pause_allocation_tracking();
        if attribution.is_empty() {
            let mut metrics = self.metrics_write(Self::shard_index(name));
            metrics.entry(name).or_default().record_span_sample(
                duration,
                child_duration,
                allocations,
            );
        } else {
            let mut metrics = self.attributed_write(Self::shard_index(name));
            metrics
                .entry(AttributedSpanKey { name, attribution })
                .or_default()
                .record_span_sample(duration, child_duration, allocations);
        }
    }

    #[inline]
    pub(super) fn metrics_read(
        &self,
        shard: usize,
    ) -> RwLockReadGuard<'_, FxHashMap<&'static str, Metrics>> {
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
    fn attributed_write(
        &self,
        shard: usize,
    ) -> RwLockWriteGuard<'_, FxHashMap<AttributedSpanKey, Metrics>> {
        self.attributed[shard]
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[inline]
    pub(super) fn counters_write(
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
