//! Production-grade Raft consensus for the distributed orchestrator.
//!
//! This module is a from-scratch implementation of the Raft consensus
//! algorithm (Ongaro & Ousterhout, 2014). Unlike the earlier in-process
//! prototype, every interaction between nodes is mediated by an explicit
//! [`RaftMessage`] flowing through a pluggable [`RaftTransport`], every
//! durable change goes through a pluggable [`RaftStorage`], and election
//! timing is driven by [`RaftCluster::tick`] rather than being implicit
//! in API calls.
//!
//! # Mental model
//!
//! ```text
//!  +---------------+        +-------------------+        +---------------+
//!  |   Caller      |--prop->|   RaftCluster     |--send->|  Transport     |
//!  |               |<-state-|   (driver)        |<-recv--|  (in-process,  |
//!  +---------------+        |  - nodes          |        |   channel,     |
//!                           |  - storage        |        |   network)     |
//!                           |  - transport      |        +---------------+
//!                           +-------------------+
//! ```
//!
//! # Public API
//!
//! - [`RaftCluster::new`] and [`RaftCluster::with_config`] create a
//!   single-process cluster suitable for tests and synchronous workloads.
//!   Both use [`InProcessTransport`] under the hood, so calls like
//!   [`RaftCluster::campaign`] and [`RaftCluster::append`] drive the
//!   cluster to quiescence before returning.
//! - [`RaftCluster::builder`] returns a [`RaftClusterBuilder`] for fully
//!   custom production deployments: pick the [`RaftStorage`] (e.g.
//!   [`FileStorage`]) and [`RaftTransport`] you want, install your own
//!   tick driver, and wire each network message back through
//!   [`RaftCluster::step`].
//! - [`RaftCluster::tick`] advances election and heartbeat timers and
//!   should be called periodically by a driver thread in real deployments.
//! - [`RaftCluster::add_member`] and [`RaftCluster::remove_member`]
//!   support runtime membership changes.
//! - [`RaftCluster::compact`] takes an explicit state machine snapshot so
//!   long-lived clusters do not grow their log indefinitely.

mod config;
mod messages;
mod node;
mod storage;
mod transport;

#[cfg(test)]
mod tests;

use super::state::{ReplicatedCommand, ReplicatedState};
use crate::{CorsaError, Result};
use corsa_core::fast::{CompactString, FastMap, FastSet, SmallVec, compact_format};
use parking_lot::{Mutex, RwLock};
use std::{sync::Arc, time::Instant};

pub use config::RaftConfig;
pub use messages::{PersistedLogEntry, RaftMessage, RaftSnapshot};
pub use storage::{FileStorage, HardState, InMemoryStorage, RaftStorage};
#[allow(unused_imports)]
pub use transport::{ChannelTransport, InProcessTransport, RaftTransport, wait_for_drain};

use node::{Outbox, RaftNode};

/// Role of a node inside a Raft cluster.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaftRole {
    /// Node currently allowed to append commands.
    Leader,
    /// Passive replica following the leader.
    Follower,
    /// Node currently requesting votes for a new term.
    Candidate,
}

/// Builder used to assemble a [`RaftCluster`] with custom storage and
/// transport implementations.
pub struct RaftClusterBuilder {
    node_ids: SmallVec<[CompactString; 4]>,
    config: RaftConfig,
    storage: Option<Arc<dyn RaftStorage>>,
    transport: Option<Arc<dyn RaftTransport>>,
    in_process_transport: Option<InProcessTransport>,
}

impl RaftClusterBuilder {
    /// Begins a builder seeded with the given node identifiers.
    pub fn new<I, S>(node_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<CompactString>,
    {
        Self {
            node_ids: node_ids.into_iter().map(Into::into).collect(),
            config: RaftConfig::default(),
            storage: None,
            transport: None,
            in_process_transport: None,
        }
    }

    /// Overrides the tuning configuration.
    pub fn config(mut self, config: RaftConfig) -> Self {
        self.config = config;
        self
    }

    /// Overrides the persistent storage backend.
    pub fn storage(mut self, storage: Arc<dyn RaftStorage>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Overrides the transport backend.
    ///
    /// When supplied, the cluster gives up its synchronous "drive to quiescence"
    /// behavior on [`RaftCluster::campaign`] and [`RaftCluster::append`] —
    /// callers must arrange for messages to flow through the transport and
    /// back into [`RaftCluster::step`].
    pub fn transport(mut self, transport: Arc<dyn RaftTransport>) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Installs an [`InProcessTransport`] (the default) so the cluster keeps
    /// the synchronous high-level helpers working.
    pub fn in_process_transport(mut self, transport: InProcessTransport) -> Self {
        self.in_process_transport = Some(transport);
        self
    }

    /// Builds the cluster.
    pub fn build(self) -> Result<RaftCluster> {
        if !self.config.is_valid() {
            return Err(CorsaError::Protocol(CompactString::from(
                "raft config is invalid: heartbeat must be smaller than election_timeout_min",
            )));
        }
        let storage: Arc<dyn RaftStorage> = self
            .storage
            .unwrap_or_else(|| Arc::new(InMemoryStorage::new()));
        let in_process = if self.transport.is_none() {
            Some(self.in_process_transport.unwrap_or_default())
        } else {
            None
        };
        let transport: Arc<dyn RaftTransport> = match self.transport {
            Some(transport) => transport,
            None => Arc::new(in_process.clone().expect("in_process initialized above")),
        };
        let members: FastSet<CompactString> = self.node_ids.iter().cloned().collect();
        let now = Instant::now();
        let mut nodes = FastMap::default();
        for (index, id) in self.node_ids.iter().cloned().enumerate() {
            if let Some(transport) = in_process.as_ref() {
                transport.register(id.as_str());
            }
            let node = RaftNode::restore(
                id.clone(),
                members.clone(),
                self.config.clone(),
                storage.clone(),
                now,
                self.config.election_jitter_seed.wrapping_add(index as u64),
            )?;
            nodes.insert(id, Arc::new(Mutex::new(node)));
        }
        Ok(RaftCluster {
            inner: Arc::new(RaftClusterInner {
                config: self.config,
                storage,
                transport,
                in_process,
                nodes: RwLock::new(nodes),
            }),
        })
    }
}

struct RaftClusterInner {
    config: RaftConfig,
    storage: Arc<dyn RaftStorage>,
    transport: Arc<dyn RaftTransport>,
    in_process: Option<InProcessTransport>,
    nodes: RwLock<FastMap<CompactString, Arc<Mutex<RaftNode>>>>,
}

/// Minimal in-process Raft cluster used by the distributed orchestrator.
///
/// This implementation supports the entire Raft protocol:
///
/// - leader election with randomized timeouts and per-term vote tracking
/// - log replication with conflict-index backfill (§5.3 of the paper)
/// - safe commitment respecting the leader-completeness property (§5.4.2)
/// - cluster membership changes via [`Self::add_member`] /
///   [`Self::remove_member`]
/// - log compaction through [`Self::compact`] and `InstallSnapshot` RPCs
///
/// Single-process callers using the default [`InProcessTransport`] get a
/// synchronous API: [`Self::campaign`] runs the election and the initial
/// heartbeat round-trip before returning, [`Self::append`] returns after
/// the entry has been committed. Production deployments override the
/// transport via [`RaftClusterBuilder`] and drive the cluster with
/// [`Self::tick`] and [`Self::step`] from their own runtime.
#[derive(Clone)]
pub struct RaftCluster {
    inner: Arc<RaftClusterInner>,
}

impl RaftCluster {
    /// Creates a cluster containing the provided node identifiers, backed by
    /// in-memory storage and an in-process transport.
    pub fn new<I, S>(node_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<CompactString>,
    {
        Self::with_config(node_ids, RaftConfig::default())
    }

    /// Creates a cluster with the given configuration but otherwise default
    /// storage and transport.
    pub fn with_config<I, S>(node_ids: I, config: RaftConfig) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<CompactString>,
    {
        RaftClusterBuilder::new(node_ids)
            .config(config)
            .build()
            .expect("default RaftClusterBuilder configuration must be valid")
    }

    /// Starts a [`RaftClusterBuilder`] for fully custom deployments.
    pub fn builder<I, S>(node_ids: I) -> RaftClusterBuilder
    where
        I: IntoIterator<Item = S>,
        S: Into<CompactString>,
    {
        RaftClusterBuilder::new(node_ids)
    }

    /// Returns the cluster's tunable configuration.
    pub fn config(&self) -> &RaftConfig {
        &self.inner.config
    }

    /// Returns the storage backend handle.
    pub fn storage(&self) -> Arc<dyn RaftStorage> {
        self.inner.storage.clone()
    }

    /// Returns the transport backend handle.
    pub fn transport(&self) -> Arc<dyn RaftTransport> {
        self.inner.transport.clone()
    }

    /// Lists every node currently registered with the cluster.
    pub fn members(&self) -> SmallVec<[CompactString; 4]> {
        self.inner.nodes.read().keys().cloned().collect()
    }

    /// Returns the current leader identifier, if one has been elected.
    pub fn leader_id(&self) -> Option<CompactString> {
        let nodes = self.inner.nodes.read();
        for node in nodes.values() {
            let guard = node.lock();
            if guard.role == RaftRole::Leader {
                return Some(guard.id.clone());
            }
            if let Some(leader) = guard.leader_id.clone() {
                if nodes.contains_key(leader.as_str()) {
                    return Some(leader);
                }
            }
        }
        None
    }

    /// Returns the role currently held by `node_id`.
    pub fn role(&self, node_id: &str) -> Option<RaftRole> {
        self.inner
            .nodes
            .read()
            .get(node_id)
            .map(|node| node.lock().role)
    }

    /// Returns the current term observed by `node_id`.
    pub fn current_term(&self, node_id: &str) -> Option<u64> {
        self.inner
            .nodes
            .read()
            .get(node_id)
            .map(|node| node.lock().hard_state.current_term)
    }

    /// Returns the leader's replicated state, or — if no leader exists — the
    /// state stored on an arbitrary node.
    pub fn state(&self) -> Option<ReplicatedState> {
        let nodes = self.inner.nodes.read();
        let mut leader = None;
        for node in nodes.values() {
            let guard = node.lock();
            if guard.role == RaftRole::Leader {
                leader = Some(guard.state_machine.clone());
                break;
            }
        }
        leader.or_else(|| {
            nodes
                .values()
                .next()
                .map(|node| node.lock().state_machine.clone())
        })
    }

    /// Returns the replicated state stored on a specific node.
    pub fn node_state(&self, node_id: &str) -> Option<ReplicatedState> {
        self.inner
            .nodes
            .read()
            .get(node_id)
            .map(|node| node.lock().state_machine.clone())
    }

    /// Forces a new election initiated by `candidate_id`.
    ///
    /// In an in-process cluster the election runs to completion before this
    /// method returns; the returned value is the elected leader's term.
    /// When a custom transport is in use the method returns the candidate's
    /// new term immediately and the caller is responsible for dispatching
    /// the produced messages.
    pub fn campaign(&self, candidate_id: &str) -> Result<u64> {
        let now = Instant::now();
        let outbox = self.with_node_mut(candidate_id, |node| node.start_election(now))?;
        self.dispatch(candidate_id, outbox)?;
        self.drain()?;
        let term = self.with_node_mut(candidate_id, |node| {
            Ok::<_, CorsaError>(node.hard_state.current_term)
        })?;
        if self.inner.in_process.is_some() {
            // Verify the candidate actually won the election in-process.
            let role = self.with_node_mut(candidate_id, |node| Ok::<_, CorsaError>(node.role))?;
            if role != RaftRole::Leader {
                return Err(CorsaError::Protocol(CompactString::from(
                    "raft election did not reach quorum",
                )));
            }
        }
        Ok(term)
    }

    /// Appends a command through the current leader, blocking (for the
    /// in-process transport) until the entry is committed.
    pub fn append(&self, leader_id: &str, command: ReplicatedCommand) -> Result<u64> {
        let now = Instant::now();
        let (index, outbox) = self.with_node_mut(leader_id, |node| node.propose(command, now))?;
        self.dispatch(leader_id, outbox)?;
        self.drain()?;
        if self.inner.in_process.is_some() {
            // Ensure the entry has been committed on the leader. With the
            // synchronous in-process transport this is true once `drain`
            // returns.
            let committed =
                self.with_node_mut(leader_id, |node| Ok::<_, CorsaError>(node.commit_index))?;
            if committed < index {
                return Err(CorsaError::Protocol(CompactString::from(
                    "raft append did not reach quorum",
                )));
            }
        }
        Ok(index)
    }

    /// Delivers a single message to the targeted node.
    ///
    /// External (network-backed) transports call this for every message they
    /// receive. The cluster takes care of producing and dispatching any
    /// response messages.
    pub fn step(&self, target: &str, message: RaftMessage) -> Result<()> {
        let now = Instant::now();
        let outbox = self.with_node_mut(target, |node| node.step(message, now))?;
        self.dispatch(target, outbox)?;
        self.drain()
    }

    /// Advances election and heartbeat timers on every node.
    ///
    /// Real deployments call this on a periodic driver thread; tests can
    /// invoke it directly to simulate the passage of time.
    pub fn tick(&self) -> Result<()> {
        let now = Instant::now();
        let nodes = self
            .inner
            .nodes
            .read()
            .keys()
            .cloned()
            .collect::<SmallVec<[CompactString; 4]>>();
        for node_id in nodes {
            let outbox = self.with_node_mut(node_id.as_str(), |node| node.tick(now))?;
            self.dispatch(node_id.as_str(), outbox)?;
        }
        self.drain()
    }

    /// Forces a state machine snapshot on the given node.
    ///
    /// Useful for clusters that prefer to drive snapshotting from an external
    /// scheduler rather than relying on
    /// [`RaftConfig::snapshot_threshold`].
    pub fn compact(&self, node_id: &str) -> Result<RaftSnapshot> {
        self.with_node_mut(node_id, |node| node.snapshot())
    }

    /// Adds a new member to the cluster.
    ///
    /// The current implementation uses a simplified single-server membership
    /// change: the new member starts as a follower with empty log and the
    /// existing leader (if any) replicates state to it on the next heartbeat.
    /// Real joint-consensus is intentionally out of scope here; for safe
    /// multi-member changes apply them one at a time.
    pub fn add_member(&self, node_id: impl Into<CompactString>) -> Result<()> {
        let id = node_id.into();
        let now = Instant::now();
        let mut nodes = self.inner.nodes.write();
        if nodes.contains_key(id.as_str()) {
            return Ok(());
        }
        if let Some(transport) = self.inner.in_process.as_ref() {
            transport.register(id.as_str());
        }
        let mut members: FastSet<CompactString> = nodes.keys().cloned().collect();
        members.insert(id.clone());
        let new_node = RaftNode::restore(
            id.clone(),
            members.clone(),
            self.inner.config.clone(),
            self.inner.storage.clone(),
            now,
            self.inner
                .config
                .election_jitter_seed
                .wrapping_add(nodes.len() as u64),
        )?;
        nodes.insert(id.clone(), Arc::new(Mutex::new(new_node)));
        // Propagate the new membership view to every existing node and
        // collect any leader so we can immediately replicate state to the
        // freshly added member.
        let mut leader_id: Option<CompactString> = None;
        for (node_id, node_handle) in nodes.iter() {
            let mut node = node_handle.lock();
            node.members = members.clone();
            if node.role == RaftRole::Leader {
                let next = node.last_log_index() + 1;
                node.next_index.insert(id.clone(), next);
                node.match_index.insert(id.clone(), 0);
                leader_id = Some(node_id.clone());
            }
        }
        drop(nodes);
        if let Some(leader) = leader_id {
            let outbox =
                self.with_node_mut(leader.as_str(), |node| node.build_append_entries(now))?;
            self.dispatch(leader.as_str(), outbox)?;
            self.drain()?;
        }
        Ok(())
    }

    /// Removes a member from the cluster.
    pub fn remove_member(&self, node_id: &str) -> Result<()> {
        let mut nodes = self.inner.nodes.write();
        if nodes.remove(node_id).is_none() {
            return Err(CorsaError::Protocol(compact_format(format_args!(
                "unknown raft node: {node_id}"
            ))));
        }
        if let Some(transport) = self.inner.in_process.as_ref() {
            transport.unregister(node_id);
        }
        let members: FastSet<CompactString> = nodes.keys().cloned().collect();
        for node_handle in nodes.values() {
            let mut node = node_handle.lock();
            node.members = members.clone();
            node.next_index.remove(node_id);
            node.match_index.remove(node_id);
        }
        Ok(())
    }

    fn with_node_mut<R>(
        &self,
        node_id: &str,
        with: impl FnOnce(&mut RaftNode) -> Result<R>,
    ) -> Result<R> {
        let nodes = self.inner.nodes.read();
        let node = nodes
            .get(node_id)
            .ok_or_else(|| {
                CorsaError::Protocol(compact_format(format_args!("unknown raft node: {node_id}")))
            })?
            .clone();
        drop(nodes);
        let mut guard = node.lock();
        with(&mut guard)
    }

    fn dispatch(&self, from: &str, outbox: Outbox) -> Result<()> {
        for (to, message) in outbox {
            self.inner.transport.send(from, to.as_str(), message)?;
        }
        Ok(())
    }

    fn drain(&self) -> Result<()> {
        let Some(transport) = self.inner.in_process.clone() else {
            return Ok(());
        };
        let max = self.inner.config.max_pending_messages.max(1);
        let mut processed = 0usize;
        loop {
            let targets = transport.nodes_with_pending();
            if targets.is_empty() {
                return Ok(());
            }
            for target in targets {
                while let Some((_from, message)) = transport.next_for(target.as_str()) {
                    processed += 1;
                    if processed > max {
                        return Err(CorsaError::Protocol(CompactString::from(
                            "raft cluster drain exceeded max_pending_messages",
                        )));
                    }
                    if !self.inner.nodes.read().contains_key(target.as_str()) {
                        // Member was removed mid-drain; drop further messages.
                        continue;
                    }
                    let now = Instant::now();
                    let outbox =
                        self.with_node_mut(target.as_str(), |node| node.step(message, now))?;
                    self.dispatch(target.as_str(), outbox)?;
                }
            }
        }
    }
}
