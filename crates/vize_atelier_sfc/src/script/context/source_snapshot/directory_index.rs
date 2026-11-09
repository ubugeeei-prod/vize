//! Canonical identities for files a probe already found, without `realpath`.
//!
//! `realpath` walks every path component: one `getattrlist` per component on
//! macOS, a `readlink` per component on Linux (a 9P round trip each on a WSL
//! `/mnt/c` drive), and an open handle plus `GetFinalPathNameByHandleW` on
//! Windows. A snapshot resolves many candidates in few directories, so it
//! canonicalizes each directory once and lists its regular files once.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use vize_carton::FxHashSet;

use super::TypeSourceSnapshot;

/// Directories with more entries are not listed: one `stat` of the probed
/// file is cheaper than reading a large package store to answer it.
const MAX_INDEXED_ENTRIES: usize = 4096;

/// A physical directory as this snapshot first listed it.
#[derive(Debug)]
pub(in crate::script::context) struct IndexedDirectory {
    canonical: PathBuf,
    /// Entries that are regular files themselves, never symbolic links.
    files: FxHashSet<OsString>,
}

impl IndexedDirectory {
    fn read(directory: &Path) -> Option<Self> {
        Self::list(std::fs::canonicalize(directory).ok()?)
    }

    /// List a directory whose canonical path is already known.
    fn list(canonical: PathBuf) -> Option<Self> {
        let mut files = FxHashSet::default();
        for (index, entry) in std::fs::read_dir(&canonical).ok()?.enumerate() {
            if index >= MAX_INDEXED_ENTRIES {
                return None;
            }
            let entry = entry.ok()?;
            // The directory entry type needs no further syscall on Linux,
            // macOS or Windows. A link stays out: its identity is its target.
            if entry.file_type().ok()?.is_file() {
                files.insert(entry.file_name());
            }
        }
        Some(Self { canonical, files })
    }

    fn identity(&self, name: &OsStr) -> Option<PathBuf> {
        self.files.contains(name).then(|| self.canonical.join(name))
    }
}

impl TypeSourceSnapshot {
    /// The canonical identity of `candidate`, a path just observed as a
    /// regular file, when its directory listing proves it equal to
    /// `realpath(candidate)`.
    ///
    /// `realpath` resolves the parent spelling, `..` included, and appends the
    /// entry's on-disk name, following the entry when it is a link. An exact
    /// listed regular-file name is that on-disk name and no link. Anything
    /// else (a link, a case-insensitive match, a file created after the
    /// listing, an unreadable or very large directory) returns `None`, and the
    /// caller canonicalizes the full path as before.
    pub(super) fn indexed_identity(&self, candidate: &Path) -> Option<PathBuf> {
        let name = candidate.file_name()?;
        let directory = candidate.parent()?;
        // A relative spelling follows the current directory, which may change
        // between probes; only absolute directories have a stable identity.
        if !directory.is_absolute() {
            return None;
        }
        self.directory_cell(directory)?
            .get_or_init(|| IndexedDirectory::read(directory))
            .as_ref()?
            .identity(name)
    }

    /// [`Self::indexed_identity`] in a directory whose canonical path the
    /// caller already resolved, without canonicalizing it again.
    pub(super) fn indexed_identity_in(&self, canonical: &Path, name: &OsStr) -> Option<PathBuf> {
        self.directory_cell(canonical)?
            .get_or_init(|| IndexedDirectory::list(canonical.to_path_buf()))
            .as_ref()?
            .identity(name)
    }

    /// One cell per directory spelling, with `.` components dropped: `a/./b`
    /// and `a/b` name one directory, physically as well. The map lock is
    /// released before any filesystem work, so unrelated directories list in
    /// parallel and one directory lists once.
    fn directory_cell(&self, directory: &Path) -> Option<super::DirectoryCell> {
        let key: PathBuf = directory.components().collect();
        Some(Arc::clone(
            self.directories.lock().ok()?.entry(key).or_default(),
        ))
    }
}
