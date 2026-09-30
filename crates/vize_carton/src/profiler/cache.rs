//! Cache hit/miss statistics.

use std::sync::atomic::{AtomicU64, Ordering};

/// Cache statistics.
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: AtomicU64,
    /// Number of cache misses
    pub misses: AtomicU64,
    /// Total entries in cache
    pub entries: AtomicU64,
}

impl CacheStats {
    /// Create new cache stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a cache hit.
    #[inline]
    pub fn hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss.
    #[inline]
    pub fn miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Update entry count.
    #[inline]
    pub fn set_entries(&self, count: u64) {
        self.entries.store(count, Ordering::Relaxed);
    }

    /// Get the hit rate (0.0 - 1.0).
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Reset statistics.
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {} hits, {} misses ({:.1}% hit rate), {} entries",
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            self.hit_rate() * 100.0,
            self.entries.load(Ordering::Relaxed)
        )
    }
}
