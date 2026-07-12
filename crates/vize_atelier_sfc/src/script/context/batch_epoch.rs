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
}

static BATCH_EPOCH_STATE: Mutex<BatchEpochState> = Mutex::new(BatchEpochState {
    last_epoch: NO_EPOCH,
    active_batches: 0,
});

fn next_batch_epoch(state: &mut BatchEpochState) -> u64 {
    let epoch = state
        .last_epoch
        .checked_add(1)
        .expect("type-resolution batch epoch exhausted");
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

        let epoch = if state.active_batches == 0 {
            NO_EPOCH
        } else {
            // Never return to an older scope's generation after an out-of-order
            // drop; split the validation window with a fresh generation.
            next_batch_epoch(&mut state)
        };
        ACTIVE_BATCH_EPOCH.store(epoch, Ordering::Relaxed);
    }
}

/// Open a scoped type-resolution batch.
///
/// Keep the returned guard alive until all parallel work has joined. While it
/// is alive, imported-type caches skip repeated metadata checks within the
/// current generation. Single compiles do not open a batch and always observe
/// current filesystem metadata.
///
/// Bind the result to a named variable and keep it alive until all parallel
/// work joins. `let _ = begin_type_resolution_batch();` drops it immediately.
pub fn begin_type_resolution_batch() -> TypeResolutionBatchGuard {
    let mut state = BATCH_EPOCH_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let active_batches = state
        .active_batches
        .checked_add(1)
        .expect("too many overlapping type-resolution batches");
    let epoch = next_batch_epoch(&mut state);
    state.active_batches = active_batches;
    ACTIVE_BATCH_EPOCH.store(epoch, Ordering::Relaxed);
    TypeResolutionBatchGuard { epoch }
}

pub(super) fn current_batch_epoch() -> u64 {
    // This generation is only compared for equality; cache contents have their
    // own locks, and batch workers join before their guard is dropped.
    ACTIVE_BATCH_EPOCH.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::{NO_EPOCH, begin_type_resolution_batch, current_batch_epoch};
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn lock_epoch_state() -> std::sync::MutexGuard<'static, ()> {
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
