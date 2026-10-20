//! Bounded maintenance for project-keyed Canon mirrors in the user cache.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const ACCESS_SUFFIX: &str = ".access";
const ACTIVE_LOCK_SUFFIX: &str = ".active.lock";
const LOCK_SUFFIXES: [&str; 3] = [".lock", ".materialize.lock", ACTIVE_LOCK_SUFFIX];
const MAX_PROJECT_ENTRIES: usize = 24;
const MAX_UNUSED_AGE: Duration = Duration::from_secs(14 * 24 * 60 * 60);

/// Record the current namespace as recently used and evict old/excess mirrors.
///
/// Lock acquisition makes eviction safe against another process materializing
/// the same project. All failures are ignored because these entries are only a
/// cache and cache maintenance must not make type checking unavailable.
pub(super) fn maintain_project_cache(current: &Path) {
    let Some(projects) = current.parent() else {
        return;
    };
    if projects.parent().and_then(Path::file_name) != Some(std::ffi::OsStr::new("canon")) {
        return;
    }
    if fs::create_dir_all(projects).is_err() {
        return;
    }

    let _ = touch(&access_path(current));
    let Ok(read_dir) = fs::read_dir(projects) else {
        return;
    };
    let now = SystemTime::now();
    let mut entries = read_dir
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path == current || !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                return None;
            }
            let name = entry.file_name();
            let name = name.to_str()?;
            if name.len() != 64 || !name.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return None;
            }
            let modified = fs::metadata(access_path(&path))
                .or_else(|_| entry.metadata())
                .and_then(|metadata| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            Some((path, modified))
        })
        .collect::<Vec<_>>();

    entries.sort_by_key(|(_, modified)| *modified);
    let excess = entries
        .len()
        .saturating_add(1)
        .saturating_sub(MAX_PROJECT_ENTRIES);
    for (index, (path, modified)) in entries.into_iter().enumerate() {
        let expired = now
            .duration_since(modified)
            .is_ok_and(|age| age >= MAX_UNUSED_AGE);
        if (index < excess || expired)
            && let Some(locks) = lock_unused(&path)
        {
            let _locks = locks;
            let _ = fs::remove_dir_all(&path);
            let _ = fs::remove_file(access_path(&path));
            // Retain the tiny sibling lock files. Unlinking a held lock lets a
            // racing process create a new inode that is not mutually excluded.
        }
    }
}

/// Keep a project cache entry alive while a [`VirtualProject`](super::VirtualProject)
/// can still read or incrementally update its materialized tree.
pub(super) fn acquire_usage_lock(root: &Path) -> Option<File> {
    let path = sibling_path(root, ACTIVE_LOCK_SUFFIX);
    fs::create_dir_all(path.parent()?).ok()?;
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .write(true)
        .open(path)
        .ok()?;
    file.lock_shared().ok()?;
    Some(file)
}

fn touch(path: &Path) -> std::io::Result<()> {
    let _ = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    Ok(())
}

fn access_path(root: &Path) -> PathBuf {
    sibling_path(root, ACCESS_SUFFIX)
}

fn sibling_path(root: &Path, suffix: &str) -> PathBuf {
    let Some(name) = root.file_name() else {
        return root.with_extension(suffix.trim_start_matches('.'));
    };
    let mut name = name.to_os_string();
    name.push(suffix);
    root.with_file_name(name)
}

fn lock_unused(root: &Path) -> Option<Vec<File>> {
    let mut locks = Vec::with_capacity(LOCK_SUFFIXES.len());
    for suffix in LOCK_SUFFIXES {
        let path = sibling_path(root, suffix);
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .truncate(false)
            .write(true)
            .open(path)
            .ok()?;
        match file.try_lock() {
            Ok(()) => locks.push(file),
            Err(TryLockError::WouldBlock) => return None,
            Err(TryLockError::Error(_)) => return None,
        }
    }
    Some(locks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> std::string::String {
        format!("{byte:064x}")
    }

    #[test]
    fn maintenance_bounds_project_entries_and_keeps_current() {
        let cache = tempfile::tempdir().unwrap();
        let projects = cache.path().join("canon/projects");
        fs::create_dir_all(&projects).unwrap();
        for index in 0..(MAX_PROJECT_ENTRIES + 8) {
            fs::create_dir(projects.join(key(index as u8))).unwrap();
        }
        let current = projects.join(key(255));
        fs::create_dir(&current).unwrap();

        maintain_project_cache(&current);

        let count = fs::read_dir(&projects)
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .count();
        assert_eq!(count, MAX_PROJECT_ENTRIES);
        assert!(current.is_dir());
        assert!(access_path(&current).is_file());
    }

    #[test]
    fn maintenance_never_removes_a_locked_entry() {
        let cache = tempfile::tempdir().unwrap();
        let projects = cache.path().join("canon/projects");
        fs::create_dir_all(&projects).unwrap();
        let locked = projects.join(key(1));
        let current = projects.join(key(2));
        fs::create_dir(&locked).unwrap();
        fs::create_dir(&current).unwrap();
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .truncate(false)
            .write(true)
            .open(sibling_path(&locked, LOCK_SUFFIXES[0]))
            .unwrap();
        lock.lock().unwrap();
        for index in 3..(MAX_PROJECT_ENTRIES + 12) {
            fs::create_dir(projects.join(key(index as u8))).unwrap();
        }

        maintain_project_cache(&current);

        assert!(locked.is_dir());
    }

    #[test]
    fn maintenance_never_removes_an_entry_with_a_live_project_owner() {
        let cache = tempfile::tempdir().unwrap();
        let projects = cache.path().join("canon/projects");
        fs::create_dir_all(&projects).unwrap();
        let live = projects.join(key(1));
        let current = projects.join(key(2));
        fs::create_dir(&live).unwrap();
        fs::create_dir(&current).unwrap();
        let _usage = acquire_usage_lock(&live).unwrap();
        for index in 3..(MAX_PROJECT_ENTRIES + 12) {
            fs::create_dir(projects.join(key(index as u8))).unwrap();
        }

        maintain_project_cache(&current);

        assert!(live.is_dir());
    }
}
