//! Orchestrator implementations and replicated-state data models.
//!
//! This module always includes the local worker pool.
//! The distributed orchestration layer is compiled only when the
//! `experimental-distributed` cargo feature is enabled.

mod api;
#[cfg(feature = "experimental-distributed")]
mod distributed;
mod panic_payload;
#[cfg(feature = "experimental-distributed")]
mod raft;
#[cfg(feature = "experimental-distributed")]
mod state;

/// Local worker-pool orchestrator with snapshot and result caches.
pub use api::{ApiOrchestrator, ApiOrchestratorConfig, ApiOrchestratorStats};
/// Distributed wrapper that replicates overlay and cache state.
#[cfg(feature = "experimental-distributed")]
pub use distributed::DistributedApiOrchestrator;
/// Raft topology and protocol exports used by the experimental distributed
/// orchestrator.
#[cfg(feature = "experimental-distributed")]
pub use raft::{
    ChannelTransport, FileStorage, HardState, InMemoryStorage, InProcessTransport,
    PersistedLogEntry, RaftCluster, RaftClusterBuilder, RaftConfig, RaftMessage, RaftRole,
    RaftSnapshot, RaftStorage, RaftTransport,
};
/// Serializable state mirrored across the distributed orchestrator cluster.
#[cfg(feature = "experimental-distributed")]
pub use state::{ReplicatedCacheEntry, ReplicatedCommand, ReplicatedSnapshot, ReplicatedState};
