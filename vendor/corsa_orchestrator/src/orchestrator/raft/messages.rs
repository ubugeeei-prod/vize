//! Raft RPC and replicated-log entry types exchanged between nodes.
//!
//! These types intentionally mirror the wire shapes described in the Raft
//! paper (§5.1 leader election, §5.3 log replication, §7 log compaction) so a
//! production transport can serialize them directly with `serde`.

use super::super::state::ReplicatedCommand;
use corsa_core::fast::{CompactString, SmallVec};
use serde::{Deserialize, Serialize};

/// Term-tagged log entry persisted on every node.
///
/// The `term` records the leader term that first appended the entry. Raft's
/// log-matching property uses `term` together with the entry's 1-based index
/// to detect conflicting log positions during replication.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedLogEntry {
    /// Leader term in which this entry was originally appended.
    pub term: u64,
    /// Replicated state machine command carried by the entry.
    pub command: ReplicatedCommand,
}

/// Snapshot of the replicated state machine used for log compaction.
///
/// A snapshot summarizes the prefix `[1, last_included_index]` of the Raft
/// log. Followers behind that prefix install the snapshot wholesale instead
/// of replaying individual entries.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RaftSnapshot {
    /// 1-based log index of the last entry the snapshot includes.
    pub last_included_index: u64,
    /// Term of the last entry the snapshot includes.
    pub last_included_term: u64,
    /// Serialized state machine payload (msgpack-friendly bytes).
    pub data: SmallVec<[u8; 256]>,
}

/// All Raft RPCs and responses routed between cluster peers.
///
/// The variants are tagged so the type is forward compatible with future
/// additions (linearizable reads, learner promotion, joint configuration).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum RaftMessage {
    /// §5.2: Candidates ask followers for votes.
    RequestVote {
        /// Election term the candidate is competing for.
        term: u64,
        /// Identifier of the candidate requesting the vote.
        candidate_id: CompactString,
        /// 1-based index of the candidate's last log entry.
        last_log_index: u64,
        /// Term of the candidate's last log entry.
        last_log_term: u64,
    },
    /// §5.2: Reply from a follower or candidate to a `RequestVote`.
    RequestVoteResponse {
        /// Responder's current term so the candidate can step down if stale.
        term: u64,
        /// Identifier of the responder.
        from: CompactString,
        /// Whether the responder granted its vote.
        vote_granted: bool,
    },
    /// §5.3: Leader replicates entries (or sends an empty heartbeat).
    AppendEntries {
        /// Leader's current term.
        term: u64,
        /// Identifier of the leader sending the request.
        leader_id: CompactString,
        /// 1-based index of the log entry immediately preceding `entries`.
        prev_log_index: u64,
        /// Term of the entry at `prev_log_index`.
        prev_log_term: u64,
        /// New entries to store (empty for heartbeats).
        entries: SmallVec<[PersistedLogEntry; 4]>,
        /// Leader's `commitIndex`, advanced as entries are persisted.
        leader_commit: u64,
    },
    /// §5.3: Reply from a follower to an `AppendEntries`.
    AppendEntriesResponse {
        /// Responder's current term.
        term: u64,
        /// Identifier of the responder.
        from: CompactString,
        /// Whether the responder accepted the entries.
        success: bool,
        /// Index of the last entry the responder matches with the leader,
        /// used by the leader to advance its `matchIndex` table.
        match_index: u64,
        /// Hint for log backfill on rejection: the follower's first index
        /// in the conflicting term. The leader uses this to avoid the
        /// one-entry-per-RTT walk-back from the original paper.
        conflict_index: u64,
        /// Term of the conflicting entry (or 0 if `conflict_index == 0`).
        conflict_term: u64,
    },
    /// §7: Leader installs a state machine snapshot on a lagging follower.
    InstallSnapshot {
        /// Leader's current term.
        term: u64,
        /// Identifier of the leader sending the snapshot.
        leader_id: CompactString,
        /// Snapshot covering log prefix up to `last_included_index`.
        snapshot: RaftSnapshot,
    },
    /// §7: Reply from a follower to an `InstallSnapshot`.
    InstallSnapshotResponse {
        /// Responder's current term.
        term: u64,
        /// Identifier of the responder.
        from: CompactString,
    },
}

impl RaftMessage {
    /// Returns the term this message carries, for term-based step-down logic.
    pub fn term(&self) -> u64 {
        match self {
            Self::RequestVote { term, .. }
            | Self::RequestVoteResponse { term, .. }
            | Self::AppendEntries { term, .. }
            | Self::AppendEntriesResponse { term, .. }
            | Self::InstallSnapshot { term, .. }
            | Self::InstallSnapshotResponse { term, .. } => *term,
        }
    }

    /// Returns whether this message is a leader-side broadcast (vs a reply).
    pub fn is_request(&self) -> bool {
        matches!(
            self,
            Self::RequestVote { .. } | Self::AppendEntries { .. } | Self::InstallSnapshot { .. }
        )
    }
}
