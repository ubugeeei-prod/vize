//! Realpath cache for one package-shadow or link-planning pass.
//!
//! Preparation repeats the same package roots once per importer edge. The
//! guard keeps those results for the pass and drops them on exit, so the next
//! preparation sees a symlink target that moved.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

struct CacheState {
    paths: FxHashMap<PathBuf, PathBuf>,
    depth: usize,
    canonicalizations: usize,
    membership_probes: usize,
}

impl CacheState {
    fn new() -> Self {
        Self {
            paths: FxHashMap::default(),
            depth: 0,
            canonicalizations: 0,
            membership_probes: 0,
        }
    }
}

thread_local! {
    static CACHE: RefCell<CacheState> = RefCell::new(CacheState::new());
}

pub(super) struct CanonicalPathGuard;

impl CanonicalPathGuard {
    pub(super) fn enter() -> Self {
        CACHE.with(|cache| cache.borrow_mut().depth += 1);
        Self
    }
}

impl Drop for CanonicalPathGuard {
    fn drop(&mut self) {
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.depth = cache.depth.saturating_sub(1);
            if cache.depth == 0 {
                cache.paths.clear();
            }
        });
    }
}

pub(super) fn canonical_path(path: &Path) -> PathBuf {
    let hit = CACHE.with(|cache| {
        let cache = cache.borrow();
        (cache.depth > 0).then(|| cache.paths.get(path).cloned())
    });
    if let Some(hit) = hit {
        if let Some(cached) = hit {
            return cached;
        }
        CACHE.with(|cache| cache.borrow_mut().canonicalizations += 1);
        let canonical = vize_carton::path::canonicalize_non_verbatim(path);
        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.depth == 0 {
                return;
            }
            if canonical.as_path() != path {
                cache
                    .paths
                    .entry(canonical.clone())
                    .or_insert_with(|| canonical.clone());
            }
            cache.paths.insert(path.to_path_buf(), canonical.clone());
        });
        return canonical;
    }
    vize_carton::path::canonicalize_non_verbatim(path)
}

pub(super) fn record_membership_probes(count: usize) {
    if count == 0 {
        return;
    }
    CACHE.with(|cache| cache.borrow_mut().membership_probes += count);
}

#[cfg(test)]
pub(super) fn reset_prep_counters() {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.canonicalizations = 0;
        cache.membership_probes = 0;
    });
}

#[cfg(test)]
pub(super) fn prep_counters() -> (usize, usize) {
    CACHE.with(|cache| {
        let cache = cache.borrow();
        (cache.canonicalizations, cache.membership_probes)
    })
}
