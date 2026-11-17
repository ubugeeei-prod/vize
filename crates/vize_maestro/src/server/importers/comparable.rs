//! The comparable spelling of indexed dependency paths.

use std::path::{Component, Path, PathBuf};

use vize_l0::FxHashMap;

/// Canonical spellings of the directories one index update has resolved.
///
/// An update canonicalizes the parent directory of every dependency it
/// indexes, and a missing parent sends it to the grandparent and further up.
/// One import can contribute many probe paths, so an edit of an SFC with a
/// few dozen imports used to `realpath` the same few directories hundreds of
/// times. The memo lives for one update only, so each edit still observes the
/// filesystem as it is at that edit.
#[derive(Default)]
pub(super) struct ComparablePaths {
    directories: FxHashMap<PathBuf, Option<PathBuf>>,
}

impl ComparablePaths {
    /// `path` with its deepest existing ancestor canonicalized and the rest
    /// appended. Without an existing ancestor, the lexically normalized path.
    pub(super) fn get(&mut self, path: &Path) -> PathBuf {
        let mut ancestor = path;
        let mut unresolved = Vec::new();
        while let Some(parent) = ancestor.parent() {
            if let Some(name) = ancestor.file_name() {
                unresolved.push(name.to_os_string());
            }
            if let Some(mut canonical) = self.canonical_directory(parent) {
                for component in unresolved.into_iter().rev() {
                    canonical.push(component);
                }
                return canonical;
            }
            ancestor = parent;
        }
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    normalized.pop();
                }
                _ => normalized.push(component.as_os_str()),
            }
        }
        normalized
    }

    fn canonical_directory(&mut self, directory: &Path) -> Option<PathBuf> {
        if let Some(canonical) = self.directories.get(directory) {
            return canonical.clone();
        }
        let canonical = std::fs::canonicalize(directory).ok();
        self.directories
            .insert(directory.to_path_buf(), canonical.clone());
        canonical
    }
}

/// [`ComparablePaths::get`] for a single path.
pub(super) fn comparable_path(path: &Path) -> PathBuf {
    ComparablePaths::default().get(path)
}
