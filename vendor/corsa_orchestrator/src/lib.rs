//! Orchestration layers for coordinating one or more Corsa workers.
//!
//! The orchestration crates are where `corsa` can outperform naive CLI usage:
//! by prewarming workers, reusing snapshots, memoizing results, and replicating
//! editor state, higher-level workflows avoid paying full initialization cost
//! for every query.
//!
//! # Entry Points
//!
//! - [`ApiOrchestrator`] manages a local pool of API workers plus caches.
//! - Distributed replication is gated behind the `experimental-distributed`
//!   cargo feature and, like the higher-level
//!   [`orchestrator::DistributedApiOrchestrator`] surface, sits outside the
//!   production support commitment (see `docs/support_policy.md`). The
//!   underlying Raft implementation, exposed as
//!   [`orchestrator::RaftCluster`], carries pluggable
//!   [`orchestrator::RaftStorage`] and [`orchestrator::RaftTransport`]
//!   traits, implements log replication with the conflict-index backfill
//!   optimisation, randomized election timing, snapshot-based log
//!   compaction, and a simplified single-server membership change protocol.

/// Re-exports the typed stdio API client layer used by the orchestrators.
pub mod api {
    pub use corsa_client::*;
}

/// Re-exports the LSP overlay types used for replicated virtual documents.
pub mod lsp {
    pub use corsa_lsp::*;
}

/// Re-exports structured operational events used by the orchestrator configs.
pub mod observability {
    pub use corsa_core::{CorsaEvent, CorsaObserver, SharedObserver};
}

pub use corsa_core::{CorsaError, CorsaEvent, CorsaObserver, Result, SharedObserver};

#[path = "orchestrator/mod.rs"]
/// Local and distributed orchestration helpers.
pub mod orchestrator;

pub use orchestrator::*;
