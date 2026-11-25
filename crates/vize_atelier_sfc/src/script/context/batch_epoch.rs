use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// Sentinel for single-file/HMR resolution outside a batch. It never equals a
/// live epoch, so every lookup revalidates its filesystem metadata.
pub(super) const NO_EPOCH: u64 = 0;

/// The generation shared by active batch scopes. Cache hits only pay this one
/// relaxed load; lifecycle bookkeeping stays off the hot path.
static ACTIVE_BATCH_EPOCH: AtomicU64 = AtomicU64::new(NO_EPOCH);

#[derive(Debug)]
struct BatchEpochState {
    /// Generations are never recycled, including across overlapping scopes.
    last_epoch: u64,
    active_batches: usize,
    /// Open windows a host began, which compile calls join.
    held_windows: usize,
}

static BATCH_EPOCH_STATE: Mutex<BatchEpochState> = Mutex::new(BatchEpochState {
    last_epoch: NO_EPOCH,
    active_batches: 0,
    held_windows: 0,
});

fn next_batch_epoch(state: &mut BatchEpochState) -> u64 {
    // Once the generations run out, fall back to the no-batch sentinel: every
    // lookup then revalidates its metadata, which is always correct.
    let Some(epoch) = state.last_epoch.checked_add(1) else {
        return NO_EPOCH;
    };
    state.last_epoch = epoch;
    epoch
}

/// A scoped filesystem-stability window for imported-type resolution.
///
/// Dropping the guard ends the batch. Normal returns, `?` error exits, and
/// unwinding therefore all close the window. The guard is neither cloneable
/// nor reusable.
#[must_use = "hold the guard for the full type-resolution batch"]
#[derive(Debug)]
pub struct TypeResolutionBatchGuard {
    epoch: u64,
    scope: Scope,
}

/// How a guard takes part in the generation it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    /// A window a host began: compile calls inside it join it.
    Held,
    /// A compile call's own window, begun with no host window open.
    Own,
    /// A compile call inside a host window, sharing its generation.
    Joined,
}

impl TypeResolutionBatchGuard {
    /// The unique generation issued when this batch scope began.
    #[must_use]
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }
}

impl Drop for TypeResolutionBatchGuard {
    fn drop(&mut self) {
        let mut state = BATCH_EPOCH_STATE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        debug_assert!(
            state.active_batches > 0,
            "type-resolution batch guard dropped without an active batch"
        );
        state.active_batches = state.active_batches.saturating_sub(1);
        if self.scope == Scope::Held {
            state.held_windows = state.held_windows.saturating_sub(1);
        }

        let epoch = if state.active_batches == 0 {
            NO_EPOCH
        } else if self.scope == Scope::Joined {
            // A joined call ends inside the window it joined, which goes on.
            return;
        } else {
            // Never return to an older scope's generation after an out-of-order
            // drop; split the validation window with a fresh generation.
            next_batch_epoch(&mut state)
        };
        ACTIVE_BATCH_EPOCH.store(epoch, Ordering::Relaxed);
        if epoch == NO_EPOCH {
            // No window holds the files stable any more, so nothing keeps what
            // the closed batches read. The state lock is still held: a batch
            // that begins now starts after its predecessors' modules are gone.
            super::external_types::retire_shared_modules();
        }
    }
}

/// Open a scoped type-resolution batch.
///
/// Keep the returned guard alive until all parallel work has joined. While it
/// is alive, the compiles of the current generation share one view of the
/// imported type modules: each file is read once and each import resolved
/// once, whichever worker reaches it first.
///
/// The batch is process-wide: a compile on any thread joins the generation
/// that is current while it runs, whoever opened it. A compile observes the
/// current files on its own only while no batch is open.
///
/// This is the host's window: whoever opens it promises that the files hold
/// still until it ends. Batch compile calls made inside it join it with
/// [`join_type_resolution_batch`] instead of starting over.
///
/// Bind the result to a named variable and keep it alive until all parallel
/// work joins. `let _ = begin_type_resolution_batch();` drops it immediately.
pub fn begin_type_resolution_batch() -> TypeResolutionBatchGuard {
    let mut state = BATCH_EPOCH_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.held_windows = state.held_windows.saturating_add(1);
    begin(&mut state, Scope::Held)
}

/// Open the type-resolution batch of one compile call.
///
/// Inside a window a host began with [`begin_type_resolution_batch`], the
/// call joins that window: the host holds the files still for all of it, so
/// what an earlier call of the window read is still what is on disk, and the
/// imported type modules are read once per window instead of once per call.
/// With no host window open, the call begins a window of its own that ends
/// with it, and calls that overlap each get a generation of their own.
pub fn join_type_resolution_batch() -> TypeResolutionBatchGuard {
    let mut state = BATCH_EPOCH_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let current = ACTIVE_BATCH_EPOCH.load(Ordering::Relaxed);
    if state.held_windows == 0 || current == NO_EPOCH {
        return begin(&mut state, Scope::Own);
    }
    state.active_batches = state.active_batches.saturating_add(1);
    TypeResolutionBatchGuard {
        epoch: current,
        scope: Scope::Joined,
    }
}

fn begin(state: &mut BatchEpochState, scope: Scope) -> TypeResolutionBatchGuard {
    let active_batches = state.active_batches.saturating_add(1);
    let epoch = next_batch_epoch(state);
    state.active_batches = active_batches;
    ACTIVE_BATCH_EPOCH.store(epoch, Ordering::Relaxed);
    TypeResolutionBatchGuard { epoch, scope }
}

pub(super) fn current_batch_epoch() -> u64 {
    // This generation is only compared for equality; cache contents have their
    // own locks, and batch workers join before their guard is dropped.
    ACTIVE_BATCH_EPOCH.load(Ordering::Relaxed)
}

#[cfg(test)]
pub(super) mod tests {
    use super::{
        NO_EPOCH, begin_type_resolution_batch, current_batch_epoch, join_type_resolution_batch,
    };
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Held by every test that opens a batch or reads what a batch's end
    /// retires: both are process-wide.
    pub(in crate::script::context) fn lock_epoch_state() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn nested_scopes_never_reuse_epochs() {
        let _lock = lock_epoch_state();
        assert_eq!(current_batch_epoch(), NO_EPOCH);

        let outer = begin_type_resolution_batch();
        let inner = begin_type_resolution_batch();
        assert!(inner.epoch() > outer.epoch());
        assert_eq!(current_batch_epoch(), inner.epoch());

        let inner_epoch = inner.epoch();
        drop(inner);
        let post_inner_epoch = current_batch_epoch();
        assert!(post_inner_epoch > inner_epoch);
        drop(outer);
        assert_eq!(current_batch_epoch(), NO_EPOCH);

        let next = begin_type_resolution_batch();
        assert!(next.epoch() > post_inner_epoch);
        drop(next);
        assert_eq!(current_batch_epoch(), NO_EPOCH);
    }

    #[test]
    fn compile_calls_join_the_window_a_host_holds() {
        let _lock = lock_epoch_state();
        let host = begin_type_resolution_batch();
        for _ in 0..3 {
            let call = join_type_resolution_batch();
            assert_eq!(call.epoch(), host.epoch());
            drop(call);
            assert_eq!(current_batch_epoch(), host.epoch());
        }
        drop(host);
        assert_eq!(current_batch_epoch(), NO_EPOCH);
    }

    #[test]
    fn compile_calls_without_a_host_window_never_share_one() {
        let _lock = lock_epoch_state();
        let first = join_type_resolution_batch();
        let overlapping = join_type_resolution_batch();
        assert!(overlapping.epoch() > first.epoch());
        drop(first);
        assert!(current_batch_epoch() > overlapping.epoch());
        drop(overlapping);
        assert_eq!(current_batch_epoch(), NO_EPOCH);

        let next = join_type_resolution_batch();
        drop(next);
        assert_eq!(current_batch_epoch(), NO_EPOCH);
    }

    #[test]
    fn a_call_outliving_its_host_window_moves_to_a_new_generation() {
        let _lock = lock_epoch_state();
        let host = begin_type_resolution_batch();
        let call = join_type_resolution_batch();
        let window = host.epoch();
        drop(host);
        assert!(current_batch_epoch() > window);
        // The host is gone: the next call cannot join it.
        let later = join_type_resolution_batch();
        assert!(later.epoch() > window);
        drop(later);
        drop(call);
        assert_eq!(current_batch_epoch(), NO_EPOCH);
    }

    #[test]
    fn concurrent_scopes_handle_out_of_order_drops() {
        let _lock = lock_epoch_state();
        let outer = begin_type_resolution_batch();
        let (started_tx, started_rx) = std::sync::mpsc::sync_channel(0);
        let (finish_tx, finish_rx) = std::sync::mpsc::sync_channel(0);

        let concurrent = std::thread::spawn(move || {
            let batch = begin_type_resolution_batch();
            started_tx.send(batch.epoch()).unwrap();
            finish_rx.recv().unwrap();
            drop(batch);
        });
        let concurrent_epoch = started_rx.recv().unwrap();
        assert!(concurrent_epoch > outer.epoch());

        drop(outer);
        assert!(current_batch_epoch() > concurrent_epoch);
        finish_tx.send(()).unwrap();
        concurrent.join().unwrap();
        assert_eq!(current_batch_epoch(), NO_EPOCH);
    }

    #[test]
    fn guard_closes_on_error_and_panic_exits() {
        fn returns_error() -> Result<(), ()> {
            let _batch = begin_type_resolution_batch();
            Err(())
        }

        let _lock = lock_epoch_state();
        assert!(returns_error().is_err());
        assert_eq!(current_batch_epoch(), NO_EPOCH);

        let result = std::panic::catch_unwind(|| {
            let _batch = begin_type_resolution_batch();
            panic!("exercise batch-guard unwind");
        });
        assert!(result.is_err());
        assert_eq!(current_batch_epoch(), NO_EPOCH);
    }
}
