use crate::{
    CorsaError, Result,
    api::{ManagedSnapshot, ProjectResponse, SnapshotChanges, SnapshotHandle},
    lsp::{VirtualChange, VirtualDocument},
};
use corsa_core::fast::{CompactString, FastMap, SmallVec, compact_format};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Snapshot metadata replicated through the distributed orchestrator.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicatedSnapshot {
    /// Snapshot handle on the node that created the snapshot.
    pub handle: SnapshotHandle,
    /// Project list visible when the snapshot was created.
    pub projects: SmallVec<[ProjectResponse; 4]>,
    /// Optional incremental change summary returned by Corsa.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<SnapshotChanges>,
}

/// Cached result entry replicated through the distributed orchestrator.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicatedCacheEntry {
    /// Absolute expiration time in Unix milliseconds, or `None` for immortal entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_unix_ms: Option<u64>,
    /// JSON-encoded value bytes.
    pub bytes: SmallVec<[u8; 256]>,
}

/// Deterministic state machine replicated by the in-process Raft cluster.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReplicatedState {
    /// Open virtual documents keyed by URI.
    pub documents: FastMap<CompactString, VirtualDocument>,
    /// Snapshot metadata keyed by application-defined cache key.
    pub snapshots: FastMap<CompactString, ReplicatedSnapshot>,
    /// Cached result payloads keyed by application-defined cache key.
    pub results: FastMap<CompactString, ReplicatedCacheEntry>,
}

/// Commands that mutate the replicated state machine.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum ReplicatedCommand {
    /// Inserts or replaces a replicated virtual document.
    PutDocument { document: VirtualDocument },
    /// Applies incremental changes to an existing virtual document.
    ApplyDocumentChange {
        uri: CompactString,
        changes: SmallVec<[VirtualChange; 4]>,
    },
    /// Removes a virtual document.
    RemoveDocument { uri: CompactString },
    /// Inserts or replaces snapshot metadata.
    PutSnapshot {
        key: CompactString,
        snapshot: ReplicatedSnapshot,
    },
    /// Removes snapshot metadata.
    RemoveSnapshot { key: CompactString },
    /// Inserts or replaces a cached result entry.
    PutResult {
        key: CompactString,
        entry: ReplicatedCacheEntry,
    },
    /// Removes a cached result entry.
    RemoveResult { key: CompactString },
}

impl From<&ManagedSnapshot> for ReplicatedSnapshot {
    fn from(value: &ManagedSnapshot) -> Self {
        Self {
            handle: value.handle.clone(),
            projects: value.projects.iter().cloned().collect(),
            changes: value.changes.clone(),
        }
    }
}

impl ReplicatedCacheEntry {
    /// Creates a new cache entry with an optional TTL.
    pub fn new(bytes: SmallVec<[u8; 256]>, ttl: Option<Duration>) -> Self {
        Self {
            expires_at_unix_ms: ttl.map(|ttl| now_unix_ms() + ttl.as_millis() as u64),
            bytes,
        }
    }

    /// Serializes a value into a replicated cache entry.
    pub fn encode<T>(value: &T, ttl: Option<Duration>) -> Result<Self>
    where
        T: Serialize,
    {
        Ok(Self::new(
            serde_json::to_vec(value)?.into_iter().collect(),
            ttl,
        ))
    }

    /// Deserializes the cached bytes back into a typed value.
    pub fn decode<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_slice(&self.bytes)?)
    }

    /// Returns whether the entry should still be treated as valid.
    pub fn is_fresh(&self) -> bool {
        self.expires_at_unix_ms
            .map(|deadline| deadline > now_unix_ms())
            .unwrap_or(true)
    }
}

impl ReplicatedState {
    /// Applies a single replicated command to the local state machine.
    ///
    /// Commands are deterministic and order-sensitive; callers are expected to
    /// feed them in Raft log order.
    pub fn apply(&mut self, command: &ReplicatedCommand) -> Result<()> {
        match command {
            ReplicatedCommand::PutDocument { document } => {
                self.documents.insert(document.key(), document.clone());
            }
            ReplicatedCommand::ApplyDocumentChange { uri, changes } => {
                let document = self.documents.get_mut(uri.as_str()).ok_or_else(|| {
                    CorsaError::Protocol(compact_format(format_args!(
                        "unknown replicated document: {uri}"
                    )))
                })?;
                // Reuse `VirtualDocument`'s UTF-16 aware editing logic so the
                // replicated model matches the local overlay behavior exactly.
                document.apply_changes(changes)?;
            }
            ReplicatedCommand::RemoveDocument { uri } => {
                self.documents.remove(uri.as_str());
            }
            ReplicatedCommand::PutSnapshot { key, snapshot } => {
                self.snapshots.insert(key.clone(), snapshot.clone());
            }
            ReplicatedCommand::RemoveSnapshot { key } => {
                self.snapshots.remove(key.as_str());
            }
            ReplicatedCommand::PutResult { key, entry } => {
                self.results.insert(key.clone(), entry.clone());
            }
            ReplicatedCommand::RemoveResult { key } => {
                self.results.remove(key.as_str());
            }
        }
        Ok(())
    }

    /// Looks up and decodes a cached result when it is still fresh.
    pub fn result<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        self.results
            .get(key)
            .filter(|entry| entry.is_fresh())
            .map(ReplicatedCacheEntry::decode)
            .transpose()
    }
}

/// Returns the current Unix time in milliseconds.
fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
