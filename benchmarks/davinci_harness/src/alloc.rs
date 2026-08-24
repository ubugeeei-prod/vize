//! Allocation counting for Davinci benches.
//!
//! [`CountingAllocator`] wraps a concrete allocator behind [`GlobalAlloc`] and
//! keeps three process-global counters: allocation-like calls, live heap
//! bytes, and their high-water mark. The default inner allocator is mimalloc
//! so benches run on the same allocator as the shipped `vize` binary
//! (`crates/vize/src/main.rs`).
//!
//! Relation to `vize_s0::profiler::ProfilingAllocator`: carton counts
//! per-window allocation traffic for the profiler behind an opt-in flag and
//! tracks no live/peak bytes. The Davinci budgets gate on peak footprint, so
//! this harness keeps its own always-on counter set; P0-11 aligns the two
//! exporters. Counting costs a handful of relaxed atomic operations per
//! allocation and applies identically to baseline and candidate runs, so
//! budget comparisons stay like-for-like.
//!
//! Installation is opt-in through the [`crate::main!`] macro, which places the
//! allocator as `#[global_allocator]` and calls [`mark_installed`] so
//! [`measure`] knows the counters are live.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::alloc::{GlobalAlloc, Layout, System};

/// Allocation-like calls (`alloc` + `alloc_zeroed` + successful `realloc`).
static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
/// Heap bytes currently live through the counting allocator.
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
/// High-water mark of [`LIVE_BYTES`]; reset to the current level at window start.
static PEAK_BYTES: AtomicU64 = AtomicU64::new(0);
/// Set by [`mark_installed`] once the counting allocator is the global allocator.
static INSTALLED: AtomicBool = AtomicBool::new(false);

/// [`GlobalAlloc`] wrapper that counts calls and live/peak bytes.
///
/// The default inner allocator is [`mimalloc::MiMalloc`], matching the
/// production `vize` binary; [`CountingAllocator::system`] exists for tests
/// that must stay on the platform allocator (for example under miri).
#[derive(Debug)]
pub struct CountingAllocator<A = mimalloc::MiMalloc> {
    inner: A,
}

impl CountingAllocator<mimalloc::MiMalloc> {
    /// Counting allocator over mimalloc - the production configuration.
    pub const fn mimalloc() -> Self {
        Self {
            inner: mimalloc::MiMalloc,
        }
    }
}

impl CountingAllocator<System> {
    /// Counting allocator over [`System`], for allocator-agnostic tests.
    pub const fn system() -> Self {
        Self { inner: System }
    }
}

impl<A> CountingAllocator<A> {
    /// Wrap an arbitrary allocator.
    pub const fn from_allocator(inner: A) -> Self {
        Self { inner }
    }
}

#[inline]
fn on_alloc(size: usize) {
    ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
    let live = LIVE_BYTES.fetch_add(size as u64, Ordering::Relaxed) + size as u64;
    PEAK_BYTES.fetch_max(live, Ordering::Relaxed);
}

#[inline]
fn on_dealloc(size: usize) {
    LIVE_BYTES.fetch_sub(size as u64, Ordering::Relaxed);
}

// SAFETY: Every method forwards the caller's arguments to the wrapped
// allocator unchanged and only updates lock-free counters after the inner
// allocator call has returned, so the wrapper adds no aliasing or layout
// behavior of its own.
unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwards the caller-provided layout unchanged.
        let ptr = unsafe { self.inner.alloc(layout) };
        if !ptr.is_null() {
            on_alloc(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwards the caller-provided layout unchanged.
        let ptr = unsafe { self.inner.alloc_zeroed(layout) };
        if !ptr.is_null() {
            on_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        on_dealloc(layout.size());
        // SAFETY: forwards the caller-provided pointer and layout unchanged.
        unsafe { self.inner.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: forwards the caller-provided pointer, layout, and size unchanged.
        let new_ptr = unsafe { self.inner.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            on_dealloc(layout.size());
            on_alloc(new_size);
        }
        new_ptr
    }
}

/// Counter values at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocStats {
    /// Allocation-like calls since process start.
    pub calls: u64,
    /// Heap bytes currently live.
    pub live_bytes: u64,
    /// High-water mark of live bytes since the last window reset.
    pub peak_bytes: u64,
}

/// Read the current counter values.
pub fn stats() -> AllocStats {
    AllocStats {
        calls: ALLOC_CALLS.load(Ordering::Relaxed),
        live_bytes: LIVE_BYTES.load(Ordering::Relaxed),
        peak_bytes: PEAK_BYTES.load(Ordering::Relaxed),
    }
}

/// Record that the counting allocator is installed as the global allocator.
///
/// Called by [`crate::main!`]; without it, [`measure`] reports `None` so a
/// harness misconfiguration surfaces as null metrics instead of silent zeros.
pub fn mark_installed() {
    INSTALLED.store(true, Ordering::Relaxed);
}

/// Whether [`mark_installed`] has run in this process.
pub fn is_installed() -> bool {
    INSTALLED.load(Ordering::Relaxed)
}

/// Allocation metrics for one measured window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocMetrics {
    /// Allocation-like calls during the window.
    pub calls: u64,
    /// Peak live bytes over the window, relative to the live bytes at window
    /// start (transient footprint of the routine, not the process).
    pub peak_bytes_over_start: u64,
}

/// Run `routine` once and report its allocation metrics, or `None` when the
/// counting allocator is not installed as the global allocator.
///
/// Window semantics: the peak counter is reset to the current live-byte level
/// at window start, so `peak_bytes_over_start` is the routine's transient
/// footprint. Counters are process-global; allocations from other threads
/// during the window are attributed to it. Benches run the routine on one
/// thread, so in practice the window covers the routine plus anything it
/// spawns itself.
pub fn measure<T>(routine: impl FnOnce() -> T) -> Option<AllocMetrics> {
    let (value, metrics) = measure_returning(routine);
    drop(value);
    metrics
}

/// Like [`measure`], but hands the routine's value back so callers that need
/// it (stage windows) can keep it alive past the measured window. Metrics are
/// captured before the value drops either way, so the two entry points report
/// identical numbers.
pub fn measure_returning<T>(routine: impl FnOnce() -> T) -> (T, Option<AllocMetrics>) {
    if !is_installed() {
        return (routine(), None);
    }
    let live_start = LIVE_BYTES.load(Ordering::Relaxed);
    PEAK_BYTES.store(live_start, Ordering::Relaxed);
    let calls_start = ALLOC_CALLS.load(Ordering::Relaxed);
    let value = routine();
    let metrics = AllocMetrics {
        calls: ALLOC_CALLS
            .load(Ordering::Relaxed)
            .saturating_sub(calls_start),
        peak_bytes_over_start: PEAK_BYTES
            .load(Ordering::Relaxed)
            .saturating_sub(live_start),
    };
    (value, Some(metrics))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One scripted allocation sequence with exact expected counter deltas.
    ///
    /// This is deliberately the only test that touches the process-global
    /// counters, so parallel test execution cannot perturb the exact
    /// assertions. The global allocator is *not* the counting allocator under
    /// `cargo test`; the sequence drives a [`CountingAllocator`] instance
    /// directly, which moves the same statics the installed allocator would.
    #[test]
    fn scripted_sequence_has_exact_counter_deltas() {
        mark_installed();
        let allocator = CountingAllocator::system();
        let layout_256 = Layout::from_size_align(256, 8).expect("static layout");
        let layout_512 = Layout::from_size_align(512, 8).expect("static layout");

        let metrics = measure(|| {
            // SAFETY: every pointer is allocated by `allocator` in this block
            // with the layout it is later reallocated/deallocated with.
            unsafe {
                let first = allocator.alloc(layout_256);
                assert!(!first.is_null());
                let second = allocator.alloc_zeroed(layout_256);
                assert!(!second.is_null());
                allocator.dealloc(first, layout_256);
                let grown = allocator.realloc(second, layout_256, 512);
                assert!(!grown.is_null());
                allocator.dealloc(grown, layout_512);
            }
        })
        .expect("mark_installed ran, so measure must report metrics");

        // alloc + alloc_zeroed + realloc = 3 allocation-like calls.
        assert_eq!(metrics.calls, 3);
        // Live-byte trace relative to window start: 256, 512, 256, 512, 0.
        assert_eq!(metrics.peak_bytes_over_start, 512);
    }
}
