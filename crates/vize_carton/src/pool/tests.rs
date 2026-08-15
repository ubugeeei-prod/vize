use super::{MAX_IDLE_PER_WORKER, acquire, checked_out, clear, idle};

/// Each test owns the worker's pool state: libtest runs every test on its own
/// thread, so the pool starts empty, and `clear` hands the memory back after.
fn with_empty_pool(body: impl FnOnce()) {
    clear();
    assert_eq!((checked_out(), idle()), (0, 0));
    body();
    clear();
}

/// The point of the pool: the arena that served file N serves file N+1 with a
/// reset in between, so the next file bumps into memory that is already
/// mapped. One small allocation per cycle keeps the arena at one chunk, whose
/// cursor `reset` rewinds — so the second file's first allocation lands at the
/// exact address the first file's did.
#[test]
fn test_the_next_file_reuses_the_same_arena_memory() {
    with_empty_pool(|| {
        let first_address = {
            let arena = acquire();
            arena.alloc_str("file N").as_ptr() as usize
        };
        assert_eq!(idle(), 1);
        let arena = acquire();
        assert_eq!(arena.allocated_bytes(), 0);
        assert_eq!(arena.alloc_str("file M").as_ptr() as usize, first_address);
    });
}

/// A guard is checked out for exactly its own scope, which is what the CLI's
/// per-file assertion reads.
#[test]
fn test_checked_out_tracks_guard_scope() {
    with_empty_pool(|| {
        let outer = acquire();
        assert_eq!(checked_out(), 1);
        {
            let inner = acquire();
            assert_eq!(checked_out(), 2);
            // Nesting takes a second arena rather than sharing one.
            assert_ne!(inner.stamp(), outer.stamp());
        }
        assert_eq!(checked_out(), 1);
        drop(outer);
        assert_eq!((checked_out(), idle()), (0, 2));
    });
}

/// Returning a guard resets its arena, so the generation advances and every
/// stamp taken during that compile is stale — the escape check that turns a
/// cross-file survival into a loud panic instead of recycled bytes.
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "arena-backed value outlived its compile")]
fn test_recycling_a_guard_makes_its_stamps_stale() {
    clear();
    let stamp = {
        let arena = acquire();
        let _ = arena.alloc_str("file N");
        arena.stamp()
    };
    let arena = acquire();
    arena.assert_stamp_current(stamp);
}

/// The counterpart: inside one compile the guard's stamp stays current.
#[test]
fn test_a_live_guard_keeps_its_stamp_current() {
    with_empty_pool(|| {
        let arena = acquire();
        let stamp = arena.stamp();
        let _ = arena.alloc_str("still the same compile");
        arena.assert_stamp_current(stamp);
        assert_eq!(arena.stamp(), stamp);
    });
}

/// The pool is bounded: a burst of nested compiles does not park arenas on the
/// worker forever.
#[test]
fn test_idle_arenas_are_bounded() {
    with_empty_pool(|| {
        let guards: std::vec::Vec<_> = (0..MAX_IDLE_PER_WORKER + 3).map(|_| acquire()).collect();
        assert_eq!(checked_out(), MAX_IDLE_PER_WORKER + 3);
        drop(guards);
        assert_eq!((checked_out(), idle()), (0, MAX_IDLE_PER_WORKER));
    });
}

/// Arenas never cross workers: the pool is per thread, so a second thread
/// starts from an empty pool and returns its arena to its own.
#[test]
fn test_pools_are_per_worker() {
    with_empty_pool(|| {
        let outer = acquire();
        let _ = outer.alloc_str("owned by this worker");
        let observed = std::thread::spawn(|| {
            let idle_at_start = idle();
            let arena = acquire();
            let _ = arena.alloc_str("owned by the other worker");
            let checked_out_inside = checked_out();
            drop(arena);
            (idle_at_start, checked_out_inside, idle(), checked_out())
        })
        .join()
        .expect("the worker thread runs to completion");
        assert_eq!(observed, (0, 1, 1, 0));
        assert_eq!((checked_out(), idle()), (1, 0));
    });
}

/// Parked owned values are dropped when the guard returns its arena, so a
/// pooled worker does not accumulate them file after file.
#[test]
fn test_parked_values_drop_when_the_guard_returns() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DROPS: AtomicUsize = AtomicUsize::new(0);
    struct Parked;
    impl Drop for Parked {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::Relaxed);
        }
    }

    with_empty_pool(|| {
        let before = DROPS.load(Ordering::Relaxed);
        for _ in 0..8 {
            let arena = acquire();
            let parked: &Parked = arena.alloc_owned(Parked);
            let _ = std::hint::black_box(parked);
        }
        assert_eq!(DROPS.load(Ordering::Relaxed), before + 8);
    });
}
