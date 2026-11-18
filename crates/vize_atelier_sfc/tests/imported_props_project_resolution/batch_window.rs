//! Serializes the one held type-resolution batch against edit-revalidation tests.

use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A type-resolution batch is process-wide: while one test holds it, caches
/// validated in its generation skip metadata checks. Tests that edit files
/// after a resolution share this lock; the test holding a batch takes it alone.
static TYPE_RESOLUTION_BATCH: RwLock<()> = RwLock::new(());

pub(crate) fn revalidating_edits() -> RwLockReadGuard<'static, ()> {
    TYPE_RESOLUTION_BATCH
        .read()
        .unwrap_or_else(PoisonError::into_inner)
}

pub(crate) fn held_batch() -> RwLockWriteGuard<'static, ()> {
    TYPE_RESOLUTION_BATCH
        .write()
        .unwrap_or_else(PoisonError::into_inner)
}
