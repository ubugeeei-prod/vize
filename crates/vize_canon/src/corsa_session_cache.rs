//! Cache key for Corsa project sessions.
//!
//! Foundation for issue #699. Today the LSP spawns a fresh Corsa session per
//! server lifetime. Opening a second workspace folder pays the full init
//! cost again. The cache key here lets the LSP look up an existing session
//! by the `tsconfig.json` it was launched against.
//!
//! The actual cache map and lifecycle (spawn / idle teardown) land in a
//! follow-up. This module ships the key so the cache and its consumers can
//! be developed against a stable shape.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Cache key identifying a Corsa project session.
///
/// Two sessions are interchangeable when they share the same canonical
/// `tsconfig.json` path. Hashing the path (rather than file contents)
/// keeps the key cheap; cache invalidation on `tsconfig.json` change is
/// the consumer's job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorsaSessionKey {
    tsconfig_path: PathBuf,
}

impl CorsaSessionKey {
    /// Build a key from a `tsconfig.json` path. The path is canonicalized
    /// when possible so symlinks resolve to the same session.
    pub fn new(tsconfig_path: impl AsRef<Path>) -> Self {
        let path = tsconfig_path.as_ref().to_path_buf();
        let canonical = std::fs::canonicalize(&path).unwrap_or(path);
        Self {
            tsconfig_path: canonical,
        }
    }

    /// Canonical `tsconfig.json` path used to identify the session.
    pub fn tsconfig_path(&self) -> &Path {
        &self.tsconfig_path
    }
}

impl Hash for CorsaSessionKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tsconfig_path.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::CorsaSessionKey;

    #[test]
    fn equal_keys_compare_equal() {
        let a = CorsaSessionKey::new("./tsconfig.json");
        let b = CorsaSessionKey::new("./tsconfig.json");
        assert_eq!(a, b);
    }

    #[test]
    fn different_paths_yield_different_keys() {
        let a = CorsaSessionKey::new("./tsconfig.json");
        let b = CorsaSessionKey::new("./tsconfig.app.json");
        assert_ne!(a, b);
    }
}
