//! Bounded maintenance for project-keyed Canon mirrors in the user cache.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const ACCESS_SUFFIX: &str = ".access";
const ACTIVE_LOCK_SUFFIX: &str = ".active.lock";
const LOCK_SUFFIXES: [&str; 3] = [".lock", ".materialize.lock", ACTIVE_LOCK_SUFFIX];
const MAX_PROJECT_ENTRIES: usize = 24;
const MAX_SCANNED_PROJECT_ENTRIES: usize = 256;
const MAX_EVICTIONS_PER_MAINTENANCE: usize = 8;
const MAX_UNUSED_AGE: Duration = Duration::from_secs(14 * 24 * 60 * 60);

#[derive(Debug, Default, PartialEq, Eq)]
struct CacheMaintenanceSummary {
    project_entries: usize,
    evicted: usize,
}

/// Record the current namespace as recently used and evict old/excess mirrors.
///
/// Lock acquisition makes eviction safe against another process materializing
/// the same project. All failures are ignored because these entries are only a
/// cache and cache maintenance must not make type checking unavailable.
pub(super) fn maintain_project_cache(current: &Path) {
    let _ = maintain_project_cache_with_summary(current);
}

fn maintain_project_cache_with_summary(current: &Path) -> CacheMaintenanceSummary {
    let mut summary = CacheMaintenanceSummary::default();
    let Some(projects) = current.parent() else {
        return summary;
    };
    if projects.parent().and_then(Path::file_name) != Some(std::ffi::OsStr::new("canon")) {
        return summary;
    }
    if fs::create_dir_all(projects).is_err() {
        return summary;
    }

    let _ = touch(&access_path(current));
    let Ok(read_dir) = fs::read_dir(projects) else {
        return summary;
    };
    let now = SystemTime::now();
    let mut entries = Vec::with_capacity(MAX_SCANNED_PROJECT_ENTRIES);
    // Sibling lock files are retained for inode-safe locking, so enumerate
    // past them while bounding metadata reads to actual project directories.
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path == current || !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.len() != 64 || !name.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            continue;
        }
        let modified = fs::metadata(access_path(&path))
            .or_else(|_| entry.metadata())
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        entries.push((path, modified));
        if entries.len() == MAX_SCANNED_PROJECT_ENTRIES {
            break;
        }
    }
    summary.project_entries = entries.len();

    entries.sort_by_key(|(_, modified)| *modified);
    let excess = entries
        .len()
        .saturating_add(1)
        .saturating_sub(MAX_PROJECT_ENTRIES);
    for (index, (path, modified)) in entries.into_iter().enumerate() {
        if summary.evicted == MAX_EVICTIONS_PER_MAINTENANCE {
            break;
        }
        let expired = now
            .duration_since(modified)
            .is_ok_and(|age| age >= MAX_UNUSED_AGE);
        if (index < excess || expired)
            && let Some(locks) = lock_unused(&path)
        {
            let _locks = locks;
            if fs::remove_dir_all(&path).is_ok() {
                summary.evicted += 1;
                let _ = fs::remove_file(access_path(&path));
            }
            // Retain the tiny sibling lock files. Unlinking a held lock lets a
            // racing process create a new inode that is not mutually excluded.
        }
    }
    summary
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
    fn maintenance_bounds_work_for_a_large_cache() {
        let cache = tempfile::tempdir().unwrap();
        let projects = cache.path().join("canon/projects");
        fs::create_dir_all(&projects).unwrap();
        let entry_count = MAX_SCANNED_PROJECT_ENTRIES * 8;
        for index in 0..entry_count {
            let entry = projects.join(format!("{index:064x}"));
            fs::create_dir(&entry).unwrap();
            fs::write(entry.join("source.vue"), "<template />").unwrap();
        }
        let current = projects.join(key(255));

        let summary = maintain_project_cache_with_summary(&current);

        assert_eq!(summary.project_entries, MAX_SCANNED_PROJECT_ENTRIES);
        assert_eq!(summary.evicted, MAX_EVICTIONS_PER_MAINTENANCE);
        let remaining = fs::read_dir(&projects)
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .count();
        assert_eq!(remaining, entry_count - MAX_EVICTIONS_PER_MAINTENANCE);
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
