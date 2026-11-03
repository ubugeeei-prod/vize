//! Explicit source operations in profiling builds only.
//!
//! Counts aggregate all instrumented Rust threads in this process, never child
//! processes. A thread selects one cache-line-separated shard without hashing.
//! After first registration an event takes a TLS read and two relaxed atomics;
//! More than 32 threads can share a shard, preserving counts but adding contention.
//! Span clocks/self time and allocations use the existing synchronous span
//! stack: child-thread work is not charged as parent-thread allocations.
//! Stop producers before snapshot/clear; a concurrent read is not a coherent
//! point-in-time cut. Counters saturate and reset only between profile windows.

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, Once};

use serde::Serialize;

use super::allocation::pause_allocation_tracking;
use super::{ProfileGuard, global_profiler};

const SHARDS: usize = 32;
static NEXT_THREAD: AtomicUsize = AtomicUsize::new(0);
static SITES: Mutex<Vec<&'static Site>> = Mutex::new(Vec::new());
thread_local! {
    static SHARD: Cell<usize> = Cell::new(NEXT_THREAD.fetch_add(1, Ordering::Relaxed) % SHARDS);
}

#[repr(align(64))]
struct Counts {
    calls: AtomicU64,
    units: AtomicU64,
}

impl Counts {
    const fn new() -> Self {
        Self {
            calls: AtomicU64::new(0),
            units: AtomicU64::new(0),
        }
    }
}

/// One static invocation site, whose identity includes file and line.
pub struct Site {
    key: &'static str,
    name: &'static str,
    file: &'static str,
    module: &'static str,
    line: u32,
    kind: &'static str,
    registered: Once,
    counts: [Counts; SHARDS],
}

impl Site {
    /// Constructed by `micro_count!` and `micro_span!`; never allocates.
    pub const fn new(
        key: &'static str,
        name: &'static str,
        file: &'static str,
        module: &'static str,
        line: u32,
        kind: &'static str,
    ) -> Self {
        Self {
            key,
            name,
            file,
            module,
            line,
            kind,
            registered: Once::new(),
            counts: [const { Counts::new() }; SHARDS],
        }
    }

    fn record(&'static self, units: u64) {
        self.registered.call_once(|| {
            let _pause = pause_allocation_tracking();
            SITES.lock().unwrap_or_else(|p| p.into_inner()).push(self);
        });
        SHARD.with(|shard| {
            let counts = self.counts.get(shard.get()).unwrap_or(&self.counts[0]);
            saturating_add(&counts.calls, 1);
            saturating_add(&counts.units, units);
        });
    }
}

fn saturating_add(counter: &AtomicU64, value: u64) {
    let _ = counter.try_update(Ordering::Relaxed, Ordering::Relaxed, |previous| {
        Some(previous.saturating_add(value))
    });
}

/// Record an event; no timing or profiler map insertion on the hot path.
#[inline]
pub fn count(site: &'static Site, units: u64) {
    if global_profiler().is_enabled() {
        site.record(units);
    }
}

/// Synchronous guard: deliberately cannot migrate across threads/await points.
pub struct Guard {
    _span: Option<ProfileGuard>,
    _same_thread: PhantomData<Rc<()>>,
}

/// Start an inclusive/self-timed source span, carrying actual invocation count.
pub fn span(site: &'static Site) -> Option<Guard> {
    let profiler = global_profiler();
    if !profiler.is_enabled() {
        return None;
    }
    site.record(0);
    Some(Guard {
        _span: profiler.global_span(site.key),
        _same_thread: PhantomData,
    })
}

/// Untruncated source counters; join span `key` to the ordinary span report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// Stable within the exact source SHA; line numbers intentionally change.
    pub key: &'static str,
    pub name: &'static str,
    pub file: &'static str,
    pub module: &'static str,
    pub line: u32,
    pub kind: &'static str,
    pub calls: u64,
    /// Sum of explicitly supplied units; zero for span-only entries.
    pub units: u64,
}

/// Quiescent process snapshot; allocation bookkeeping excludes this readback.
pub fn snapshot() -> Vec<Entry> {
    let _pause = pause_allocation_tracking();
    let sites = SITES.lock().unwrap_or_else(|p| p.into_inner());
    let mut entries = Vec::new();
    for site in sites.iter() {
        let (mut calls, mut units) = (0_u64, 0_u64);
        for counts in &site.counts {
            calls = calls.saturating_add(counts.calls.load(Ordering::Relaxed));
            units = units.saturating_add(counts.units.load(Ordering::Relaxed));
        }
        if calls > 0 {
            entries.push(Entry {
                key: site.key,
                name: site.name,
                file: site.file,
                module: site.module,
                line: site.line,
                kind: site.kind,
                calls,
                units,
            });
        }
    }
    entries.sort_by_key(|entry| entry.key);
    entries
}

pub(super) fn clear() {
    let sites = SITES.lock().unwrap_or_else(|p| p.into_inner());
    for site in sites.iter() {
        for counts in &site.counts {
            counts.calls.store(0, Ordering::Relaxed);
            counts.units.store(0, Ordering::Relaxed);
        }
    }
}
