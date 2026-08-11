//! Per-project cache leases and bounded collection.

use std::{
    fs::{self, TryLockError},
    io::ErrorKind,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use super::{
    ensure_entry,
    ownership::{ensure_bucket, ensure_project, validate_project},
    validate_entry,
};

const MAX_INACTIVE_CONFIGS: usize = 8;
const MAX_INACTIVE_PROJECTS_PER_SHARD: usize = 8;
const LOCK_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct ConfigLease {
    path: PathBuf,
    file: Option<fs::File>,
}

impl Drop for ConfigLease {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}

pub(super) fn acquire(
    cache_root: &Path,
    project_digest: &str,
    config_digest: &str,
) -> Result<(PathBuf, PathBuf, ConfigLease), std::io::Error> {
    let shard = project_digest.get(..2).ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::InvalidInput,
            "Nuxt config project identity has no cache shard",
        )
    })?;
    let bucket = ensure_bucket(cache_root, shard)?;
    let _bucket_lock = CacheLock::acquire(&bucket.join(".gc.lock"), "cache shard")?;
    let project_cache = ensure_project(&bucket, project_digest)?;
    let _project_lock = project_lock(&project_cache)?;
    record_project_access(&project_cache)?;
    let entry = ensure_entry(&project_cache, config_digest)?;
    cleanup_dead_leases(&entry)?;
    let lease = tempfile::Builder::new()
        .prefix(&format!(".lease-{}-", std::process::id()))
        .tempfile_in(&entry)?;
    lease.as_file().lock()?;
    let (file, path) = lease.keep().map_err(|error| error.error)?;
    Ok((
        project_cache,
        entry,
        ConfigLease {
            path,
            file: Some(file),
        },
    ))
}

pub(super) fn collect(project_cache: &Path, current: &Path) -> Result<(), std::io::Error> {
    let _lock = project_lock(project_cache)?;
    let mut entries = Vec::new();
    for candidate in fs::read_dir(project_cache)? {
        let candidate = candidate?;
        let path = candidate.path();
        if !validate_entry(project_cache, &path)? {
            continue;
        }
        entries.push((candidate.metadata()?.modified()?, path));
    }
    if entries.len() <= MAX_INACTIVE_CONFIGS + 1 {
        return Ok(());
    }
    entries.sort_by_key(|(modified, _)| *modified);
    let mut remaining = entries.len() - (MAX_INACTIVE_CONFIGS + 1);
    for (_, entry) in entries.into_iter().filter(|(_, path)| path != current) {
        if collect_entry(&entry)? {
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }
    Ok(())
}

pub(super) fn collect_projects(
    cache_root: &Path,
    current_project: &Path,
) -> Result<usize, std::io::Error> {
    let bucket = current_project.parent().ok_or_else(|| {
        std::io::Error::new(ErrorKind::InvalidInput, "Nuxt project cache has no shard")
    })?;
    if bucket.parent() != Some(cache_root) {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Nuxt project cache is outside its cache shard",
        ));
    }
    let _bucket_lock = CacheLock::acquire(&bucket.join(".gc.lock"), "cache shard")?;
    cleanup_orphan_project_locks(bucket)?;
    let mut projects = Vec::new();
    for candidate in fs::read_dir(bucket)? {
        let candidate = candidate?;
        let path = candidate.path();
        if !validate_project(bucket, &path)? {
            continue;
        }
        projects.push((project_access_order(&path), path));
    }
    let scanned = projects.len();
    if projects.len() <= MAX_INACTIVE_PROJECTS_PER_SHARD + 1 {
        return Ok(scanned);
    }
    projects.sort_by_key(|(modified, _)| *modified);
    let mut remaining = projects.len() - (MAX_INACTIVE_PROJECTS_PER_SHARD + 1);
    for (_, project) in projects
        .into_iter()
        .filter(|(_, path)| path != current_project)
    {
        let project_lock_path = project_lock_path(&project)?;
        let project_lock = CacheLock::acquire(&project_lock_path, "project-local cache")?;
        if project_has_live_readers(&project)? {
            continue;
        }
        match fs::remove_dir_all(&project) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        drop(project_lock);
        match fs::remove_file(&project_lock_path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
    Ok(scanned)
}

fn cleanup_orphan_project_locks(bucket: &Path) -> Result<(), std::io::Error> {
    for candidate in fs::read_dir(bucket)? {
        let candidate = candidate?;
        let name = candidate.file_name();
        let Some(digest) = name
            .to_str()
            .and_then(|name| name.strip_prefix(".project-"))
            .and_then(|name| name.strip_suffix(".lock"))
        else {
            continue;
        };
        if bucket.join(digest).exists() {
            continue;
        }
        let lock = CacheLock::acquire(&candidate.path(), "orphan project")?;
        drop(lock);
        match fs::remove_file(candidate.path()) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn record_project_access(project: &Path) -> Result<(), std::io::Error> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(std::io::Error::other)?;
    let path = project.join(".last-used");
    if let Ok(metadata) = fs::symlink_metadata(&path)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Nuxt project cache access marker is not a regular file",
        ));
    }
    fs::write(path, format!("{:039}\n", elapsed.as_nanos()))
}

fn project_access_order(project: &Path) -> u128 {
    fs::read_to_string(project.join(".last-used"))
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or_default()
}

fn project_has_live_readers(project: &Path) -> Result<bool, std::io::Error> {
    for candidate in fs::read_dir(project)? {
        let candidate = candidate?;
        let entry = candidate.path();
        if !validate_entry(project, &entry)? {
            continue;
        }
        cleanup_dead_leases(&entry)?;
        if entry_has_live_readers(&entry)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn collect_entry(entry: &Path) -> Result<bool, std::io::Error> {
    cleanup_dead_leases(entry)?;
    if entry_has_live_readers(entry)? {
        return Ok(false);
    }
    match fs::remove_dir_all(entry) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error),
    }
}

fn entry_has_live_readers(entry: &Path) -> Result<bool, std::io::Error> {
    Ok(fs::read_dir(entry)?
        .filter_map(Result::ok)
        .any(|candidate| {
            candidate
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(".lease-"))
        }))
}

struct CacheLock {
    file: fs::File,
}

impl CacheLock {
    fn acquire(path: &Path, scope: &str) -> Result<Self, std::io::Error> {
        if let Ok(metadata) = fs::symlink_metadata(path)
            && (metadata.file_type().is_symlink() || !metadata.is_file())
        {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("Nuxt config {scope} lock is not a regular file"),
            ));
        }
        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .truncate(false)
            .write(true)
            .open(path)?;
        let started = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { file }),
                Err(TryLockError::Error(error)) if error.kind() == ErrorKind::Interrupted => {}
                Err(TryLockError::Error(error)) => return Err(error),
                Err(TryLockError::WouldBlock) => {}
            }
            if started.elapsed() >= LOCK_TIMEOUT {
                return Err(std::io::Error::new(
                    ErrorKind::TimedOut,
                    format!("timed out waiting for the Nuxt config {scope} lock"),
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for CacheLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

fn project_lock(project_cache: &Path) -> Result<CacheLock, std::io::Error> {
    CacheLock::acquire(&project_lock_path(project_cache)?, "project-local cache")
}

fn project_lock_path(project_cache: &Path) -> Result<PathBuf, std::io::Error> {
    let bucket = project_cache.parent().ok_or_else(|| {
        std::io::Error::new(ErrorKind::InvalidInput, "Nuxt project cache has no shard")
    })?;
    let digest = project_cache.file_name().ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::InvalidInput,
            "Nuxt project cache has no identity",
        )
    })?;
    Ok(bucket.join(format!(".project-{}.lock", digest.to_string_lossy())))
}

fn cleanup_dead_leases(entry: &Path) -> Result<(), std::io::Error> {
    for candidate in fs::read_dir(entry)?.filter_map(Result::ok) {
        let name = candidate.file_name();
        if !name
            .to_str()
            .is_some_and(|name| name.starts_with(".lease-"))
        {
            continue;
        }
        let path = candidate.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "Nuxt config reader lease is not a regular file",
            ));
        }
        let file = fs::OpenOptions::new().read(true).write(true).open(&path)?;
        match file.try_lock() {
            Ok(()) => {
                drop(file);
                match fs::remove_file(path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
            Err(TryLockError::WouldBlock) => {}
            Err(TryLockError::Error(error)) => return Err(error),
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "lease_tests.rs"]
mod tests;
