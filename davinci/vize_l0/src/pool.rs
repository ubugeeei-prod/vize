//! Per-worker arena pool: one arena reused across files (Davinci P1-11).
//!
//! Batch compilation is file-parallel over rayon, and before this the arena
//! was built and torn down per file — every compile started from an empty
//! arena and climbed oxc's chunk ladder again (64 KiB, 128 KiB, 256 KiB …),
//! so a batch paid the growth for every file and held every rung until that
//! file finished. The pool keeps the arena instead: `Allocator::reset`
//! recycles the bytes (oxc keeps the current chunk and returns the rest), and
//! the next file allocates into memory that is already mapped.
//!
//! # Why thread-local
//!
//! Davinci's architecture note fixes the shape — "each rayon worker owns a
//! pooled arena, so arenas are never shared across threads or files" — and a
//! `thread_local!` free list is exactly that with no coordination at all: no
//! mutex on the acquire path (oxc's own `AllocatorPool` locks a shared stack),
//! no worker registry to thread through the CLI's `par_iter`, and it serves
//! every embedder that compiles a file (CLI batch, LSP, napi) rather than only
//! the one call site that could pass a pool handle down. Rayon worker threads
//! outlive the batch, so a worker's arena is reused for every file that worker
//! takes. The cost is that the pool is invisible in signatures; that is what
//! the debug bookkeeping below is for.
//!
//! # Lifetime contract
//!
//! [`acquire`] hands out a [`PooledAllocator`] guard that *owns* its arena for
//! the compile. Arena-backed values borrow the guard, so the borrow checker
//! rejects any value that would outlive it — the reset happens in the guard's
//! `Drop`, after the compile has converted everything it keeps into its owned
//! form. Nothing may hold a guard beyond one file: a guard parked in a cache
//! keeps an arena checked out, which [`checked_out`] reports and the CLI's
//! per-file `debug_assert` turns into a failure.

use std::cell::{Cell, RefCell};
use std::ops::Deref;

use crate::allocator::Allocator;

/// Idle arenas kept per worker.
///
/// One is the steady state — a compile acquires, resets and returns before the
/// next file starts. Nested compiles (a template compile inside an SFC compile
/// that already holds one) push the depth up briefly, so the list keeps a few;
/// beyond that the arena is dropped and its memory returned to the process
/// rather than parked forever on a worker that will not need it again.
const MAX_IDLE_PER_WORKER: usize = 4;

thread_local! {
    /// This worker's idle arenas. Never shared: a `thread_local` is per rayon
    /// worker, and the guard moves the arena out while it is in use.
    static IDLE: RefCell<std::vec::Vec<Allocator>> = const { RefCell::new(std::vec::Vec::new()) };

    /// Guards alive on this worker. Maintained in every build (one `Cell`
    /// update per compile); the file-boundary assertion that reads it is
    /// debug-only.
    static CHECKED_OUT: Cell<usize> = const { Cell::new(0) };
}

/// Takes an arena from this worker's pool, or builds one if the pool is empty.
///
/// The returned guard resets the arena and returns it to the pool when
/// dropped, so the caller's compile owns the whole arena for its duration.
pub fn acquire() -> PooledAllocator {
    // `try_with` throughout: a compile running inside a thread-local destructor
    // (the LSP's shutdown paths do run compiler code) finds the pool already
    // torn down, and building an arena of its own is the right answer there —
    // panicking on the way out is not.
    let allocator = IDLE
        .try_with(|idle| idle.borrow_mut().pop())
        .unwrap_or_default();
    let allocator = allocator.unwrap_or_default();
    // A pooled arena is handed out empty. If it is not, a previous compile's
    // bytes are about to be visible to this one — the cross-file survival the
    // pool exists to make impossible.
    debug_assert_eq!(
        allocator.allocated_bytes(),
        0,
        "pooled arena handed out without a reset"
    );
    let _ = CHECKED_OUT.try_with(|count| count.set(count.get() + 1));
    PooledAllocator { allocator }
}

/// Number of arenas this worker has checked out and not yet returned.
///
/// Zero between files: every compile drops its guard before its owned
/// artifacts cross the file boundary, so a non-zero count at a file boundary
/// means a guard was parked somewhere it must not be (a cache, a static, a
/// `mem::forget`) and an arena is pinned across files.
#[inline]
pub fn checked_out() -> usize {
    CHECKED_OUT.try_with(Cell::get).unwrap_or(0)
}

/// Number of arenas parked on this worker, ready for the next file.
#[inline]
pub fn idle() -> usize {
    IDLE.try_with(|idle| idle.borrow().len()).unwrap_or(0)
}

/// Drops this worker's idle arenas, returning their memory to the process.
///
/// Nothing in a batch needs this — the pool is bounded by
/// `MAX_IDLE_PER_WORKER` arenas per worker and worker exit drops them — but a
/// long-lived process that has finished compiling (an LSP going idle, a test
/// asserting on pool state) can hand the memory back.
pub fn clear() {
    let _ = IDLE.try_with(|idle| idle.borrow_mut().clear());
}

/// An arena borrowed from this worker's pool for one compile.
///
/// Derefs to the [`Allocator`] the compile allocates from; on drop the arena
/// is reset (invalidating every arena-backed value, running every parked
/// destructor) and returned to the pool.
pub struct PooledAllocator {
    /// The borrowed arena. `Drop` swaps in an empty (unreserved, so
    /// allocation-free) arena to move this one back into the pool.
    allocator: Allocator,
}

impl PooledAllocator {
    /// The pooled arena.
    #[inline]
    pub fn allocator(&self) -> &Allocator {
        &self.allocator
    }
}

impl Deref for PooledAllocator {
    type Target = Allocator;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.allocator()
    }
}

impl Drop for PooledAllocator {
    fn drop(&mut self) {
        let mut allocator = core::mem::take(&mut self.allocator);
        // Reset here, not on the next acquire: the memory is recycled as soon
        // as the compile that borrowed it ends, and the arena's generation
        // advances so any stamp taken during that compile reads as stale.
        allocator.reset();
        let _ = CHECKED_OUT.try_with(|count| count.set(count.get().saturating_sub(1)));
        // If the pool is already gone (thread teardown), the arena is simply
        // dropped here — a `Drop` that panicked would abort the process.
        let _ = IDLE.try_with(|idle| {
            let mut idle = idle.borrow_mut();
            if idle.len() < MAX_IDLE_PER_WORKER {
                idle.push(allocator);
            }
        });
    }
}

#[cfg(test)]
mod tests {
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
            let guards: std::vec::Vec<_> =
                (0..MAX_IDLE_PER_WORKER + 3).map(|_| acquire()).collect();
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
}
