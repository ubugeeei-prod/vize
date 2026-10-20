//! Session-scoped physical path identities for canonical editor requests.

use std::path::{Path, PathBuf};

use super::ServerState;

impl ServerState {
    /// Resolve one path at most once between filesystem notifications.
    pub(crate) fn canonical_editor_path(&self, path: &Path) -> PathBuf {
        if let Some(cached) = self.canonical_paths.get(path) {
            return cached.clone();
        }
        let physical = vize_carton::path::physical_path(path);
        self.canonical_paths
            .entry(path.to_path_buf())
            .or_insert_with(|| physical.clone());
        physical
    }

    pub(crate) fn invalidate_canonical_editor_paths(&self) {
        self.canonical_paths.clear();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::ServerState;

    #[test]
    fn watched_invalidation_refreshes_a_retargeted_symlink() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        let alias = directory.path().join("alias");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        std::fs::write(first.join("source.ts"), "first").unwrap();
        std::fs::write(second.join("source.ts"), "second").unwrap();
        let first_source = first.canonicalize().unwrap().join("source.ts");
        let second_source = second.canonicalize().unwrap().join("source.ts");
        std::os::unix::fs::symlink(&first, &alias).unwrap();
        let state = ServerState::new();
        let authored = alias.join("source.ts");

        assert_eq!(state.canonical_editor_path(&authored), first_source);
        std::fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(&second, &alias).unwrap();
        assert_eq!(state.canonical_editor_path(&authored), first_source);

        state.invalidate_canonical_editor_paths();
        assert_eq!(state.canonical_editor_path(&authored), second_source);
    }
}
