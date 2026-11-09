//! Caller-supplied roots and importers, canonicalized without a full `realpath`.
//!
//! A transform root or public importer must follow a retargeted directory link
//! on every call, so its identity is never frozen by spelling. One `stat`
//! resolves every link on the way and names the physical directory; that
//! directory's canonical path and listing are then shared by every root in it.

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use super::{TypeSourceSnapshot, source_path};

impl TypeSourceSnapshot {
    /// [`source_path`] for a path a caller supplied, refreshed on every call.
    pub(in crate::script::context) fn public_identity(&self, path: &Path) -> PathBuf {
        self.physical_identity(path)
            .unwrap_or_else(|| source_path(path))
    }

    /// `realpath(path)` is the canonical physical directory followed by the
    /// entry's on-disk name, so an exact listed regular file in the directory
    /// `stat` reached is that path. A directory holds one canonical path per
    /// snapshot; directories cannot be hard-linked. Lexical `..`, relative
    /// spellings, unsaved or linked files and case mismatches keep the full
    /// canonicalization and its lexical fallback.
    #[cfg(unix)]
    fn physical_identity(&self, path: &Path) -> Option<PathBuf> {
        use std::os::unix::fs::MetadataExt;
        if !path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
        {
            return None;
        }
        let name = path.file_name()?;
        let directory = path.parent()?;
        let metadata = std::fs::metadata(directory).ok()?;
        let cell = Arc::clone(
            self.physical_directories
                .lock()
                .ok()?
                .entry((metadata.dev(), metadata.ino()))
                .or_default(),
        );
        let canonical = cell
            .get_or_init(|| std::fs::canonicalize(directory).ok())
            .as_ref()?;
        self.indexed_identity(&canonical.join(name))
    }

    #[cfg(not(unix))]
    fn physical_identity(&self, _path: &Path) -> Option<PathBuf> {
        // Windows canonicalization opens one handle regardless of depth.
        None
    }
}
