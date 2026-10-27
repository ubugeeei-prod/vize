//! Exact overlapping-thread allocation attribution; no backend or filesystem.
#![expect(clippy::expect_used, reason = "test failures panic")]
#![expect(
    clippy::disallowed_types,
    reason = "the concurrency law shares atomic handshakes"
)]

use std::alloc::System;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use vize_l0::profiler::{ProfilingAllocator, global_profiler};

#[global_allocator]
static ALLOCATOR: ProfilingAllocator<System> = ProfilingAllocator::from_allocator(System);

#[derive(Default)]
struct Handshake {
    ready: AtomicU8,
    start: AtomicBool,
    entered: AtomicU8,
    allocated: AtomicU8,
    release: AtomicBool,
    finished: AtomicU8,
    stop: AtomicBool,
}

fn wait_mask(value: &AtomicU8, mask: u8) {
    while value.load(Ordering::Acquire) & mask != mask {
        std::thread::yield_now();
    }
}

fn wait_flag(value: &AtomicBool) {
    while !value.load(Ordering::Acquire) {
        std::thread::yield_now();
    }
}

fn worker(
    state: Arc<Handshake>,
    bit: u8,
    outer: &'static str,
    inner: &'static str,
    bytes: usize,
    child_bytes: usize,
) {
    std::thread::yield_now();
    state.ready.fetch_or(bit, Ordering::Release);
    wait_flag(&state.start);
    vize_l0::profile!(outer, {
        state.entered.fetch_or(bit, Ordering::Release);
        wait_mask(&state.entered, 3);
        let own = std::hint::black_box(Vec::<u8>::with_capacity(bytes));
        let child = vize_l0::profile!(
            inner,
            std::hint::black_box(Vec::<u8>::with_capacity(child_bytes))
        );
        drop(child);
        state.allocated.fetch_or(bit, Ordering::Release);
        wait_flag(&state.release);
        drop(own);
    });
    state.finished.fetch_or(bit, Ordering::Release);
    wait_flag(&state.stop);
}

#[test]
fn overlapping_threads_keep_allocations_and_nested_self_costs_separate() {
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    let state = Arc::new(Handshake::default());
    let left = {
        let state = Arc::clone(&state);
        std::thread::spawn(move || {
            worker(
                state,
                1,
                "profile.alloc.a.outer",
                "profile.alloc.a.inner",
                1024,
                256,
            );
        })
    };
    let right = {
        let state = Arc::clone(&state);
        std::thread::spawn(move || {
            worker(
                state,
                2,
                "profile.alloc.b.outer",
                "profile.alloc.b.inner",
                4096,
                128,
            );
        })
    };
    wait_mask(&state.ready, 3);
    profiler.enable();
    state.start.store(true, Ordering::Release);
    wait_mask(&state.allocated, 3);
    vize_l0::profile!("profile.alloc.noise", {
        drop(std::hint::black_box(Vec::<u8>::with_capacity(8192)));
    });
    state.release.store(true, Ordering::Release);
    wait_mask(&state.finished, 3);
    profiler.disable();
    state.stop.store(true, Ordering::Release);
    left.join().expect("left worker completes");
    right.join().expect("right worker completes");

    for (name, calls, bytes, self_calls, self_bytes) in [
        ("profile.alloc.a.outer", 2, 1280, 1, 1024),
        ("profile.alloc.a.inner", 1, 256, 1, 256),
        ("profile.alloc.b.outer", 2, 4224, 1, 4096),
        ("profile.alloc.b.inner", 1, 128, 1, 128),
        ("profile.alloc.noise", 1, 8192, 1, 8192),
    ] {
        let metric = profiler.get(name).expect("every scope is recorded");
        println!(
            "allocation_scope={}",
            serde_json::json!({
                "name": name,
                "count": metric.count,
                "calls": metric.alloc_calls,
                "bytes": metric.alloc_bytes,
                "selfCalls": metric.self_alloc_calls,
                "selfBytes": metric.self_alloc_bytes,
            })
        );
        assert_eq!(metric.count, 1, "{name}");
        assert_eq!(
            (metric.alloc_calls, metric.alloc_bytes),
            (calls, bytes),
            "{name} includes only its own thread and nested scopes"
        );
        assert_eq!(
            (metric.self_alloc_calls, metric.self_alloc_bytes),
            (self_calls, self_bytes),
            "{name} excludes its recorded children"
        );
        assert_eq!(
            metric.total_duration,
            metric.self_duration + metric.child_duration,
            "{name}"
        );
    }
}

#[cfg(not(feature = "micro-profiling"))]
#[test]
fn normal_guard_preserves_its_send_contract() {
    fn require_send<T: Send>() {}
    require_send::<vize_l0::profiler::ProfileGuard>();
}
