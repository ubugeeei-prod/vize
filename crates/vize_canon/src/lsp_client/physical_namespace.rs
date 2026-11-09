//! One synchronization's physical-namespace observations of Canon documents.
//!
//! `canonicalize` resolves every component of every document on every request.
//! Under a canonical project root, a document whose project-relative
//! directories and leaf are not links resolves to itself, so each directory is
//! observed once per batch and each leaf once. Any link, unusual component or
//! non-canonical root keeps the complete resolution.

use std::fs::Metadata;
use std::path::{Component, Path, PathBuf};
use vize_l0::FxHashMap;

use super::session_paths::{
    is_within_namespace_root, is_within_physical_project_namespace, namespace_root,
};

pub(super) struct PhysicalNamespace<'a> {
    root: &'a Path,
    logical_root: Option<PathBuf>,
    canonical_root: bool,
    /// Whether a directory and its ancestors up to the root are not links.
    directories: FxHashMap<PathBuf, bool>,
}

/// What one batch learned about one document path.
pub(super) struct DocumentObservation {
    /// Inside the protocol URI namespace of the root.
    pub(super) logical: bool,
    /// An existing regular file, following links as `Path::is_file` does.
    pub(super) file: bool,
    /// Its physical path stays inside the root.
    pub(super) physical: bool,
    /// The leaf's own `lstat`, when it answered for the whole path.
    pub(super) metadata: Option<Metadata>,
}

impl<'a> PhysicalNamespace<'a> {
    pub(super) fn new(root: &'a Path) -> Self {
        Self {
            root,
            logical_root: namespace_root(root),
            canonical_root: root.canonicalize().is_ok_and(|physical| physical == root),
            directories: FxHashMap::default(),
        }
    }

    /// `is_within_project_namespace(path, root)` with the root converted once.
    pub(super) fn is_logical(&self, path: &Path) -> bool {
        self.logical_root
            .as_deref()
            .is_some_and(|root| is_within_namespace_root(path, root))
    }

    /// The answers of `is_within_project_namespace`, `Path::is_file` and
    /// `is_within_physical_project_namespace` for `path`.
    pub(super) fn observe(&mut self, path: &Path) -> DocumentObservation {
        self.observe_with(path, None)
    }

    /// [`Self::observe`] reusing an already known lexical-namespace answer.
    pub(super) fn observe_with(
        &mut self,
        path: &Path,
        logical: Option<bool>,
    ) -> DocumentObservation {
        let logical = logical.unwrap_or_else(|| self.is_logical(path));
        if let Some(metadata) = self.unlinked_leaf(path) {
            return DocumentObservation {
                logical,
                file: metadata.file_type().is_file(),
                // An unlinked path under a canonical root is its own physical
                // path, so the physical answer is the logical one.
                physical: logical,
                metadata: Some(metadata),
            };
        }
        DocumentObservation {
            logical,
            file: path.is_file(),
            physical: is_within_physical_project_namespace(path, self.root),
            metadata: None,
        }
    }

    /// `lstat` of a project-relative leaf when neither it nor any directory
    /// between it and the canonical root is a link.
    fn unlinked_leaf(&mut self, path: &Path) -> Option<Metadata> {
        if !self.canonical_root {
            return None;
        }
        let relative = path.strip_prefix(self.root).ok()?;
        let mut components = relative.components().peekable();
        components.peek()?;
        if !components.all(|component| matches!(component, Component::Normal(_))) {
            return None;
        }
        let metadata = std::fs::symlink_metadata(path).ok()?;
        if metadata.file_type().is_symlink() || !self.unlinked_directory(path.parent()?) {
            return None;
        }
        Some(metadata)
    }

    fn unlinked_directory(&mut self, directory: &Path) -> bool {
        let mut pending = Vec::new();
        let mut current = directory;
        let mut known = true;
        while current != self.root {
            if let Some(&answer) = self.directories.get(current) {
                known = answer;
                break;
            }
            pending.push(current.to_path_buf());
            let Some(parent) = current.parent() else {
                known = false;
                break;
            };
            current = parent;
        }
        // Resolve from the root outwards so one link marks its descendants.
        for directory in pending.into_iter().rev() {
            known = known
                && std::fs::symlink_metadata(&directory)
                    .is_ok_and(|metadata| metadata.file_type().is_dir());
            self.directories.insert(directory, known);
        }
        known
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn namespace_answers(root: &Path, path: &Path) -> (bool, bool, bool) {
        let observed = PhysicalNamespace::new(root).observe(path);
        (observed.logical, observed.file, observed.physical)
    }

    fn reference_answers(root: &Path, path: &Path) -> (bool, bool, bool) {
        (
            super::super::session_paths::is_within_project_namespace(path, root),
            path.is_file(),
            is_within_physical_project_namespace(path, root),
        )
    }

    #[test]
    fn every_observation_matches_complete_resolution() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("project");
        let outside = temp.path().canonicalize().unwrap().join("outside");
        std::fs::create_dir_all(root.join("src/deep")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(root.join("src/deep/A.vue.ts"), "a").unwrap();
        std::fs::write(outside.join("B.vue.ts"), "b").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("linked")).unwrap();
        std::os::unix::fs::symlink(root.join("src/deep"), root.join("inner")).unwrap();
        std::os::unix::fs::symlink(outside.join("B.vue.ts"), root.join("src/B.vue.ts")).unwrap();
        let paths = [
            root.join("src/deep/A.vue.ts"),
            root.join("src/deep/missing.ts"),
            root.join("src/deep"),
            root.join("linked/B.vue.ts"),
            root.join("inner/A.vue.ts"),
            root.join("src/B.vue.ts"),
            root.join("src/../src/deep/A.vue.ts"),
            outside.join("B.vue.ts"),
            root.clone(),
        ];
        let mut shared = PhysicalNamespace::new(&root);
        for path in &paths {
            let expected = reference_answers(&root, path);
            assert_eq!(
                namespace_answers(&root, path),
                expected,
                "{}",
                path.display()
            );
            let observed = shared.observe(path);
            assert_eq!(
                (observed.logical, observed.file, observed.physical),
                expected,
                "{}",
                path.display()
            );
        }
    }

    #[test]
    fn a_link_above_the_root_keeps_complete_resolution() {
        let temp = tempfile::tempdir().unwrap();
        let physical = temp.path().canonicalize().unwrap().join("physical");
        std::fs::create_dir_all(physical.join("src")).unwrap();
        std::fs::write(physical.join("src/A.vue.ts"), "a").unwrap();
        let logical = temp.path().canonicalize().unwrap().join("logical");
        std::os::unix::fs::symlink(&physical, &logical).unwrap();
        let mut namespace = PhysicalNamespace::new(&logical);
        let path = logical.join("src/A.vue.ts");
        let observed = namespace.observe(&path);
        assert!(observed.metadata.is_none());
        assert_eq!(
            (observed.logical, observed.file, observed.physical),
            reference_answers(&logical, &path)
        );
    }

    #[test]
    fn unlinked_documents_observe_each_directory_once() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("a/b")).unwrap();
        for name in ["one.ts", "two.ts"] {
            std::fs::write(root.join("a/b").join(name), name).unwrap();
        }
        let mut namespace = PhysicalNamespace::new(&root);
        for name in ["one.ts", "two.ts"] {
            let observed = namespace.observe(&root.join("a/b").join(name));
            assert!(observed.logical && observed.file && observed.physical);
            assert!(observed.metadata.is_some());
        }
        assert_eq!(namespace.directories.len(), 2);
    }
}
