//! Canonical identities for probed files, without `realpath` per candidate.
//!
//! `realpath` walks every path component: one `getattrlist` per component on
//! macOS, a `readlink` per component on Linux (a 9P round trip each on a WSL
//! `/mnt/c` drive), and an open handle plus `GetFinalPathNameByHandleW` on
//! Windows. A snapshot resolves many candidates in few directories, so it
//! canonicalizes each directory once.
//!
//! On Linux, `realpath(dir/name)` is `realpath(dir)/name` whenever `name` is
//! no link, since glibc keeps every spelled name. What `lstat` reports
//! therefore answers a candidate (a request's snapshot asks it of an
//! `O_PATH | O_NOFOLLOW` handle it holds for the file's read). A directory
//! is listed only once it has answered [`PROBES_BEFORE_LISTING`] probes: an
//! editor's lone transform probes a directory two or three times, a host
//! burst hundreds of times. macOS respells names in their on-disk case and
//! Windows canonicalizes through a handle, so those systems list each
//! directory's regular files on the first probe.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use vize_carton::FxHashSet;

use super::TypeSourceSnapshot;

/// Directories with more entries are not listed: one `stat` of the probed
/// file is cheaper than reading a large package store to answer it.
const MAX_INDEXED_ENTRIES: usize = 4096;

/// Probes a directory answers with one `O_PATH` open each before it is
/// listed: a listing costs an open, a `fstat`, two `getdents64` and a close.
#[cfg(target_os = "linux")]
const PROBES_BEFORE_LISTING: u32 = 4;

/// A directory spelling as this snapshot first resolved it.
#[derive(Debug, Default)]
pub(in crate::script::context) struct ProbedDirectory {
    canonical: OnceLock<Option<PathBuf>>,
    /// Entries that are regular files themselves, never symbolic links.
    files: OnceLock<Option<FxHashSet<OsString>>>,
    #[cfg(target_os = "linux")]
    probes: std::sync::atomic::AtomicU32,
}

/// What one probe of a name in a canonical directory proved.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Probe {
    /// A regular file that is no link: its `realpath` is this path.
    Found(PathBuf),
    /// Nothing exists at this physical path.
    #[cfg_attr(
        not(target_os = "linux"),
        expect(dead_code, reason = "only Linux probes prove absence")
    )]
    Absent(PathBuf),
    /// A link, a directory or an unreadable entry: canonicalize as before.
    Unknown,
}

impl ProbedDirectory {
    #[cfg(test)]
    pub(super) fn listed(&self) -> bool {
        self.files.get().is_some()
    }

    /// List a directory whose canonical path is already known.
    fn list(canonical: &Path) -> Option<FxHashSet<OsString>> {
        let mut files = FxHashSet::default();
        for (index, entry) in std::fs::read_dir(canonical).ok()?.enumerate() {
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
        Some(files)
    }

    /// The listing, once this directory has earned one.
    fn files(&self, canonical: &Path) -> Option<&FxHashSet<OsString>> {
        if let Some(files) = self.files.get() {
            return files.as_ref();
        }
        #[cfg(target_os = "linux")]
        if self
            .probes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            < PROBES_BEFORE_LISTING
        {
            return None;
        }
        self.files.get_or_init(|| Self::list(canonical)).as_ref()
    }

    /// Listings freeze hits only, like `existing`: a name they lack is
    /// probed again, so files created after the listing still resolve.
    fn probe(&self, snapshot: &TypeSourceSnapshot, canonical: &Path, name: &OsStr) -> Probe {
        if self
            .files(canonical)
            .is_some_and(|files| files.contains(name))
        {
            return Probe::Found(canonical.join(name));
        }
        #[cfg(target_os = "linux")]
        {
            let path = canonical.join(name);
            // `O_NOFOLLOW` keeps a link itself, so its type is what `lstat`
            // reports; the held handle keeps a found file resolved for its read.
            let metadata = if snapshot.held.enabled() {
                super::realpath::open_path(&path, libc::O_NOFOLLOW).and_then(|handle| {
                    let metadata = handle.metadata()?;
                    if metadata.is_file() {
                        snapshot.held.hold(handle);
                    }
                    Ok(metadata)
                })
            } else {
                std::fs::symlink_metadata(&path)
            };
            match metadata {
                Ok(metadata) if metadata.is_file() => Probe::Found(path),
                Ok(_) => Probe::Unknown,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Probe::Absent(path),
                Err(_) => Probe::Unknown,
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = snapshot;
            Probe::Unknown
        }
    }
}

impl TypeSourceSnapshot {
    /// What `candidate`'s directory proves about it, with its canonical
    /// identity when it is a regular file that is no link.
    ///
    /// `realpath` resolves the parent spelling, `..` included, and appends the
    /// entry's on-disk name, following the entry when it is a link. An exact
    /// listed regular-file name is that on-disk name and no link. Anything
    /// else (a link, a case-insensitive match, a file created after the
    /// listing, an unreadable or very large directory) is `Unknown` outside
    /// Linux, and the caller canonicalizes the full path as before.
    pub(super) fn probe_candidate(&self, candidate: &Path) -> Probe {
        let (Some(name), Some(directory)) = (candidate.file_name(), candidate.parent()) else {
            return Probe::Unknown;
        };
        // A relative spelling follows the current directory, which may change
        // between probes; only absolute directories have a stable identity.
        if !directory.is_absolute() {
            return Probe::Unknown;
        }
        let Some(cell) = self.directory_cell(directory) else {
            return Probe::Unknown;
        };
        let canonical = cell.canonical.get_or_init(|| {
            let canonical = self
                .walk_canonical_directory(directory)
                .or_else(|| self.canonicalize_held(directory))?;
            self.note_canonical_directory(&canonical);
            Some(canonical)
        });
        match canonical {
            Some(canonical) => cell.probe(self, canonical, name),
            None => Probe::Unknown,
        }
    }

    /// [`Self::probe_candidate`] in a directory whose canonical path the
    /// caller already resolved, without canonicalizing it again.
    pub(super) fn probe_in(&self, canonical: &Path, name: &OsStr) -> Probe {
        let Some(cell) = self.directory_cell(canonical) else {
            return Probe::Unknown;
        };
        cell.canonical.get_or_init(|| {
            self.note_canonical_directory(canonical);
            Some(canonical.to_path_buf())
        });
        cell.probe(self, canonical, name)
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
