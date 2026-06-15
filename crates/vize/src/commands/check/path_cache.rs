//! Run-scoped memoization of `Path::canonicalize` results.

use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

/// Run-scoped cache of `Path::canonicalize` results.
///
/// Canonicalization hits the filesystem on every call, and the explicit-subset
/// check path canonicalizes the same paths repeatedly: once to build the
/// reported-file set, once per diagnostic when filtering, and once per
/// resolution while walking transitive relative imports. Sharing one per-run
/// cache across those sites keeps each unique path at a single syscall.
///
/// Paths that fail to canonicalize (e.g. not on disk) memoize their original
/// spelling, matching the previous per-call `unwrap_or` fallback.
#[derive(Default)]
pub(super) struct CanonicalPathCache {
    cache: FxHashMap<PathBuf, PathBuf>,
}

impl CanonicalPathCache {
    /// Canonicalize `path`, falling back to the original spelling when the
    /// path cannot be resolved on disk.
    pub(super) fn canonicalize(&mut self, path: &Path) -> PathBuf {
        if let Some(cached) = self.cache.get(path) {
            return cached.clone();
        }
        let canonical = vize_carton::path::canonicalize_non_verbatim(path);
        self.cache.insert(path.to_path_buf(), canonical.clone());
        canonical
    }
}

#[cfg(test)]
mod tests {
    use super::CanonicalPathCache;

    #[test]
    fn canonicalizes_existing_paths_and_memoizes_missing_ones() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("a.ts");
        std::fs::write(&file, "export const a = 1;\n").unwrap();

        let mut cache = CanonicalPathCache::default();
        let canonical = cache.canonicalize(&file);
        assert_eq!(
            canonical,
            vize_carton::path::canonicalize_non_verbatim(&file)
        );
        // Cached lookups return the same result.
        assert_eq!(cache.canonicalize(&file), canonical);

        let missing = temp.path().join("missing.ts");
        assert_eq!(cache.canonicalize(&missing), missing);
        assert_eq!(cache.canonicalize(&missing), missing);
    }
}
