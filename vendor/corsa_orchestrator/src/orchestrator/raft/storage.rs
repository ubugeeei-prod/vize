//! Pluggable durable storage for Raft state.
//!
//! Raft requires three pieces of state to survive node restarts:
//!
//! 1. The **hard state** (`current_term`, `voted_for`) so a node never grants
//!    two votes in the same term.
//! 2. The **log** of [`PersistedLogEntry`] values so committed entries are
//!    not lost.
//! 3. Any **state machine snapshots** taken for log compaction.
//!
//! The trait below isolates the policy of *how* that state is persisted from
//! the Raft state machine. Production deployments wire a file- or
//! database-backed implementation; tests use the in-memory implementation.

use super::messages::{PersistedLogEntry, RaftSnapshot};
use crate::{CorsaError, Result};
use corsa_core::fast::{CompactString, FastMap, SmallVec, compact_format};
use parking_lot::RwLock;
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

/// Durable Raft state for one node, separately from the log.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HardState {
    /// Latest term the node has seen.
    pub current_term: u64,
    /// Candidate the node has voted for in the current term, if any.
    pub voted_for: Option<CompactString>,
}

/// Pluggable persistent storage for one Raft node.
///
/// Implementations are required to provide atomic-with-respect-to-crashes
/// semantics within a single method call. The Raft driver calls these
/// methods on the hot path of every term change and every log append.
pub trait RaftStorage: Send + Sync {
    /// Persists the latest `(current_term, voted_for)` pair.
    fn save_hard_state(&self, node_id: &str, state: &HardState) -> Result<()>;
    /// Loads the persisted hard state, defaulting to `(0, None)` for a fresh
    /// node.
    fn load_hard_state(&self, node_id: &str) -> Result<HardState>;

    /// Appends new entries to the log, replacing anything at the same indices
    /// or beyond. `start_index` is the 1-based index of the first new entry.
    fn append_log(
        &self,
        node_id: &str,
        start_index: u64,
        entries: &[PersistedLogEntry],
    ) -> Result<()>;

    /// Truncates the log so the highest retained 1-based index is `last_index`
    /// (truncates from the *end*; used by leaders rejecting a stale tail).
    fn truncate_log_suffix(&self, node_id: &str, last_index: u64) -> Result<()>;

    /// Compacts the log prefix by discarding entries with 1-based index at or
    /// below `last_included_index`. Used after a snapshot is durably stored.
    fn compact_log_prefix(&self, node_id: &str, last_included_index: u64) -> Result<()>;

    /// Reads the persisted log, returning all entries in 1-based index order.
    fn load_log(&self, node_id: &str) -> Result<SmallVec<[PersistedLogEntry; 32]>>;

    /// Returns the 1-based index of the snapshot that the durable log was
    /// compacted to, or `0` when no snapshot exists.
    fn log_base_index(&self, node_id: &str) -> Result<u64>;

    /// Returns the term of the snapshot the log was compacted to, or `0`.
    fn log_base_term(&self, node_id: &str) -> Result<u64>;

    /// Persists a new state machine snapshot for `node_id`.
    fn save_snapshot(&self, node_id: &str, snapshot: &RaftSnapshot) -> Result<()>;

    /// Loads the most recent persisted snapshot, if any.
    fn load_snapshot(&self, node_id: &str) -> Result<Option<RaftSnapshot>>;
}

// --- In-memory implementation ------------------------------------------------

#[derive(Default)]
struct InMemoryEntry {
    hard_state: HardState,
    log: SmallVec<[PersistedLogEntry; 32]>,
    log_base_index: u64,
    log_base_term: u64,
    snapshot: Option<RaftSnapshot>,
}

/// Process-local storage suitable for unit tests and single-process clusters.
///
/// Cloning the [`InMemoryStorage`] handle shares the same underlying data; it
/// is therefore safe (and useful) to construct one storage and hand a clone
/// to each node in a synthetic cluster.
#[derive(Clone, Default)]
pub struct InMemoryStorage {
    inner: Arc<RwLock<FastMap<CompactString, InMemoryEntry>>>,
}

impl InMemoryStorage {
    /// Creates a fresh, empty storage instance.
    pub fn new() -> Self {
        Self::default()
    }

    fn with_entry<R>(
        &self,
        node_id: &str,
        with: impl FnOnce(&mut InMemoryEntry) -> Result<R>,
    ) -> Result<R> {
        let mut map = self.inner.write();
        let key = CompactString::from(node_id);
        let entry = map.entry(key).or_default();
        with(entry)
    }
}

impl RaftStorage for InMemoryStorage {
    fn save_hard_state(&self, node_id: &str, state: &HardState) -> Result<()> {
        self.with_entry(node_id, |entry| {
            entry.hard_state = state.clone();
            Ok(())
        })
    }

    fn load_hard_state(&self, node_id: &str) -> Result<HardState> {
        Ok(self
            .inner
            .read()
            .get(node_id)
            .map(|entry| entry.hard_state.clone())
            .unwrap_or_default())
    }

    fn append_log(
        &self,
        node_id: &str,
        start_index: u64,
        entries: &[PersistedLogEntry],
    ) -> Result<()> {
        self.with_entry(node_id, |entry| {
            if start_index == 0 {
                return Err(CorsaError::Protocol(CompactString::from(
                    "raft log indices are 1-based; cannot append at index 0",
                )));
            }
            let base = entry.log_base_index;
            if start_index <= base {
                return Err(CorsaError::Protocol(compact_format(format_args!(
                    "raft log append below snapshot prefix: {start_index} <= {base}"
                ))));
            }
            let local_start = (start_index - base - 1) as usize;
            if local_start > entry.log.len() {
                return Err(CorsaError::Protocol(compact_format(format_args!(
                    "raft log append leaves a gap: local_start={local_start} len={}",
                    entry.log.len()
                ))));
            }
            entry.log.truncate(local_start);
            for new_entry in entries {
                entry.log.push(new_entry.clone());
            }
            Ok(())
        })
    }

    fn truncate_log_suffix(&self, node_id: &str, last_index: u64) -> Result<()> {
        self.with_entry(node_id, |entry| {
            let base = entry.log_base_index;
            if last_index < base {
                return Err(CorsaError::Protocol(compact_format(format_args!(
                    "raft log truncate below snapshot prefix: {last_index} < {base}"
                ))));
            }
            let retain = (last_index - base) as usize;
            entry.log.truncate(retain);
            Ok(())
        })
    }

    fn compact_log_prefix(&self, node_id: &str, last_included_index: u64) -> Result<()> {
        self.with_entry(node_id, |entry| {
            let base = entry.log_base_index;
            if last_included_index <= base {
                return Ok(());
            }
            let drop = (last_included_index - base) as usize;
            if drop >= entry.log.len() {
                // Final entry of the dropped range carries the snapshot term.
                if let Some(last) = entry.log.last() {
                    entry.log_base_term = last.term;
                }
                entry.log.clear();
            } else {
                entry.log_base_term = entry.log[drop - 1].term;
                let remaining: SmallVec<[PersistedLogEntry; 32]> =
                    entry.log.drain(..drop).collect::<SmallVec<_>>();
                let _ = remaining;
            }
            entry.log_base_index = last_included_index;
            Ok(())
        })
    }

    fn load_log(&self, node_id: &str) -> Result<SmallVec<[PersistedLogEntry; 32]>> {
        Ok(self
            .inner
            .read()
            .get(node_id)
            .map(|entry| entry.log.clone())
            .unwrap_or_default())
    }

    fn log_base_index(&self, node_id: &str) -> Result<u64> {
        Ok(self
            .inner
            .read()
            .get(node_id)
            .map(|entry| entry.log_base_index)
            .unwrap_or(0))
    }

    fn log_base_term(&self, node_id: &str) -> Result<u64> {
        Ok(self
            .inner
            .read()
            .get(node_id)
            .map(|entry| entry.log_base_term)
            .unwrap_or(0))
    }

    fn save_snapshot(&self, node_id: &str, snapshot: &RaftSnapshot) -> Result<()> {
        self.with_entry(node_id, |entry| {
            entry.snapshot = Some(snapshot.clone());
            entry.log_base_index = snapshot.last_included_index;
            entry.log_base_term = snapshot.last_included_term;
            Ok(())
        })
    }

    fn load_snapshot(&self, node_id: &str) -> Result<Option<RaftSnapshot>> {
        Ok(self
            .inner
            .read()
            .get(node_id)
            .and_then(|entry| entry.snapshot.clone()))
    }
}

// --- File-backed implementation ---------------------------------------------

/// File-system backed [`RaftStorage`] suitable for production single-node
/// disk persistence.
///
/// Each node identifier maps to a directory under `root` containing:
///
/// - `hard_state.json` — atomically rewritten on every term/vote change
/// - `log.bin` — append-only log entries serialized with `serde_json`,
///   followed by a newline; the file is rewritten on truncate/compact
/// - `snapshot.bin` — the most recently written snapshot
///
/// On disk durability is achieved by writing to a temp file in the same
/// directory and `fs::rename`ing it over the target. Callers that need
/// stricter `fsync` guarantees on individual writes can wrap this with a
/// custom implementation; the `FileStorage` here is robust against crashes
/// in the *middle* of a write because the new file is only renamed into
/// place after it has been fully flushed.
pub struct FileStorage {
    root: PathBuf,
    cache: RwLock<FastMap<CompactString, InMemoryEntry>>,
}

impl FileStorage {
    /// Creates a [`FileStorage`] rooted at `path`.
    ///
    /// The directory is created if missing.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let root = path.into();
        fs::create_dir_all(&root).map_err(CorsaError::from)?;
        Ok(Self {
            root,
            cache: RwLock::new(FastMap::default()),
        })
    }

    fn node_dir(&self, node_id: &str) -> PathBuf {
        self.root.join(node_id)
    }

    fn ensure_node_dir(&self, node_id: &str) -> Result<PathBuf> {
        let dir = self.node_dir(node_id);
        fs::create_dir_all(&dir).map_err(CorsaError::from)?;
        Ok(dir)
    }

    fn write_atomically(target: &Path, bytes: &[u8]) -> Result<()> {
        let parent = target.parent().ok_or_else(|| {
            CorsaError::Protocol(CompactString::from(
                "raft file storage target has no parent",
            ))
        })?;
        let mut tmp = parent.join(
            target
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        );
        tmp.set_extension("tmp");
        {
            let mut file = fs::File::create(&tmp).map_err(CorsaError::from)?;
            file.write_all(bytes).map_err(CorsaError::from)?;
            file.flush().map_err(CorsaError::from)?;
            // Best-effort fsync; missing on some filesystems but never an error.
            let _ = file.sync_all();
        }
        fs::rename(&tmp, target).map_err(CorsaError::from)?;
        Ok(())
    }

    fn read_optional(path: &Path) -> Result<Option<SmallVec<[u8; 256]>>> {
        match fs::File::open(path) {
            Ok(mut file) => {
                let mut buffer = SmallVec::<[u8; 256]>::new();
                let mut tmp = [0u8; 4096];
                loop {
                    let read = file.read(&mut tmp).map_err(CorsaError::from)?;
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&tmp[..read]);
                }
                Ok(Some(buffer))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(CorsaError::from(error)),
        }
    }

    fn load_entry(&self, node_id: &str) -> Result<InMemoryEntry> {
        let dir = self.node_dir(node_id);
        let hard_state = Self::read_optional(&dir.join("hard_state.json"))?
            .map(|bytes| serde_json::from_slice::<PersistedHardState>(&bytes))
            .transpose()?
            .map(HardState::from)
            .unwrap_or_default();
        let log = Self::read_optional(&dir.join("log.json"))?
            .map(|bytes| serde_json::from_slice::<PersistedLog>(&bytes))
            .transpose()?
            .map(|persisted| persisted.entries)
            .unwrap_or_default();
        let snapshot = Self::read_optional(&dir.join("snapshot.json"))?
            .map(|bytes| serde_json::from_slice::<RaftSnapshot>(&bytes))
            .transpose()?;
        let (log_base_index, log_base_term) = snapshot
            .as_ref()
            .map(|s| (s.last_included_index, s.last_included_term))
            .unwrap_or((0, 0));
        Ok(InMemoryEntry {
            hard_state,
            log,
            log_base_index,
            log_base_term,
            snapshot,
        })
    }

    fn with_entry<R>(
        &self,
        node_id: &str,
        with: impl FnOnce(&mut InMemoryEntry) -> Result<R>,
    ) -> Result<R> {
        let key = CompactString::from(node_id);
        let mut cache = self.cache.write();
        if !cache.contains_key(key.as_str()) {
            let entry = self.load_entry(node_id)?;
            cache.insert(key.clone(), entry);
        }
        let entry = cache.get_mut(key.as_str()).expect("just inserted");
        with(entry)
    }

    fn flush_hard_state(&self, node_id: &str, state: &HardState) -> Result<()> {
        let dir = self.ensure_node_dir(node_id)?;
        let bytes = serde_json::to_vec(&PersistedHardState::from(state.clone()))?;
        Self::write_atomically(&dir.join("hard_state.json"), &bytes)
    }

    fn flush_log(&self, node_id: &str, entries: &[PersistedLogEntry]) -> Result<()> {
        let dir = self.ensure_node_dir(node_id)?;
        let persisted = PersistedLog {
            entries: entries.iter().cloned().collect(),
        };
        let bytes = serde_json::to_vec(&persisted)?;
        Self::write_atomically(&dir.join("log.json"), &bytes)
    }

    fn flush_snapshot(&self, node_id: &str, snapshot: &RaftSnapshot) -> Result<()> {
        let dir = self.ensure_node_dir(node_id)?;
        let bytes = serde_json::to_vec(snapshot)?;
        Self::write_atomically(&dir.join("snapshot.json"), &bytes)
    }
}

impl RaftStorage for FileStorage {
    fn save_hard_state(&self, node_id: &str, state: &HardState) -> Result<()> {
        self.with_entry(node_id, |entry| {
            entry.hard_state = state.clone();
            Ok(())
        })?;
        self.flush_hard_state(node_id, state)
    }

    fn load_hard_state(&self, node_id: &str) -> Result<HardState> {
        self.with_entry(node_id, |entry| Ok(entry.hard_state.clone()))
    }

    fn append_log(
        &self,
        node_id: &str,
        start_index: u64,
        entries: &[PersistedLogEntry],
    ) -> Result<()> {
        let log = self.with_entry(node_id, |entry| {
            if start_index == 0 {
                return Err(CorsaError::Protocol(CompactString::from(
                    "raft log indices are 1-based; cannot append at index 0",
                )));
            }
            let base = entry.log_base_index;
            if start_index <= base {
                return Err(CorsaError::Protocol(compact_format(format_args!(
                    "raft log append below snapshot prefix: {start_index} <= {base}"
                ))));
            }
            let local_start = (start_index - base - 1) as usize;
            if local_start > entry.log.len() {
                return Err(CorsaError::Protocol(compact_format(format_args!(
                    "raft log append leaves a gap: local_start={local_start} len={}",
                    entry.log.len()
                ))));
            }
            entry.log.truncate(local_start);
            for new_entry in entries {
                entry.log.push(new_entry.clone());
            }
            Ok(entry.log.clone())
        })?;
        self.flush_log(node_id, &log)
    }

    fn truncate_log_suffix(&self, node_id: &str, last_index: u64) -> Result<()> {
        let log = self.with_entry(node_id, |entry| {
            let base = entry.log_base_index;
            if last_index < base {
                return Err(CorsaError::Protocol(compact_format(format_args!(
                    "raft log truncate below snapshot prefix: {last_index} < {base}"
                ))));
            }
            let retain = (last_index - base) as usize;
            entry.log.truncate(retain);
            Ok(entry.log.clone())
        })?;
        self.flush_log(node_id, &log)
    }

    fn compact_log_prefix(&self, node_id: &str, last_included_index: u64) -> Result<()> {
        let log = self.with_entry(node_id, |entry| {
            let base = entry.log_base_index;
            if last_included_index <= base {
                return Ok(entry.log.clone());
            }
            let drop = (last_included_index - base) as usize;
            if drop >= entry.log.len() {
                if let Some(last) = entry.log.last() {
                    entry.log_base_term = last.term;
                }
                entry.log.clear();
            } else {
                entry.log_base_term = entry.log[drop - 1].term;
                let _: SmallVec<[PersistedLogEntry; 32]> =
                    entry.log.drain(..drop).collect::<SmallVec<_>>();
            }
            entry.log_base_index = last_included_index;
            Ok(entry.log.clone())
        })?;
        self.flush_log(node_id, &log)
    }

    fn load_log(&self, node_id: &str) -> Result<SmallVec<[PersistedLogEntry; 32]>> {
        self.with_entry(node_id, |entry| Ok(entry.log.clone()))
    }

    fn log_base_index(&self, node_id: &str) -> Result<u64> {
        self.with_entry(node_id, |entry| Ok(entry.log_base_index))
    }

    fn log_base_term(&self, node_id: &str) -> Result<u64> {
        self.with_entry(node_id, |entry| Ok(entry.log_base_term))
    }

    fn save_snapshot(&self, node_id: &str, snapshot: &RaftSnapshot) -> Result<()> {
        self.with_entry(node_id, |entry| {
            entry.snapshot = Some(snapshot.clone());
            entry.log_base_index = snapshot.last_included_index;
            entry.log_base_term = snapshot.last_included_term;
            Ok(())
        })?;
        self.flush_snapshot(node_id, snapshot)
    }

    fn load_snapshot(&self, node_id: &str) -> Result<Option<RaftSnapshot>> {
        self.with_entry(node_id, |entry| Ok(entry.snapshot.clone()))
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PersistedHardState {
    current_term: u64,
    voted_for: Option<String>,
}

impl From<HardState> for PersistedHardState {
    fn from(value: HardState) -> Self {
        Self {
            current_term: value.current_term,
            voted_for: value.voted_for.map(|v| v.into_string()),
        }
    }
}

impl From<PersistedHardState> for HardState {
    fn from(value: PersistedHardState) -> Self {
        Self {
            current_term: value.current_term,
            voted_for: value.voted_for.map(CompactString::from),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct PersistedLog {
    entries: SmallVec<[PersistedLogEntry; 32]>,
}
