//! Persistent cache for CLI evaluations of JavaScript configs.
//!
//! Evaluating a Vite config starts Node and the project's Vite, which costs far
//! more than the native work of a short CLI run. An entry under
//! `node_modules/.vize/config-cache` is reused only while every input recorded
//! for it is unchanged: the import graph, files and directory listings read
//! through `node:fs`, the node_modules routes of bare imports, the environment
//! variables read, and the Node executable. Inputs outside that set (commands,
//! network, clock) are not tracked; `VIZE_CONFIG_CACHE=0` disables the cache.

use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Bump when the entry layout or the meaning of a recorded input changes.
const FORMAT: u32 = 1;
/// Entries kept per config, so alternating environments (a terminal, a
/// package-manager script, CI steps) each keep their own evaluation.
const ENTRIES: usize = 4;
/// Coarse filesystems store whole or even seconds. A file this recent may still
/// change within the same timestamp, so its evaluation is not stored.
const RACY_WINDOW: Duration = Duration::from_secs(2);
/// Always part of the key: they select the Node runtime and its module flags.
const RUNTIME_ENV: [&str; 3] = ["NODE_OPTIONS", "NODE_PATH", "PATH"];

/// Let this process reuse evaluations; batch CLI commands opt in.
pub fn enable_config_evaluation_cache() {
    ENABLED.store(true, Ordering::Relaxed);
}

/// The configuration values one Node evaluation produced.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CachedEvaluation {
    pub config: serde_json::Value,
    pub complete: bool,
    pub watch_dependencies: Vec<PathBuf>,
    pub dependencies: Vec<std::string::String>,
}

/// Inputs reported by `config-inputs.mjs`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RecordedInputs {
    env: Vec<(std::string::String, bool)>,
    env_enumerated: bool,
    opaque: bool,
    files: Vec<(PathBuf, bool)>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Key {
    format: u32,
    version: std::string::String,
    runtime: u64,
    config: PathBuf,
    explicit: bool,
}

/// What an evaluation read from one environment variable.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum EnvRead {
    /// Digest of the value; `None` when unset.
    Value(Option<std::string::String>),
    /// Only whether it is set (`in`, `Object.hasOwn`, enumeration).
    Presence(bool),
}

impl EnvRead {
    fn current(name: &str, value: bool) -> Self {
        let current = std::env::var_os(name);
        if value {
            Self::Value(current.map(|value| digest(value.as_encoded_bytes())))
        } else {
            Self::Presence(current.is_some())
        }
    }

    fn reads_value(&self) -> bool {
        matches!(self, Self::Value(_))
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Fingerprint {
    kind: u8,
    len: u64,
    modified: u64,
    changed: u64,
    inode: u64,
    link: Option<PathBuf>,
}

/// Owned when read; borrows the evaluation when written.
#[derive(Debug, Serialize, Deserialize)]
struct Entry<E = CachedEvaluation> {
    key: Key,
    env: Vec<(std::string::String, EnvRead)>,
    /// Digest of the variable names when the evaluation enumerated them.
    env_names: Option<std::string::String>,
    /// Path, whether its directory listing was read, and its fingerprint.
    inputs: Vec<(PathBuf, bool, Fingerprint)>,
    evaluation: E,
}

impl<E> Entry<E> {
    fn env_fresh(&self) -> bool {
        self.env
            .iter()
            .all(|(name, read)| EnvRead::current(name, read.reads_value()) == *read)
            && self
                .env_names
                .as_ref()
                .is_none_or(|digest| *digest == env_names_digest())
    }

    fn inputs_fresh(&self) -> bool {
        self.inputs
            .iter()
            .all(|(path, listing, stored)| fingerprint(path, *listing) == *stored)
    }
}

/// Where one config's entries live and the key they must match.
pub(super) struct CacheSlot {
    path: PathBuf,
    key: Key,
    /// Entries whose files are unchanged but whose environment differs.
    others: Vec<Entry>,
}

impl CacheSlot {
    /// `None` unless the cache is enabled and a node_modules directory exists.
    pub(super) fn new(config: &Path, explicit: bool, script: &str) -> Option<Self> {
        if !ENABLED.load(Ordering::Relaxed)
            || std::env::var_os("VIZE_CONFIG_CACHE").is_some_and(|value| value == "0")
        {
            return None;
        }
        let node_modules = config
            .ancestors()
            .skip(1)
            .map(|directory| directory.join("node_modules"))
            .find(|directory| directory.is_dir())?;
        let mut runtime = rustc_hash::FxHasher::default();
        script.hash(&mut runtime);
        let key = Key {
            format: FORMAT,
            version: env!("CARGO_PKG_VERSION").into(),
            runtime: runtime.finish(),
            config: config.to_path_buf(),
            explicit,
        };
        let mut name = rustc_hash::FxHasher::default();
        (&key.config, explicit).hash(&mut name);
        let file = crate::cstr!("{:016x}.json", name.finish());
        Some(Self {
            path: node_modules
                .join(".vize")
                .join("config-cache")
                .join(file.as_str()),
            key,
            others: Vec::new(),
        })
    }

    /// A stored evaluation none of whose inputs changed.
    pub(super) fn load(&mut self) -> Option<CachedEvaluation> {
        let bytes = std::fs::read(&self.path).ok()?;
        let entries: Vec<Entry> = serde_json::from_slice(&bytes).ok()?;
        for entry in entries {
            if entry.key != self.key || !entry.inputs_fresh() {
                continue;
            }
            if entry.env_fresh() {
                return Some(entry.evaluation);
            }
            self.others.push(entry);
        }
        None
    }

    /// Store a successful evaluation unless an input could still be changing.
    pub(super) fn store(
        self,
        started: SystemTime,
        evaluation: &CachedEvaluation,
        recorded: RecordedInputs,
    ) {
        if recorded.opaque {
            return;
        }
        let mut paths = BTreeMap::new();
        for (path, listing) in recorded.files {
            *paths.entry(path).or_insert(false) |= listing;
        }
        let imported = evaluation.dependencies.iter().map(PathBuf::from);
        for path in imported.chain(evaluation.watch_dependencies.iter().cloned()) {
            paths.entry(path).or_insert(false);
        }
        paths.insert(self.key.config.clone(), false);
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(node) = which::which("node") {
            paths.insert(node.canonicalize().unwrap_or(node), false);
        }
        let recent = started
            .checked_sub(RACY_WINDOW)
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map_or(u64::MAX, nanos);
        let mut inputs = Vec::with_capacity(paths.len());
        for (path, listing) in paths {
            let fingerprint = fingerprint(&path, listing);
            if fingerprint.modified >= recent {
                return;
            }
            inputs.push((path, listing, fingerprint));
        }
        let mut env = BTreeMap::new();
        let runtime = RUNTIME_ENV.map(|name| (name.into(), true));
        for (name, value) in recorded.env.into_iter().chain(runtime) {
            *env.entry(name).or_insert(false) |= value;
        }
        let entry = Entry {
            env: env
                .into_iter()
                .map(|(name, value)| {
                    let read = EnvRead::current(&name, value);
                    (name, read)
                })
                .collect(),
            env_names: recorded.env_enumerated.then(env_names_digest),
            key: self.key,
            inputs,
            evaluation,
        };
        // Best effort: an unwritable cache only costs the next evaluation.
        let _ = write_entries(&self.path, &entry, &self.others);
    }
}

fn write_entries(
    path: &Path,
    entry: &Entry<&CachedEvaluation>,
    others: &[Entry],
) -> std::io::Result<()> {
    let mut bytes = vec![b'['];
    bytes.extend(serde_json::to_vec(entry)?);
    for other in others.iter().take(ENTRIES - 1) {
        bytes.push(b',');
        bytes.extend(serde_json::to_vec(other)?);
    }
    bytes.push(b']');
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(directory)?;
    let mut file = tempfile::NamedTempFile::new_in(directory)?;
    file.write_all(&bytes)?;
    file.persist(path).map_err(|error| error.error)?;
    Ok(())
}

fn digest(bytes: &[u8]) -> std::string::String {
    let mut hex = std::string::String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        hex.push_str(crate::cstr!("{byte:02x}").as_str());
    }
    hex
}

fn env_names_digest() -> std::string::String {
    let mut names: Vec<_> = std::env::vars_os().map(|(name, _)| name).collect();
    names.sort();
    let mut bytes = Vec::new();
    for name in &names {
        bytes.extend_from_slice(name.as_encoded_bytes());
        bytes.push(0);
    }
    digest(&bytes)
}

fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn since_epoch(time: std::io::Result<SystemTime>) -> u64 {
    time.ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, nanos)
}

/// Kind, size, timestamps and identity; a directory's mtime only when its
/// listing was read, since entries added below it do not change its children.
fn fingerprint(path: &Path, listing: bool) -> Fingerprint {
    let link = std::fs::symlink_metadata(path)
        .ok()
        .filter(|metadata| metadata.file_type().is_symlink())
        .and_then(|_| std::fs::read_link(path).ok());
    let Ok(metadata) = std::fs::metadata(path) else {
        return Fingerprint {
            link,
            ..Fingerprint::default()
        };
    };
    if metadata.is_dir() {
        return Fingerprint {
            kind: 2,
            modified: if listing {
                since_epoch(metadata.modified())
            } else {
                0
            },
            link,
            ..Fingerprint::default()
        };
    }
    #[cfg(unix)]
    let (changed, inode) = {
        use std::os::unix::fs::MetadataExt;
        let seconds = u64::try_from(metadata.ctime()).unwrap_or(0);
        let nanos = u64::try_from(metadata.ctime_nsec()).unwrap_or(0);
        (
            seconds.saturating_mul(1_000_000_000).saturating_add(nanos),
            metadata.ino(),
        )
    };
    #[cfg(not(unix))]
    let (changed, inode) = (0, 0);
    Fingerprint {
        kind: if metadata.is_file() { 1 } else { 3 },
        len: metadata.len(),
        modified: since_epoch(metadata.modified()),
        changed,
        inode,
        link,
    }
}
