//! Ownership, identity, and atomic publication for generated Nuxt configs.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use vize_carton::String;

mod lease;
mod ownership;
pub(super) use lease::ConfigLease;
pub(super) use ownership::{ensure_entry, validate_entry};

pub(super) fn config_cache_root(project_root: &Path) -> Result<PathBuf, std::io::Error> {
    let requested = dirs::cache_dir()
        .ok_or_else(|| std::io::Error::other("no user cache directory is available"))?
        .join("vize/check/nuxt/v2");
    let planned = canonicalize_with_missing_tail(&requested);
    validate_config_cache_root(&planned, project_root)?;
    fs::create_dir_all(&requested)?;
    validate_config_cache_root(&requested, project_root)
}

pub(super) fn validate_config_cache_root(
    cache_root: &Path,
    project_root: &Path,
) -> Result<PathBuf, std::io::Error> {
    let cache_root = vize_carton::path::canonicalize_non_verbatim(cache_root);
    let project_root = vize_carton::path::canonicalize_non_verbatim(project_root);
    let dependency_root =
        vize_carton::path::canonicalize_non_verbatim(&project_root.join("node_modules"));
    let names_dependency_tree = cache_root.components().any(|component| {
        matches!(component, std::path::Component::Normal(name) if name
            .to_string_lossy()
            .eq_ignore_ascii_case("node_modules"))
    });
    if names_dependency_tree
        || cache_root.starts_with(&project_root)
        || cache_root.starts_with(&dependency_root)
    {
        return Err(std::io::Error::other(
            "Vize's Nuxt config cache resolves inside the project dependency tree",
        ));
    }
    Ok(cache_root)
}

fn canonicalize_with_missing_tail(path: &Path) -> PathBuf {
    let mut ancestor = path.to_path_buf();
    let mut tail = Vec::new();
    while !ancestor.exists() {
        let Some(name) = ancestor.file_name().map(ToOwned::to_owned) else {
            break;
        };
        tail.push(name);
        if !ancestor.pop() {
            break;
        }
    }
    let mut resolved = vize_carton::path::canonicalize_non_verbatim(&ancestor);
    for name in tail.into_iter().rev() {
        resolved.push(name);
    }
    resolved
}

pub(super) fn encode_digest(digest: impl AsRef<[u8]>) -> String {
    let digest = digest.as_ref();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

#[cfg(unix)]
pub(super) fn update_digest_path(digest: &mut Sha256, path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let bytes = path.as_os_str().as_bytes();
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}

#[cfg(windows)]
pub(super) fn update_digest_path(digest: &mut Sha256, path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    let units = path.as_os_str().encode_wide().collect::<Vec<_>>();
    digest.update((units.len() as u64).to_le_bytes());
    for unit in units {
        digest.update(unit.to_le_bytes());
    }
}

#[cfg(not(any(unix, windows)))]
pub(super) fn update_digest_path(digest: &mut Sha256, path: &Path) {
    let path = path.as_os_str().to_string_lossy();
    digest.update((path.len() as u64).to_le_bytes());
    digest.update(path.as_bytes());
}

pub(super) fn publish_config_atomically(path: &Path, content: &[u8]) -> Result<(), std::io::Error> {
    publish_config_atomically_with_hook(path, content, |_| {})
}

pub(super) fn acquire_config_lease(
    cache_root: &Path,
    project_digest: &str,
    config_digest: &str,
) -> Result<(PathBuf, PathBuf, ConfigLease), std::io::Error> {
    lease::acquire(cache_root, project_digest, config_digest)
}

pub(super) fn collect_project_cache(
    project_cache: &Path,
    current: &Path,
) -> Result<(), std::io::Error> {
    lease::collect(project_cache, current)
}

pub(super) fn collect_cache_projects(
    cache_root: &Path,
    current_project: &Path,
) -> Result<(), std::io::Error> {
    lease::collect_projects(cache_root, current_project).map(|_| ())
}

pub(super) fn publish_config_atomically_with_hook(
    path: &Path,
    content: &[u8],
    before_publish: impl FnOnce(&Path),
) -> Result<(), std::io::Error> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let publication_lock = publication_lock(directory)?;
    cleanup_abandoned_pending(directory)?;
    if path.exists() {
        return verify_published_config(path, content);
    }
    let prefix = format!(".vize-nuxt-config-{}-", std::process::id());
    let mut pending = tempfile::Builder::new()
        .prefix(&prefix)
        .suffix(".pending")
        .tempfile_in(directory)?;
    pending.write_all(content)?;
    pending.flush()?;
    pending.as_file().sync_all()?;
    let (file, pending_path) = pending.keep().map_err(|error| error.error)?;
    drop(file);
    before_publish(&pending_path);
    if path.exists() {
        fs::remove_file(&pending_path)?;
        return verify_published_config(path, content);
    }
    if let Err(error) = fs::rename(&pending_path, path) {
        let _ = fs::remove_file(pending_path);
        return Err(error);
    }
    sync_directory(directory)?;
    verify_published_config(path, content)?;
    publication_lock.unlock()
}

fn publication_lock(directory: &Path) -> Result<fs::File, std::io::Error> {
    let path = directory.join(".publish.lock");
    if let Ok(metadata) = fs::symlink_metadata(&path)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Nuxt config publication lock is not a regular file",
        ));
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .write(true)
        .open(path)?;
    file.lock()?;
    Ok(file)
}

fn cleanup_abandoned_pending(directory: &Path) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(pid) = pending_owner_pid(name) else {
            continue;
        };
        if pending_is_abandoned(&entry.path(), pid) {
            match fs::remove_file(entry.path()) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn pending_is_abandoned(_path: &Path, pid: u64) -> bool {
    !process_is_running(pid)
}

#[cfg(windows)]
fn pending_is_abandoned(_path: &Path, pid: u64) -> bool {
    !process_is_running(pid)
}

#[cfg(not(any(unix, windows)))]
fn pending_is_abandoned(path: &Path, _pid: u64) -> bool {
    const SAFE_STALE_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .is_some_and(|age| age >= SAFE_STALE_AGE)
}

fn pending_owner_pid(name: &str) -> Option<u64> {
    let owner = name.strip_prefix(".vize-nuxt-config-")?;
    let (pid, tail) = owner.split_once('-')?;
    tail.ends_with(".pending")
        .then(|| pid.parse().ok())
        .flatten()
}

#[cfg(unix)]
fn process_is_running(pid: u64) -> bool {
    if pid == 0 || pid > i32::MAX as u64 {
        return false;
    }
    // SAFETY: signal 0 only checks whether the owner process still exists.
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(windows)]
fn process_is_running(pid: u64) -> bool {
    if pid == 0 || pid > u32::MAX as u64 {
        return false;
    }
    type Handle = *mut std::ffi::c_void;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const WAIT_OBJECT_0: u32 = 0;
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, process_id: u32) -> Handle;
        fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
        fn CloseHandle(handle: Handle) -> i32;
    }
    // SAFETY: the API receives a validated PID and the returned handle is
    // closed below. A process we cannot query is conservatively retained.
    let handle = unsafe { OpenProcess(SYNCHRONIZE, 0, pid as u32) };
    if handle.is_null() {
        return std::io::Error::last_os_error().raw_os_error() != Some(87);
    }
    // SAFETY: `handle` is live and the zero timeout only observes its state.
    let state = unsafe { WaitForSingleObject(handle, 0) };
    // SAFETY: `handle` was returned by OpenProcess exactly once.
    unsafe { CloseHandle(handle) };
    state != WAIT_OBJECT_0
}

fn verify_published_config(path: &Path, expected: &[u8]) -> Result<(), std::io::Error> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "content-addressed Nuxt checker config is not a regular file",
        ));
    }
    if fs::read(path)? == expected {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "content-addressed Nuxt checker config has unexpected bytes",
        ))
    }
}

#[cfg(unix)]
fn sync_directory(directory: &Path) -> Result<(), std::io::Error> {
    fs::File::open(directory)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_directory: &Path) -> Result<(), std::io::Error> {
    Ok(())
}
