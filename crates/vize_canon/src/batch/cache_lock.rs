//! Coordination for opening and removing project-keyed cache lock files.

use std::ffi::OsString;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

const CACHE_LOCK_SUFFIX: &str = ".lock";

/// Keep cache maintenance from replacing a per-project lock between opening
/// its path and acquiring the lock on that inode.
pub(super) fn acquire_shared(virtual_root: &Path) -> io::Result<File> {
    let file = open(virtual_root)?;
    file.lock_shared()?;
    Ok(file)
}

/// Exclude new per-project lock acquisitions while idle lock files are removed.
/// Maintenance stays best-effort and skips cleanup instead of waiting for a
/// concurrent opener.
pub(super) fn try_acquire_exclusive(virtual_root: &Path) -> Option<File> {
    let file = open(virtual_root).ok()?;
    match file.try_lock() {
        Ok(()) => Some(file),
        Err(TryLockError::WouldBlock | TryLockError::Error(_)) => None,
    }
}

fn open(virtual_root: &Path) -> io::Result<File> {
    let path = lock_path(virtual_root).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "virtual project cache root has no projects directory",
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .write(true)
        .open(path)
}

fn lock_path(virtual_root: &Path) -> Option<PathBuf> {
    let projects = virtual_root.parent()?;
    let mut name = OsString::from(projects.file_name()?);
    name.push(CACHE_LOCK_SUFFIX);
    Some(projects.with_file_name(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordination_lock_is_singleton_outside_projects_directory() {
        assert_eq!(
            lock_path(Path::new("/cache/vize/canon/projects/project")),
            Some(PathBuf::from("/cache/vize/canon/projects.lock"))
        );
    }
}
