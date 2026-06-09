//! Allocation tracking for profiling.
//!
//! Provides a [`GlobalAlloc`] wrapper that records allocation pressure while
//! profiling is enabled, plus the suppression machinery that keeps the
//! profiler's own bookkeeping from counting itself.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

thread_local! {
    static ALLOCATION_TRACKING_SUPPRESSION: Cell<u32> = const { Cell::new(0) };
}

pub(super) static ALLOCATION_TRACKING_ENABLED: AtomicBool = AtomicBool::new(false);
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
pub(super) struct AllocationTrackingPause;

impl Drop for AllocationTrackingPause {
    fn drop(&mut self) {
        ALLOCATION_TRACKING_SUPPRESSION.with(|depth| {
            depth.set(depth.get().saturating_sub(1));
        });
    }
}

#[inline]
pub(super) fn pause_allocation_tracking() -> AllocationTrackingPause {
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
