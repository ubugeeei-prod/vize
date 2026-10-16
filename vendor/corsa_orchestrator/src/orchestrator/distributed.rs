use super::{
    api::{ApiOrchestrator, ApiOrchestratorConfig},
    raft::RaftCluster,
    state::{ReplicatedCacheEntry, ReplicatedCommand, ReplicatedSnapshot, ReplicatedState},
};
use crate::{
    CorsaError, Result,
    api::{ApiClient, ApiProfile, ManagedSnapshot, UpdateSnapshotParams},
    lsp::{VirtualChange, VirtualDocument},
};
use corsa_core::fast::{CompactString, SmallVec, compact_format};
use lsp_types::Uri;
use serde::{Serialize, de::DeserializeOwned};
use std::{future::Future, sync::Arc, time::Duration};

/// Distributed orchestrator that mirrors state through a Raft cluster.
///
/// This type layers replication and leader-based mutation ordering on top of
/// the local [`ApiOrchestrator`]. The underlying [`RaftCluster`] offers
/// pluggable storage, pluggable transport, randomized election timing,
/// simplified single-server membership changes, and snapshot-based log
/// compaction. The whole distributed layer — this surface and the Raft
/// implementation beneath it — is gated by the `experimental-distributed`
/// cargo feature and sits outside the production support commitment (see
/// `docs/support_policy.md`).
///
/// # Examples
///
/// ```
/// use corsa_lsp::VirtualDocument;
/// use corsa_orchestrator::DistributedApiOrchestrator;
///
/// let cluster = DistributedApiOrchestrator::new(["node-a", "node-b", "node-c"]);
/// cluster.campaign("node-a")?;
///
/// let document = VirtualDocument::in_memory("cluster", "/main.ts", "typescript", "let x = 1;")?;
/// cluster.open_virtual_document("node-a", document.clone())?;
///
/// assert_eq!(
///     cluster.document("node-a", &document.uri).unwrap().text,
///     "let x = 1;"
/// );
/// # Ok::<(), corsa_orchestrator::CorsaError>(())
/// ```
#[derive(Clone)]
pub struct DistributedApiOrchestrator {
    local: Arc<ApiOrchestrator>,
    raft: RaftCluster,
}

impl DistributedApiOrchestrator {
    /// Creates a new distributed orchestrator with the given node identifiers.
    pub fn new<I, S>(node_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<CompactString>,
    {
        Self::with_local_config(node_ids, ApiOrchestratorConfig::default())
    }

    /// Creates a new distributed orchestrator with explicit local cache limits.
    pub fn with_local_config<I, S>(node_ids: I, local_config: ApiOrchestratorConfig) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<CompactString>,
    {
        Self {
            local: Arc::new(ApiOrchestrator::new(local_config)),
            raft: RaftCluster::new(node_ids),
        }
    }

    /// Returns the underlying Raft cluster.
    ///
    /// Advanced callers can inspect roles and replicated state directly through
    /// the returned handle.
    pub fn raft(&self) -> &RaftCluster {
        &self.raft
    }

    /// Starts a leader election for `node_id`.
    pub fn campaign(&self, node_id: &str) -> Result<u64> {
        self.raft.campaign(node_id)
    }

    /// Returns the current leader, if any.
    pub fn leader_id(&self) -> Option<CompactString> {
        self.raft.leader_id()
    }

    /// Returns the leader state, or the first node when no leader is elected.
    pub fn state(&self) -> Option<ReplicatedState> {
        self.raft.state()
    }

    /// Returns the replicated state stored on a specific node.
    pub fn node_state(&self, node_id: &str) -> Option<ReplicatedState> {
        self.raft.node_state(node_id)
    }

    /// Looks up a replicated virtual document on a node.
    pub fn document(&self, node_id: &str, uri: &Uri) -> Option<VirtualDocument> {
        self.node_state(node_id)?
            .documents
            .get(uri.as_str())
            .cloned()
    }

    /// Looks up a replicated snapshot record on a node.
    pub fn snapshot_record(&self, node_id: &str, key: &str) -> Option<ReplicatedSnapshot> {
        self.node_state(node_id)?.snapshots.get(key).cloned()
    }

    /// Replicates a newly opened virtual document.
    ///
    /// The supplied `leader_id` must currently be allowed to append commands.
    pub fn open_virtual_document(
        &self,
        leader_id: &str,
        document: VirtualDocument,
    ) -> Result<VirtualDocument> {
        self.raft.append(
            leader_id,
            ReplicatedCommand::PutDocument {
                document: document.clone(),
            },
        )?;
        Ok(document)
    }

    /// Applies and replicates incremental changes for a virtual document.
    ///
    /// The change is first validated locally against the current replicated
    /// document contents, then appended to the Raft log.
    pub fn change_virtual_document(
        &self,
        leader_id: &str,
        uri: &Uri,
        changes: impl IntoIterator<Item = VirtualChange>,
    ) -> Result<VirtualDocument> {
        let changes = changes
            .into_iter()
            .collect::<SmallVec<[VirtualChange; 4]>>();
        let mut document = self.document(leader_id, uri).ok_or_else(|| {
            CorsaError::Protocol(compact_format(format_args!(
                "unknown virtual document: {}",
                uri.as_str()
            )))
        })?;
        document.apply_changes(&changes)?;
        self.raft.append(
            leader_id,
            ReplicatedCommand::ApplyDocumentChange {
                uri: CompactString::from(uri.as_str()),
                changes: changes.into_iter().collect(),
            },
        )?;
        Ok(document)
    }

    /// Removes a replicated virtual document.
    pub fn close_virtual_document(&self, leader_id: &str, uri: &Uri) -> Result<()> {
        if self.document(leader_id, uri).is_none() {
            return Err(CorsaError::Protocol(compact_format(format_args!(
                "unknown virtual document: {}",
                uri.as_str()
            ))));
        }
        self.raft.append(
            leader_id,
            ReplicatedCommand::RemoveDocument {
                uri: CompactString::from(uri.as_str()),
            },
        )?;
        Ok(())
    }

    /// Performs a cached computation and replicates the result through Raft.
    ///
    /// Reads prefer the replicated cache first. On a miss, the value is
    /// computed locally and then published to the cluster log.
    pub async fn cached<T, F, Fut>(
        &self,
        profile: &ApiProfile,
        leader_id: &str,
        key: impl Into<CompactString>,
        ttl: Option<Duration>,
        task: F,
    ) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce(ApiClient) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let key = key.into();
        if let Some(value) = self
            .state()
            .map(|state| state.result(key.as_str()))
            .transpose()?
            .flatten()
        {
            return Ok(value);
        }
        let value = task(self.local.lease(profile).await?).await?;
        self.raft.append(
            leader_id,
            ReplicatedCommand::PutResult {
                key,
                entry: ReplicatedCacheEntry::encode(&value, ttl)?,
            },
        )?;
        Ok(value)
    }

    /// Creates or reuses a snapshot locally and mirrors its record to the cluster.
    ///
    /// Only the snapshot metadata is replicated. The underlying live snapshot
    /// handle remains local to the current process.
    pub async fn cached_snapshot(
        &self,
        profile: &ApiProfile,
        leader_id: &str,
        key: impl Into<CompactString>,
        params: UpdateSnapshotParams,
    ) -> Result<Arc<ManagedSnapshot>> {
        let key = key.into();
        let snapshot = self
            .local
            .cached_snapshot(profile, key.clone(), params)
            .await?;
        self.raft.append(
            leader_id,
            ReplicatedCommand::PutSnapshot {
                key,
                snapshot: ReplicatedSnapshot::from(snapshot.as_ref()),
            },
        )?;
        Ok(snapshot)
    }

    /// Invalidates a replicated snapshot record.
    ///
    /// This removes both the local snapshot cache entry and the replicated
    /// record associated with `key`.
    pub fn invalidate_snapshot(&self, leader_id: &str, key: &str) -> Result<()> {
        self.local.invalidate_snapshot(key);
        self.raft.append(
            leader_id,
            ReplicatedCommand::RemoveSnapshot {
                key: CompactString::from(key),
            },
        )?;
        Ok(())
    }

    /// Invalidates a replicated cached result.
    pub fn invalidate_cached(&self, leader_id: &str, key: &str) -> Result<()> {
        self.local.invalidate_cached(key);
        self.raft.append(
            leader_id,
            ReplicatedCommand::RemoveResult {
                key: CompactString::from(key),
            },
        )?;
        Ok(())
    }
}
