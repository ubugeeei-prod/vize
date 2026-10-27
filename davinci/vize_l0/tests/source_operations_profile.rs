//! No libtest/reporting threads inside the allocation window.

#[global_allocator]
static ALLOCATOR: vize_l0::profiler::ProfilingAllocator<std::alloc::System> =
    vize_l0::profiler::ProfilingAllocator::from_allocator(std::alloc::System);

// Keep libtest's listing protocol while excluding its reporting threads from
// the allocator window. Nextest invokes this executable with --list first.
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let ignored = args.iter().any(|arg| arg == "--ignored");
    if args.iter().any(|arg| arg == "--list") {
        if !ignored {
            println!("source_operations_profile: test");
        }
        return;
    }
    if !ignored {
        run();
    }
}

#[cfg(not(feature = "micro-profiling"))]
fn run() {
    let side_effects = std::cell::Cell::new(0);
    vize_l0::micro_count!("test.disabled", {
        side_effects.set(1);
        100
    });
    assert_eq!(
        side_effects.get(),
        0,
        "ordinary builds must not evaluate counters"
    );
    assert_eq!(vize_l0::micro_span!("test.disabled", 41 + 1), 42);
}

#[cfg(feature = "micro-profiling")]
#[expect(
    clippy::unwrap_used,
    reason = "the standalone collector law asserts by panicking"
)]
fn run() {
    use std::alloc::{Layout, alloc, dealloc, realloc};
    use vize_l0::profiler::{allocation_snapshot, global_profiler, micro};

    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let before = allocation_snapshot();
    let expected_line = line!() + 1;
    vize_l0::micro_count!("test.registration", 7);
    let snapshot = micro::snapshot();
    let after = allocation_snapshot();
    assert_eq!(
        before.allocation_calls(),
        after.allocation_calls(),
        "collector must exclude its own storage"
    );
    let entry = snapshot
        .iter()
        .find(|entry| entry.name == "test.registration")
        .unwrap();
    assert_eq!((entry.calls, entry.units), (1, 7));
    assert_eq!(entry.file, file!());
    assert_eq!(entry.module, module_path!());
    assert_eq!(entry.line, expected_line);
    assert_eq!(entry.kind, "counter");
    // The readback was allocated with tracking suppressed, so drop it outside
    // the subsequent exact allocator operation window too.
    profiler.disable();
    drop(snapshot);
    profiler.clear();
    profiler.enable();

    vize_l0::micro_span!("test.source.outer", {
        // SAFETY: valid nonzero layouts; each successful pointer is released
        // once with its current layout, and the old pointer is not reused.
        unsafe {
            let first = alloc(Layout::from_size_align(16, 8).unwrap());
            assert!(!first.is_null());
            let grown = realloc(first, Layout::from_size_align(16, 8).unwrap(), 32);
            assert!(!grown.is_null());
            vize_l0::micro_span!("test.source.inner", {
                dealloc(grown, Layout::from_size_align(32, 8).unwrap());
            });
        }
    });
    let exact = allocation_snapshot();
    profiler.disable();
    assert_eq!((exact.alloc_calls, exact.alloc_bytes), (1, 16));
    assert_eq!(
        (
            exact.realloc_calls,
            exact.realloc_old_bytes,
            exact.realloc_new_bytes
        ),
        (1, 16, 32)
    );
    assert_eq!((exact.dealloc_calls, exact.dealloc_bytes), (1, 32));
    let sites = micro::snapshot();
    let outer_site = sites
        .iter()
        .find(|site| site.name == "test.source.outer")
        .unwrap();
    let outer = profiler.get(outer_site.key).unwrap();
    assert_eq!((outer_site.calls, outer.count), (1, 1));
    assert_eq!((outer.alloc_calls, outer.alloc_bytes), (2, 48));
    assert_eq!(
        outer.total_duration,
        outer.self_duration + outer.child_duration
    );

    profiler.clear();
    profiler.enable();
    let workers: [_; 64] = std::array::from_fn(|_| {
        std::thread::spawn(|| {
            for _ in 0..1_000 {
                vize_l0::micro_count!("test.parallel", 3);
            }
        })
    });
    for worker in workers {
        worker.join().unwrap();
    }
    profiler.disable();
    let sites = micro::snapshot();
    let parallel = sites
        .iter()
        .find(|site| site.name == "test.parallel")
        .unwrap();
    assert_eq!((parallel.calls, parallel.units), (64_000, 192_000));
    assert_eq!(
        sites.len(),
        1,
        "window reset must exclude previous source counts"
    );
    profiler.clear();
    assert!(micro::snapshot().is_empty());
    println!("source operation scopes, allocation exclusion, and 64-worker counts passed");
}
