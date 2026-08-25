//! Ownership, identity, and atomic publication for generated Nuxt configs.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use vize_s0::String;

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
    let cache_root = vize_s0::path::canonicalize_non_verbatim(cache_root);
    let project_root = vize_s0::path::canonicalize_non_verbatim(project_root);
    let dependency_root =
        vize_s0::path::canonicalize_non_verbatim(&project_root.join("node_modules"));
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
    let mut resolved = vize_s0::path::canonicalize_non_verbatim(&ancestor);
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
    let _publication_lock = publication_lock(directory)?;
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
    Ok(())
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
        if !is_pending_name(name) {
            continue;
        }
        // Publication owns the directory lock, so no matching pending file can
        // still have a live writer. This is generation-safe even after PID reuse.
        match fs::remove_file(entry.path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

pub(super) fn is_pending_name(name: &str) -> bool {
    let Some(owner) = name.strip_prefix(".vize-nuxt-config-") else {
        return false;
    };
    let Some((pid, tail)) = owner.split_once('-') else {
        return false;
    };
    pid.parse::<u32>().is_ok() && tail.ends_with(".pending")
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
