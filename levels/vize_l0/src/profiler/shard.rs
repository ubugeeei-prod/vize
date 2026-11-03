//! Total access to the profiler's per-shard locks.

use std::sync::RwLock;

use super::core::PROFILER_SHARDS;

/// The lock of `shard`. Callers pass a `shard_index`, which masks the index
/// below the shard count, so shard 0 is only a total fallback.
pub(super) fn lock<T>(locks: &[RwLock<T>; PROFILER_SHARDS], shard: usize) -> &RwLock<T> {
    locks.get(shard).unwrap_or(&locks[0])
}
