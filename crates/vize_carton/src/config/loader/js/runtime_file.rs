//! A private, scoped ES module keeps evaluator code out of process arguments.

use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

mod maintenance;

use maintenance::{maintain_cache, reserve_cache_capacity};

/// Preserve the logical argv contract previously supplied by Node eval mode.
const ARGV_COMPATIBILITY: &[u8] = b"process.argv.splice(1, 1);\n";

// Give only the owned evaluator a stable V8 script identity. Its private cache
// path must not turn an unchanged authored failure into a new editor notice.
// Authored module paths and complete stderr bytes keep their native identities.
const SOURCE_IDENTITY: &[u8] = b"\n//# sourceURL=vize:config-runtime\n";
const CACHE_DIRECTORY: &str = "config-runtime-v1";
const CATALOG_LEASE: &str = "catalog.lock";
const CACHE_LIMIT: usize = 4;
const PROCESS_CACHE_LIMIT: usize = 32;

#[expect(
    clippy::disallowed_types,
    reason = "the process cache shares one validated lease without reopening it"
)]
type SharedRuntime = std::sync::Arc<CachedRuntime>;

struct CachedRuntime {
    path: PathBuf,
    _lease: File,
    metadata: RuntimeMetadata,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct RuntimeMetadata {
    len: u64,
    modified: Option<SystemTime>,
}

struct ProcessRuntime {
    home: PathBuf,
    source: Vec<u8>,
    runtime: SharedRuntime,
}

pub(in crate::config::loader) struct NodeRuntimeFile {
    path: PathBuf,
    _lease: Option<SharedRuntime>,
    _temporary: Option<tempfile::TempPath>,
}

impl NodeRuntimeFile {
    pub(in crate::config::loader) fn new(script: &str) -> io::Result<Self> {
        let source = runtime_source(script);
        if let Ok(home) = cache_home()
            && let Ok(cached) = process_runtime(&home, &source)
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

    pub(in crate::config::loader) fn command(&self, config_path: &Path) -> Command {
        let mut command = Command::new("node");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // The editor host has no console. Config evaluation must not open one.
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
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

fn process_cache() -> &'static Mutex<Vec<ProcessRuntime>> {
    static CACHE: OnceLock<Mutex<Vec<ProcessRuntime>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

fn process_runtime(home: &Path, source: &[u8]) -> io::Result<NodeRuntimeFile> {
    if let Some(cached) = process_cache_hit(home, source) {
        return Ok(cached);
    }
    reserve_process_runtime(home);
    let cached = cached_runtime(home, source)?;
    remember_process_runtime(home, source, &cached);
    Ok(cached)
}

fn reserve_process_runtime(home: &Path) {
    let Ok(mut entries) = process_cache().lock() else {
        return;
    };
    if entries.iter().filter(|entry| entry.home == home).count() >= CACHE_LIMIT
        && let Some(position) = entries.iter().position(|entry| entry.home == home)
    {
        entries.remove(position);
    }
}

fn process_cache_hit(home: &Path, source: &[u8]) -> Option<NodeRuntimeFile> {
    let runtime = {
        let mut entries = process_cache().lock().ok()?;
        let position = entries
            .iter()
            .position(|entry| entry.home == home && entry.source == source)?;
        let entry = entries.remove(position);
        let runtime = entry.runtime.clone();
        entries.push(entry);
        runtime
    };
    if runtime_metadata(&runtime.path).as_ref() == Some(&runtime.metadata) {
        return Some(NodeRuntimeFile {
            path: runtime.path.clone(),
            _lease: Some(runtime),
            _temporary: None,
        });
    }
    if let Ok(mut entries) = process_cache().lock() {
        entries.retain(|entry| !SharedRuntime::ptr_eq(&entry.runtime, &runtime));
    }
    None
}

fn remember_process_runtime(home: &Path, source: &[u8], runtime: &NodeRuntimeFile) {
    let Some(shared) = runtime._lease.as_ref() else {
        return;
    };
    let Ok(mut entries) = process_cache().lock() else {
        return;
    };
    entries.retain(|entry| !(entry.home == home && entry.source == source));
    while entries.iter().filter(|entry| entry.home == home).count() >= CACHE_LIMIT {
        let Some(position) = entries.iter().position(|entry| entry.home == home) else {
            break;
        };
        entries.remove(position);
    }
    if entries.len() >= PROCESS_CACHE_LIMIT {
        entries.remove(0);
    }
    entries.push(ProcessRuntime {
        home: home.to_path_buf(),
        source: source.to_vec(),
        runtime: shared.clone(),
    });
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
    let catalog = open_lease(&root.join(CATALOG_LEASE))?;

    let hash = sha256_hex(source);
    let path = root.join(crate::cstr!("{hash}.mjs").as_str());
    let lease_path = root.join(crate::cstr!("{hash}.lease").as_str());

    // Reserve an eviction slot before opening the target lease. Once the
    // catalog's shared boundary below is entered, it stays intact until the
    // runtime lease is validated, so maintenance cannot replace that lease
    // with a different file identity.
    reserve_cache_capacity(&root, &path, &catalog);
    let lease = open_shared_lease(&catalog, &lease_path)?;

    // Cache hits are readers. Keeping their lease shared lets the same
    // evaluator be reused by nested loads and other processes without an
    // exclusive-lock cycle. Only a missing or damaged entry needs repair.
    let validate = (|| {
        if runtime_matches(&path, source) {
            return Ok(());
        }
        lease.unlock()?;
        lease.try_lock()?;
        if !runtime_matches(&path, source) {
            remove_invalid_runtime(&path)?;
            publish_runtime(&root, &path, &hash, source)?;
        }
        lease.unlock()?;
        lease.try_lock_shared()?;
        runtime_matches(&path, source).then_some(()).ok_or_else(|| {
            io::Error::new(
                ErrorKind::NotFound,
                "cached config runtime disappeared before use",
            )
        })
    })();
    finish_catalog_operation(&catalog, validate)?;
    maintain_cache(&root, &path, &catalog);
    cached_runtime_file(path, lease)
}

fn cached_runtime_file(path: PathBuf, lease: File) -> io::Result<NodeRuntimeFile> {
    let metadata = runtime_metadata(&path).ok_or_else(|| {
        io::Error::new(
            ErrorKind::NotFound,
            "cached config runtime disappeared before metadata capture",
        )
    })?;
    let runtime = SharedRuntime::new(CachedRuntime {
        path: path.clone(),
        _lease: lease,
        metadata,
    });
    Ok(NodeRuntimeFile {
        path,
        _lease: Some(runtime),
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
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(path)?;
            fs::symlink_metadata(path)?
        }
        Err(error) => return Err(error),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "config runtime cache is not an owned directory",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(())
}

fn open_lease(path: &Path) -> io::Result<File> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => open_existing_lease(path, &metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            match private_open_options().create_new(true).open(path) {
                Ok(file) => {
                    set_private_file_permissions(&file)?;
                    Ok(file)
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    let metadata = fs::symlink_metadata(path)?;
                    open_existing_lease(path, &metadata)
                }
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

fn open_existing_lease(path: &Path, metadata: &fs::Metadata) -> io::Result<File> {
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "config runtime lease is not an owned file",
        ));
    }
    let file = private_open_options().open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            set_private_file_permissions(&file)?;
        }
    }
    Ok(file)
}

fn open_shared_lease(catalog: &File, path: &Path) -> io::Result<File> {
    catalog.try_lock_shared()?;
    let result = open_lease(path).and_then(|lease| {
        lease.try_lock_shared()?;
        Ok(lease)
    });
    if result.is_err() {
        let _ = catalog.unlock();
    }
    result
}

fn finish_catalog_operation<T>(catalog: &File, result: io::Result<T>) -> io::Result<T> {
    let unlock = catalog.unlock();
    match result {
        Ok(value) => {
            unlock?;
            Ok(value)
        }
        Err(error) => Err(error),
    }
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

fn runtime_metadata(path: &Path) -> Option<RuntimeMetadata> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return None;
    }
    Some(RuntimeMetadata {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
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

fn sha256_hex(source: &[u8]) -> std::string::String {
    let mut hex = std::string::String::with_capacity(64);
    for byte in Sha256::digest(source) {
        hex.push_str(crate::cstr!("{byte:02x}").as_str());
    }
    hex
}

#[cfg(test)]
mod tests;
