//! A private, scoped ES module keeps evaluator code out of process arguments.

use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

/// Preserve the logical argv contract previously supplied by Node eval mode.
const ARGV_COMPATIBILITY: &[u8] = b"process.argv.splice(1, 1);\n";

// Give only the owned evaluator a stable V8 script identity. Its private cache
// path must not turn an unchanged authored failure into a new editor notice.
// Authored module paths and complete stderr bytes keep their native identities.
const SOURCE_IDENTITY: &[u8] = b"\n//# sourceURL=vize:config-runtime\n";
const CACHE_DIRECTORY: &str = "config-runtime-v1";
const CACHE_LIMIT: usize = 4;

pub(super) struct NodeRuntimeFile {
    path: PathBuf,
    _lease: Option<File>,
    _temporary: Option<tempfile::TempPath>,
}

impl NodeRuntimeFile {
    pub(super) fn new(script: &str) -> io::Result<Self> {
        let source = runtime_source(script);
        if let Ok(home) = cache_home()
            && let Ok(cached) = cached_runtime(&home, &source)
        {
            return Ok(cached);
        }
        temporary_runtime(&source)
    }

    #[cfg(test)]
    fn new_with_cache_home(script: &str, home: &Path) -> io::Result<Self> {
        let source = runtime_source(script);
        cached_runtime(home, &source).or_else(|_| temporary_runtime(&source))
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn command(&self, config_path: &Path) -> Command {
        let mut command = Command::new("node");
        command
            .arg(self.path())
            .arg(config_path)
            .current_dir(config_path.parent().unwrap_or_else(|| Path::new(".")));
        command
    }

    #[cfg(test)]
    fn is_cached(&self) -> bool {
        self._lease.is_some()
    }
}

fn runtime_source(script: &str) -> Vec<u8> {
    let mut source =
        Vec::with_capacity(ARGV_COMPATIBILITY.len() + script.len() + SOURCE_IDENTITY.len());
    source.extend_from_slice(ARGV_COMPATIBILITY);
    source.extend_from_slice(script.as_bytes());
    source.extend_from_slice(SOURCE_IDENTITY);
    source
}

fn cache_home() -> io::Result<PathBuf> {
    let home = |name| {
        env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    if let Some(xdg) = home("XDG_CACHE_HOME") {
        return Ok(xdg);
    }
    if cfg!(windows) {
        return home("LOCALAPPDATA")
            .or_else(|| home("USERPROFILE").map(|path| path.join("AppData").join("Local")))
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "no platform cache home"));
    }
    let user = home("HOME")
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "no platform cache home"))?;
    if cfg!(target_os = "macos") {
        Ok(user.join("Library").join("Caches"))
    } else {
        Ok(user.join(".cache"))
    }
}

fn cache_root(home: &Path) -> PathBuf {
    home.join("vize").join(CACHE_DIRECTORY)
}

fn cached_runtime(home: &Path, source: &[u8]) -> io::Result<NodeRuntimeFile> {
    let vize = home.join("vize");
    let root = cache_root(home);
    ensure_private_directory(&vize)?;
    ensure_private_directory(&root)?;
    recover_interrupted_writes(&root);

    let hash = sha256_hex(source);
    let path = root.join(crate::cstr!("{hash}.mjs").as_str());
    let lease_path = root.join(crate::cstr!("{hash}.lease").as_str());
    let lease = open_lease(&lease_path)?;
    lease.lock()?;
    if !runtime_matches(&path, source) {
        remove_invalid_runtime(&path)?;
        publish_runtime(&root, &path, &hash, source)?;
    }
    lease.unlock()?;
    lease.lock_shared()?;
    if !runtime_matches(&path, source) {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            "cached config runtime disappeared before use",
        ));
    }
    prune_cache(&root, &path);
    Ok(NodeRuntimeFile {
        path,
        _lease: Some(lease),
        _temporary: None,
    })
}

fn temporary_runtime(source: &[u8]) -> io::Result<NodeRuntimeFile> {
    let mut file = tempfile::Builder::new()
        .prefix("vize-config-runtime-")
        .suffix(".mjs")
        .tempfile()?;
    file.write_all(source)?;
    file.flush()?;
    set_private_file_permissions(file.as_file())?;
    let path = file.into_temp_path();
    Ok(NodeRuntimeFile {
        path: path.to_path_buf(),
        _lease: None,
        _temporary: Some(path),
    })
}

fn ensure_private_directory(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "config runtime cache is not an owned directory",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn open_lease(path: &Path) -> io::Result<File> {
    let file = match private_open_options().create_new(true).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path)?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "config runtime lease is not an owned file",
                ));
            }
            private_open_options().open(path)?
        }
        Err(error) => return Err(error),
    };
    set_private_file_permissions(&file)?;
    Ok(file)
}

fn private_open_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn set_private_file_permissions(file: &File) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn runtime_matches(path: &Path, source: &[u8]) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        && fs::read(path).is_ok_and(|bytes| bytes == source)
}

fn remove_invalid_runtime(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(io::Error::new(
                ErrorKind::InvalidInput,
                "config runtime cache entry is not an owned file",
            ))
        }
        Ok(_) => fs::remove_file(path),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn publish_runtime(root: &Path, path: &Path, hash: &str, source: &[u8]) -> io::Result<()> {
    let prefix = crate::cstr!("{hash}.{}.", std::process::id());
    let mut temporary = tempfile::Builder::new()
        .prefix(prefix.as_str())
        .suffix(".tmp")
        .tempfile_in(root)?;
    temporary.as_file().lock()?;
    temporary.write_all(source)?;
    temporary.flush()?;
    set_private_file_permissions(temporary.as_file())?;
    let file = temporary
        .persist_noclobber(path)
        .map_err(|error| error.error)?;
    set_private_file_permissions(&file)?;
    drop(file);
    Ok(())
}

fn recover_interrupted_writes(root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let entries: Vec<_> = entries.flatten().collect();
    for entry in &entries {
        let path = entry.path();
        if !entry.file_name().to_string_lossy().ends_with(".tmp") {
            continue;
        }
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            continue;
        }
        let Ok(file) = private_open_options().open(&path) else {
            continue;
        };
        if file.try_lock().is_err() {
            continue;
        }
        drop(file);
        let _ = fs::remove_file(path);
    }
    for entry in entries {
        let path = entry.path();
        if path
            .extension()
            .is_none_or(|extension| extension != "lease")
        {
            continue;
        }
        let runtime = path.with_extension("mjs");
        if fs::symlink_metadata(&runtime)
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        {
            continue;
        }
        let Ok(file) = private_open_options().open(&path) else {
            continue;
        };
        if file.try_lock().is_err() {
            continue;
        }
        drop(file);
        let _ = fs::remove_file(path);
    }
}

fn prune_cache(root: &Path, current: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut runtimes: Vec<_> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            is_runtime_entry(&path).then(|| {
                let modified = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .unwrap_or(UNIX_EPOCH);
                (path, modified)
            })
        })
        .collect();
    runtimes.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));
    let mut retained = runtimes.len();
    for (path, _) in runtimes.into_iter().rev() {
        if retained <= CACHE_LIMIT {
            break;
        }
        if path == current || !remove_inactive_runtime(&path) {
            continue;
        }
        retained -= 1;
    }
}

fn is_runtime_entry(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.strip_suffix(".mjs")
        .is_some_and(|stem| stem.len() == 64 && stem.bytes().all(|byte| byte.is_ascii_hexdigit()))
        && fs::symlink_metadata(path)
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn remove_inactive_runtime(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };
    let lease_path = path.with_file_name(crate::cstr!("{stem}.lease").as_str());
    let Ok(metadata) = fs::symlink_metadata(&lease_path) else {
        return false;
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return false;
    }
    let Ok(lease) = private_open_options().open(&lease_path) else {
        return false;
    };
    if lease.try_lock().is_err() || fs::remove_file(path).is_err() {
        return false;
    }
    drop(lease);
    let _ = fs::remove_file(lease_path);
    true
}

fn sha256_hex(source: &[u8]) -> std::string::String {
    let mut hex = std::string::String::with_capacity(64);
    for byte in Sha256::digest(source) {
        hex.push_str(crate::cstr!("{byte:02x}").as_str());
    }
    hex
}

#[cfg(test)]
mod tests;
