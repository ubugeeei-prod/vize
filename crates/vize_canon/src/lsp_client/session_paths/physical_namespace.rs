//! Physical containment for a whole document batch.
//!
//! `realpath` walks every component of every path. A batch of materialized
//! files shares a handful of directories, so each distinct parent resolves
//! once per batch; a regular-file leaf then lies where its resolved parent
//! puts it. Any other leaf, a symlink in particular, still resolves in full.
//! The memo lives for one batch observation only.

use std::path::{Path, PathBuf};

use vize_l0::FxHashMap;

use super::{is_within_project_namespace, logical_namespace_path};

pub(in crate::lsp_client) struct PhysicalNamespace<'a> {
    root: &'a Path,
    logical_root: Option<PathBuf>,
    parents: FxHashMap<PathBuf, Option<PathBuf>>,
}

impl<'a> PhysicalNamespace<'a> {
    pub(in crate::lsp_client) fn new(root: &'a Path) -> Self {
        Self {
            root,
            logical_root: root.to_str().and_then(|_| logical_namespace_path(root)),
            parents: FxHashMap::default(),
        }
    }

    /// `is_within_project_namespace(path, root)` with the root normalized once.
    pub(in crate::lsp_client) fn contains_logical(&self, path: &Path) -> bool {
        let Some(root) = self.logical_root.as_deref() else {
            return false;
        };
        path.to_str().is_some()
            && logical_namespace_path(path).is_some_and(|path| path.starts_with(root))
    }

    /// `is_within_physical_project_namespace(path, root)`.
    pub(in crate::lsp_client) fn contains_physical(&mut self, path: &Path) -> bool {
        let regular_leaf =
            std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_file());
        let physical = match (regular_leaf, path.parent(), path.file_name()) {
            (true, Some(parent), Some(name)) => self
                .parents
                .entry(parent.to_path_buf())
                .or_insert_with(|| parent.canonicalize().ok())
                .as_ref()
                .map(|parent| parent.join(name)),
            _ => path.canonicalize().ok(),
        };
        physical.is_some_and(|physical| is_within_project_namespace(&physical, self.root))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::lsp_client::session_paths::is_within_physical_project_namespace;

    #[test]
    fn batch_containment_matches_per_path_resolution() {
        let directory = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let root = directory.path().join("project");
        std::fs::create_dir_all(root.join("src/nested")).unwrap();
        std::fs::write(root.join("src/a.ts"), "a").unwrap();
        std::fs::write(root.join("src/nested/b.ts"), "b").unwrap();
        std::fs::write(outside.path().join("c.ts"), "c").unwrap();
        std::fs::create_dir_all(outside.path().join("dir")).unwrap();
        std::fs::write(outside.path().join("dir/d.ts"), "d").unwrap();
        // A linked leaf and a linked directory both leave the namespace.
        std::os::unix::fs::symlink(outside.path().join("c.ts"), root.join("src/c.ts")).unwrap();
        std::os::unix::fs::symlink(outside.path().join("dir"), root.join("src/linked")).unwrap();
        let root = root.canonicalize().unwrap();
        let paths = [
            root.join("src/a.ts"),
            root.join("src/nested/b.ts"),
            root.join("src/c.ts"),
            root.join("src/linked/d.ts"),
            root.join("src/missing.ts"),
            root.join("src/nested/../a.ts"),
        ];
        let mut namespace = PhysicalNamespace::new(&root);
        for path in &paths {
            assert_eq!(
                namespace.contains_physical(path),
                is_within_physical_project_namespace(path, &root),
                "{}",
                path.display()
            );
            assert_eq!(
                namespace.contains_logical(path),
                is_within_project_namespace(path, &root),
                "{}",
                path.display()
            );
        }
        assert!(namespace.contains_physical(&paths[0]));
        assert!(!namespace.contains_physical(&paths[2]));
        assert!(!namespace.contains_physical(&paths[3]));
    }
}
