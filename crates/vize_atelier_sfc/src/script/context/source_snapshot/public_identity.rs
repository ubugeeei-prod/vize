//! Caller-supplied roots and importers, canonicalized without a full `realpath`.
//!
//! A transform root or public importer must follow a retargeted directory link
//! on every call, so its identity is never frozen by spelling. For a request's
//! snapshot on Linux, one `O_PATH` open walks the root once and
//! `/proc/self/fd` names it: a root that is its own `realpath` needs nothing
//! else, and its held handle keeps its directories resolved for the request.
//! Otherwise one `stat` resolves every link on the way and names the physical
//! directory; that directory's canonical path and probes are then shared by
//! every root in it.

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
    /// entry's on-disk name, so a regular file that is no link in the
    /// directory `stat` reached is that path, and a missing file is that path
    /// as `source_path` spells it. A directory holds one canonical path per
    /// snapshot; directories cannot be hard-linked. Lexical `..`, relative
    /// spellings, linked files and case mismatches keep the full
    /// canonicalization and its lexical fallback.
    #[cfg(unix)]
    fn physical_identity(&self, path: &Path) -> Option<PathBuf> {
        use super::directory_index::Probe;
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
        #[cfg(target_os = "linux")]
        {
            let first = self.held.note_root(path);
            if self.held.enabled()
                && let Some(handle) = super::realpath::canonical_handle(path)
            {
                // The root and every directory above it are their own `realpath`.
                self.note_canonical_directory(directory);
                // Later calls for the same root re-resolve it but hold nothing more.
                if first {
                    self.held.hold(handle);
                }
                return Some(path.to_path_buf());
            }
        }
        let metadata = std::fs::metadata(directory).ok()?;
        let cell = Arc::clone(
            self.physical_directories
                .lock()
                .ok()?
                .entry((metadata.dev(), metadata.ino()))
                .or_default(),
        );
        let canonical = cell
            .get_or_init(|| self.canonicalize_held(directory))
            .as_ref()?;
        match self.probe_in(canonical, name) {
            // `source_path` of a missing file is its canonical parent and name.
            Probe::Found(path) | Probe::Absent(path) => Some(path),
            Probe::Unknown => None,
        }
    }

    #[cfg(not(unix))]
    fn physical_identity(&self, _path: &Path) -> Option<PathBuf> {
        // Windows canonicalization opens one handle regardless of depth.
        None
    }
}
