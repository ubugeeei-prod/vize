//! Per-node Raft state machine.
//!
//! This module encapsulates the canonical Raft algorithm described in the
//! original "In Search of an Understandable Consensus Algorithm" paper
//! (Ongaro & Ousterhout, 2014). Every node owns:
//!
//! - persistent state (term, vote, log) backed by [`RaftStorage`]
//! - volatile state (role, `commit_index`, `last_applied`)
//! - leader-only volatile state (`next_index`, `match_index` per peer)
//! - the deterministic replicated state machine [`ReplicatedState`]
//!
//! `RaftNode::step` is the single entry point for incoming messages and
//! produces the messages the caller must dispatch through the transport.
//! `RaftNode::tick` advances internal timers and produces heartbeat or
//! election-start messages when those timers fire.

use super::{
    RaftRole,
    config::RaftConfig,
    messages::{PersistedLogEntry, RaftMessage, RaftSnapshot},
    storage::{HardState, RaftStorage},
};
use crate::{
    CorsaError, Result,
    orchestrator::state::{ReplicatedCommand, ReplicatedState},
};
use corsa_core::fast::{CompactString, FastMap, FastSet, SmallVec, compact_format};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Messages a state-machine step asks the driver to dispatch.
pub type Outbox = SmallVec<[(CompactString, RaftMessage); 4]>;

/// Per-node Raft state machine.
pub(super) struct RaftNode {
    pub(super) id: CompactString,
    pub(super) role: RaftRole,
    pub(super) hard_state: HardState,
    pub(super) commit_index: u64,
    pub(super) last_applied: u64,
    pub(super) log_base_index: u64,
    pub(super) log_base_term: u64,
    pub(super) log: SmallVec<[PersistedLogEntry; 32]>,
    pub(super) leader_id: Option<CompactString>,
    pub(super) members: FastSet<CompactString>,
    pub(super) state_machine: ReplicatedState,
    pub(super) next_index: FastMap<CompactString, u64>,
    pub(super) match_index: FastMap<CompactString, u64>,
    pub(super) votes_granted: FastSet<CompactString>,
    pub(super) votes_rejected: FastSet<CompactString>,
    pub(super) election_deadline: Instant,
    pub(super) heartbeat_deadline: Instant,
    pub(super) config: RaftConfig,
    pub(super) storage: Arc<dyn RaftStorage>,
    pub(super) jitter_state: u64,
}

impl RaftNode {
    pub(super) fn restore(
        id: CompactString,
        members: FastSet<CompactString>,
        config: RaftConfig,
        storage: Arc<dyn RaftStorage>,
        now: Instant,
        jitter_seed: u64,
    ) -> Result<Self> {
        let hard_state = storage.load_hard_state(id.as_str())?;
        let log = storage.load_log(id.as_str())?;
        let log_base_index = storage.log_base_index(id.as_str())?;
        let log_base_term = storage.log_base_term(id.as_str())?;
        let snapshot = storage.load_snapshot(id.as_str())?;
        let mut state_machine = ReplicatedState::default();
        if let Some(snapshot) = snapshot.as_ref() {
            if !snapshot.data.is_empty() {
                state_machine = serde_json::from_slice(&snapshot.data)?;
            }
        }
        let mut node = Self {
            id,
            role: RaftRole::Follower,
            hard_state,
            commit_index: log_base_index,
            last_applied: log_base_index,
            log_base_index,
            log_base_term,
            log,
            leader_id: None,
            members,
            state_machine,
            next_index: FastMap::default(),
            match_index: FastMap::default(),
            votes_granted: FastSet::default(),
            votes_rejected: FastSet::default(),
            election_deadline: now,
            heartbeat_deadline: now,
            config,
            storage,
            jitter_state: jitter_seed,
        };
        node.reset_election_deadline(now);
        Ok(node)
    }

    pub(super) fn last_log_index(&self) -> u64 {
        self.log_base_index + self.log.len() as u64
    }

    pub(super) fn last_log_term(&self) -> u64 {
        self.log
            .last()
            .map(|entry| entry.term)
            .unwrap_or(self.log_base_term)
    }

    fn term_at_index(&self, index: u64) -> Option<u64> {
        if index == 0 {
            return Some(0);
        }
        if index == self.log_base_index {
            return Some(self.log_base_term);
        }
        if index < self.log_base_index || index > self.last_log_index() {
            return None;
        }
        let offset = (index - self.log_base_index - 1) as usize;
        self.log.get(offset).map(|entry| entry.term)
    }

    fn entries_after(&self, prev_index: u64) -> SmallVec<[PersistedLogEntry; 4]> {
        let start = (prev_index + 1).max(self.log_base_index + 1);
        if start > self.last_log_index() {
            return SmallVec::new();
        }
        let local_start = (start - self.log_base_index - 1) as usize;
        let local_end = (local_start + self.config.max_entries_per_append).min(self.log.len());
        self.log[local_start..local_end].iter().cloned().collect()
    }

    fn persist_hard_state(
        &mut self,
        new_term: u64,
        voted_for: Option<CompactString>,
    ) -> Result<()> {
        let hard_state = HardState {
            current_term: new_term,
            voted_for,
        };
        self.storage
            .save_hard_state(self.id.as_str(), &hard_state)?;
        self.hard_state = hard_state;
        Ok(())
    }

    fn become_follower(
        &mut self,
        new_term: u64,
        leader_id: Option<CompactString>,
        now: Instant,
    ) -> Result<()> {
        let term_changed = new_term != self.hard_state.current_term;
        if term_changed {
            self.persist_hard_state(new_term, None)?;
        } else if self.role != RaftRole::Follower {
            // Keep the persisted voted_for stable when only role changes.
        }
        self.role = RaftRole::Follower;
        self.leader_id = leader_id;
        self.votes_granted.clear();
        self.votes_rejected.clear();
        self.reset_election_deadline(now);
        Ok(())
    }

    fn become_candidate(&mut self, now: Instant) -> Result<()> {
        let new_term = self.hard_state.current_term + 1;
        self.persist_hard_state(new_term, Some(self.id.clone()))?;
        self.role = RaftRole::Candidate;
        self.leader_id = None;
        self.votes_granted.clear();
        self.votes_rejected.clear();
        self.votes_granted.insert(self.id.clone());
        self.reset_election_deadline(now);
        Ok(())
    }

    fn become_leader(&mut self, now: Instant) {
        self.role = RaftRole::Leader;
        self.leader_id = Some(self.id.clone());
        let next = self.last_log_index() + 1;
        self.next_index.clear();
        self.match_index.clear();
        let self_id = self.id.clone();
        let last_log_index = self.last_log_index();
        for member in &self.members {
            if *member == self_id {
                self.next_index.insert(member.clone(), next);
                self.match_index.insert(member.clone(), last_log_index);
            } else {
                self.next_index.insert(member.clone(), next);
                self.match_index.insert(member.clone(), 0);
            }
        }
        self.heartbeat_deadline = now;
    }

    fn reset_election_deadline(&mut self, now: Instant) {
        let (min, max) = self.config.election_timeout_range();
        let jitter = self.election_jitter(min, max);
        self.election_deadline = now + jitter;
    }

    fn election_jitter(&mut self, min: Duration, max: Duration) -> Duration {
        // Deterministic SplitMix64 stream so tests are reproducible.
        let mut state = self.jitter_state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        self.jitter_state = state;
        let _ = &mut state;
        let span = max.saturating_sub(min);
        if span.is_zero() {
            return min;
        }
        let nanos = z % (span.as_nanos() as u64 + 1);
        min + Duration::from_nanos(nanos)
    }

    pub(super) fn start_election(&mut self, now: Instant) -> Result<Outbox> {
        self.become_candidate(now)?;
        let mut outbox = Outbox::new();
        let last_log_index = self.last_log_index();
        let last_log_term = self.last_log_term();
        let term = self.hard_state.current_term;
        for member in self.members.iter().filter(|m| **m != self.id).cloned() {
            outbox.push((
                member,
                RaftMessage::RequestVote {
                    term,
                    candidate_id: self.id.clone(),
                    last_log_index,
                    last_log_term,
                },
            ));
        }
        if outbox.is_empty() {
            // Single-node cluster: immediately become leader.
            self.become_leader(now);
            self.apply_committed_up_to(self.last_log_index())?;
        }
        Ok(outbox)
    }

    pub(super) fn propose(
        &mut self,
        command: ReplicatedCommand,
        now: Instant,
    ) -> Result<(u64, Outbox)> {
        if self.role != RaftRole::Leader {
            return Err(CorsaError::Protocol(compact_format(format_args!(
                "raft node is not leader: {}",
                self.id
            ))));
        }
        let entry = PersistedLogEntry {
            term: self.hard_state.current_term,
            command,
        };
        let start_index = self.last_log_index() + 1;
        self.storage
            .append_log(self.id.as_str(), start_index, std::slice::from_ref(&entry))?;
        self.log.push(entry);
        self.match_index
            .insert(self.id.clone(), self.last_log_index());
        self.next_index
            .insert(self.id.clone(), self.last_log_index() + 1);

        let outbox = self.build_append_entries(now)?;
        // Even with no peers (single-node), we may already have quorum.
        self.advance_commit_index()?;
        self.apply_committed_up_to(self.commit_index)?;
        Ok((start_index, outbox))
    }

    pub(super) fn build_append_entries(&mut self, now: Instant) -> Result<Outbox> {
        let mut outbox = Outbox::new();
        let leader_term = self.hard_state.current_term;
        let leader_commit = self.commit_index;
        let peers = self
            .members
            .iter()
            .filter(|m| **m != self.id)
            .cloned()
            .collect::<SmallVec<[CompactString; 4]>>();
        for peer in peers {
            let next_index = self.next_index.get(peer.as_str()).copied().unwrap_or(1);
            if next_index <= self.log_base_index {
                // Follower needs an installed snapshot.
                if let Some(snapshot) = self.storage.load_snapshot(self.id.as_str())? {
                    outbox.push((
                        peer.clone(),
                        RaftMessage::InstallSnapshot {
                            term: leader_term,
                            leader_id: self.id.clone(),
                            snapshot,
                        },
                    ));
                    continue;
                }
            }
            let prev_index = next_index.saturating_sub(1);
            let prev_term = self.term_at_index(prev_index).unwrap_or(0);
            let entries = self.entries_after(prev_index);
            outbox.push((
                peer,
                RaftMessage::AppendEntries {
                    term: leader_term,
                    leader_id: self.id.clone(),
                    prev_log_index: prev_index,
                    prev_log_term: prev_term,
                    entries,
                    leader_commit,
                },
            ));
        }
        self.heartbeat_deadline = now + self.config.heartbeat_interval;
        Ok(outbox)
    }

    fn handle_request_vote(
        &mut self,
        term: u64,
        candidate_id: CompactString,
        last_log_index: u64,
        last_log_term: u64,
        now: Instant,
    ) -> Result<Outbox> {
        if term > self.hard_state.current_term {
            self.become_follower(term, None, now)?;
        }
        let mut grant = false;
        if term >= self.hard_state.current_term {
            let our_last_term = self.last_log_term();
            let our_last_index = self.last_log_index();
            let up_to_date = last_log_term > our_last_term
                || (last_log_term == our_last_term && last_log_index >= our_last_index);
            let voted_for_ok = self
                .hard_state
                .voted_for
                .as_deref()
                .map(|v| v == candidate_id.as_str())
                .unwrap_or(true);
            if up_to_date && voted_for_ok {
                self.persist_hard_state(term, Some(candidate_id.clone()))?;
                self.reset_election_deadline(now);
                grant = true;
            }
        }
        let mut outbox = Outbox::new();
        outbox.push((
            candidate_id,
            RaftMessage::RequestVoteResponse {
                term: self.hard_state.current_term,
                from: self.id.clone(),
                vote_granted: grant,
            },
        ));
        Ok(outbox)
    }

    fn handle_request_vote_response(
        &mut self,
        term: u64,
        from: CompactString,
        granted: bool,
        now: Instant,
    ) -> Result<Outbox> {
        if term > self.hard_state.current_term {
            self.become_follower(term, None, now)?;
            return Ok(Outbox::new());
        }
        if self.role != RaftRole::Candidate || term != self.hard_state.current_term {
            return Ok(Outbox::new());
        }
        if granted {
            self.votes_granted.insert(from);
        } else {
            self.votes_rejected.insert(from);
        }
        let quorum = quorum_size(self.members.len());
        if self.votes_granted.len() >= quorum {
            self.become_leader(now);
            // Send initial empty AppendEntries (heartbeat) so peers learn the
            // new leader and clear their election timers.
            return self.build_append_entries(now);
        }
        if self.votes_rejected.len() >= quorum {
            // Election lost; step back to follower and wait for the next
            // randomized election timeout.
            self.become_follower(self.hard_state.current_term, None, now)?;
        }
        Ok(Outbox::new())
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_append_entries(
        &mut self,
        term: u64,
        leader_id: CompactString,
        prev_log_index: u64,
        prev_log_term: u64,
        entries: SmallVec<[PersistedLogEntry; 4]>,
        leader_commit: u64,
        now: Instant,
    ) -> Result<Outbox> {
        let mut outbox = Outbox::new();
        if term < self.hard_state.current_term {
            outbox.push((
                leader_id,
                RaftMessage::AppendEntriesResponse {
                    term: self.hard_state.current_term,
                    from: self.id.clone(),
                    success: false,
                    match_index: 0,
                    conflict_index: 0,
                    conflict_term: 0,
                },
            ));
            return Ok(outbox);
        }
        if term > self.hard_state.current_term || self.role != RaftRole::Follower {
            self.become_follower(term, Some(leader_id.clone()), now)?;
        } else {
            self.leader_id = Some(leader_id.clone());
        }
        self.reset_election_deadline(now);

        // Log consistency check: previous-entry term must match.
        let our_prev_term = self.term_at_index(prev_log_index);
        if our_prev_term != Some(prev_log_term) {
            let (conflict_index, conflict_term) = self.conflict_hint(prev_log_index);
            outbox.push((
                leader_id,
                RaftMessage::AppendEntriesResponse {
                    term: self.hard_state.current_term,
                    from: self.id.clone(),
                    success: false,
                    match_index: 0,
                    conflict_index,
                    conflict_term,
                },
            ));
            return Ok(outbox);
        }

        // Truncate conflicting suffix, then append new entries.
        let append_start_index = prev_log_index + 1;
        if !entries.is_empty() {
            self.storage
                .truncate_log_suffix(self.id.as_str(), prev_log_index)?;
            let local_retain = (prev_log_index - self.log_base_index) as usize;
            self.log.truncate(local_retain);
            self.storage
                .append_log(self.id.as_str(), append_start_index, &entries)?;
            for entry in &entries {
                self.log.push(entry.clone());
            }
        }
        let final_match = self.last_log_index();
        let new_commit = leader_commit.min(final_match);
        if new_commit > self.commit_index {
            self.commit_index = new_commit;
            self.apply_committed_up_to(new_commit)?;
        }
        outbox.push((
            leader_id,
            RaftMessage::AppendEntriesResponse {
                term: self.hard_state.current_term,
                from: self.id.clone(),
                success: true,
                match_index: final_match,
                conflict_index: 0,
                conflict_term: 0,
            },
        ));
        Ok(outbox)
    }

    fn conflict_hint(&self, prev_log_index: u64) -> (u64, u64) {
        if prev_log_index > self.last_log_index() {
            return (self.last_log_index() + 1, 0);
        }
        match self.term_at_index(prev_log_index) {
            Some(term) => {
                // Walk back to the first index in this conflicting term.
                let mut hint = prev_log_index;
                while hint > self.log_base_index {
                    let prev = hint - 1;
                    match self.term_at_index(prev) {
                        Some(prev_term) if prev_term == term => hint = prev,
                        _ => break,
                    }
                }
                (hint, term)
            }
            None => (self.last_log_index() + 1, 0),
        }
    }

    fn handle_append_entries_response(
        &mut self,
        term: u64,
        from: CompactString,
        success: bool,
        match_index: u64,
        conflict_index: u64,
        now: Instant,
    ) -> Result<Outbox> {
        if term > self.hard_state.current_term {
            self.become_follower(term, None, now)?;
            return Ok(Outbox::new());
        }
        if self.role != RaftRole::Leader || term != self.hard_state.current_term {
            return Ok(Outbox::new());
        }
        if success {
            self.match_index.insert(from.clone(), match_index);
            self.next_index.insert(from.clone(), match_index + 1);
            let prev_commit = self.commit_index;
            self.advance_commit_index()?;
            self.apply_committed_up_to(self.commit_index)?;
            if self.commit_index > prev_commit {
                // Inform followers about the new commit point right away so
                // they apply the entries instead of waiting for the next
                // heartbeat. AppendEntries with no new entries acts as a
                // commit notification carrying `leader_commit`.
                return self.build_append_entries(now);
            }
            return Ok(Outbox::new());
        }
        // Backfill: walk next_index back using the follower's hint.
        let current = self.next_index.get(from.as_str()).copied().unwrap_or(1);
        let next = if conflict_index == 0 {
            current.saturating_sub(1).max(1)
        } else {
            conflict_index.max(1)
        };
        self.next_index.insert(from.clone(), next);
        // Immediately retry by sending another AppendEntries (or snapshot).
        let mut outbox = Outbox::new();
        let prev_index = next.saturating_sub(1);
        if next <= self.log_base_index {
            if let Some(snapshot) = self.storage.load_snapshot(self.id.as_str())? {
                outbox.push((
                    from,
                    RaftMessage::InstallSnapshot {
                        term: self.hard_state.current_term,
                        leader_id: self.id.clone(),
                        snapshot,
                    },
                ));
                return Ok(outbox);
            }
        }
        let prev_term = self.term_at_index(prev_index).unwrap_or(0);
        let entries = self.entries_after(prev_index);
        outbox.push((
            from,
            RaftMessage::AppendEntries {
                term: self.hard_state.current_term,
                leader_id: self.id.clone(),
                prev_log_index: prev_index,
                prev_log_term: prev_term,
                entries,
                leader_commit: self.commit_index,
            },
        ));
        Ok(outbox)
    }

    fn handle_install_snapshot(
        &mut self,
        term: u64,
        leader_id: CompactString,
        snapshot: RaftSnapshot,
        now: Instant,
    ) -> Result<Outbox> {
        if term < self.hard_state.current_term {
            let mut outbox = Outbox::new();
            outbox.push((
                leader_id,
                RaftMessage::InstallSnapshotResponse {
                    term: self.hard_state.current_term,
                    from: self.id.clone(),
                },
            ));
            return Ok(outbox);
        }
        if term > self.hard_state.current_term || self.role != RaftRole::Follower {
            self.become_follower(term, Some(leader_id.clone()), now)?;
        } else {
            self.leader_id = Some(leader_id.clone());
        }
        self.reset_election_deadline(now);

        if !snapshot.data.is_empty() {
            self.state_machine = serde_json::from_slice(&snapshot.data)?;
        } else {
            self.state_machine = ReplicatedState::default();
        }
        self.storage.save_snapshot(self.id.as_str(), &snapshot)?;
        self.log_base_index = snapshot.last_included_index;
        self.log_base_term = snapshot.last_included_term;
        self.storage
            .compact_log_prefix(self.id.as_str(), snapshot.last_included_index)?;
        // If our log already contained the same prefix, retain the suffix;
        // otherwise discard it because it cannot have been committed.
        let last_local = self.last_log_index();
        if last_local <= snapshot.last_included_index {
            self.log.clear();
        } else {
            self.log.clear();
            let entries = self.storage.load_log(self.id.as_str())?;
            for entry in entries {
                self.log.push(entry);
            }
        }
        self.commit_index = self.commit_index.max(snapshot.last_included_index);
        self.last_applied = self.last_applied.max(snapshot.last_included_index);

        let mut outbox = Outbox::new();
        outbox.push((
            leader_id,
            RaftMessage::InstallSnapshotResponse {
                term: self.hard_state.current_term,
                from: self.id.clone(),
            },
        ));
        Ok(outbox)
    }

    fn handle_install_snapshot_response(
        &mut self,
        term: u64,
        from: CompactString,
        now: Instant,
    ) -> Result<Outbox> {
        if term > self.hard_state.current_term {
            self.become_follower(term, None, now)?;
            return Ok(Outbox::new());
        }
        if self.role != RaftRole::Leader {
            return Ok(Outbox::new());
        }
        // After a snapshot install, the follower is caught up to log_base_index.
        self.match_index.insert(from.clone(), self.log_base_index);
        self.next_index
            .insert(from.clone(), self.log_base_index + 1);
        self.build_append_entries(now)
    }

    fn advance_commit_index(&mut self) -> Result<()> {
        if self.role != RaftRole::Leader {
            return Ok(());
        }
        let current_term = self.hard_state.current_term;
        let self_id = self.id.clone();
        let last_log_index = self.last_log_index();
        let mut indices: SmallVec<[u64; 8]> = self
            .members
            .iter()
            .map(|member| {
                if *member == self_id {
                    last_log_index
                } else {
                    self.match_index.get(member.as_str()).copied().unwrap_or(0)
                }
            })
            .collect();
        indices.sort_unstable();
        let quorum = quorum_size(self.members.len());
        // The (n - quorum)th smallest index (0-based) is the largest index
        // replicated on a majority.
        let candidate = indices[indices.len() - quorum];
        if candidate <= self.commit_index {
            return Ok(());
        }
        // §5.4.2: Leaders may only mark entries from their *own* term as
        // committed via majority replication; older-term entries become
        // committed transitively once any same-term entry commits.
        if self.term_at_index(candidate) == Some(current_term) {
            self.commit_index = candidate;
        }
        Ok(())
    }

    fn apply_committed_up_to(&mut self, target: u64) -> Result<()> {
        while self.last_applied < target {
            let next = self.last_applied + 1;
            let offset = (next - self.log_base_index - 1) as usize;
            let Some(entry) = self.log.get(offset).cloned() else {
                break;
            };
            self.state_machine.apply(&entry.command)?;
            self.last_applied = next;
        }
        self.maybe_compact_log()
    }

    pub(super) fn step(&mut self, message: RaftMessage, now: Instant) -> Result<Outbox> {
        match message {
            RaftMessage::RequestVote {
                term,
                candidate_id,
                last_log_index,
                last_log_term,
            } => self.handle_request_vote(term, candidate_id, last_log_index, last_log_term, now),
            RaftMessage::RequestVoteResponse {
                term,
                from,
                vote_granted,
            } => self.handle_request_vote_response(term, from, vote_granted, now),
            RaftMessage::AppendEntries {
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            } => self.handle_append_entries(
                term,
                leader_id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
                now,
            ),
            RaftMessage::AppendEntriesResponse {
                term,
                from,
                success,
                match_index,
                conflict_index,
                conflict_term: _,
            } => self.handle_append_entries_response(
                term,
                from,
                success,
                match_index,
                conflict_index,
                now,
            ),
            RaftMessage::InstallSnapshot {
                term,
                leader_id,
                snapshot,
            } => self.handle_install_snapshot(term, leader_id, snapshot, now),
            RaftMessage::InstallSnapshotResponse { term, from } => {
                self.handle_install_snapshot_response(term, from, now)
            }
        }
    }

    pub(super) fn tick(&mut self, now: Instant) -> Result<Outbox> {
        let mut outbox = Outbox::new();
        match self.role {
            RaftRole::Follower | RaftRole::Candidate => {
                if now >= self.election_deadline {
                    outbox.extend(self.start_election(now)?);
                }
            }
            RaftRole::Leader => {
                if now >= self.heartbeat_deadline {
                    outbox.extend(self.build_append_entries(now)?);
                }
            }
        }
        Ok(outbox)
    }

    pub(super) fn maybe_compact_log(&mut self) -> Result<()> {
        let threshold = self.config.snapshot_threshold;
        if threshold == usize::MAX {
            return Ok(());
        }
        if self.last_applied <= self.log_base_index {
            return Ok(());
        }
        let applied_in_log = (self.last_applied - self.log_base_index) as usize;
        if applied_in_log < threshold {
            return Ok(());
        }
        let last_included_index = self.last_applied;
        let last_included_term = match self.term_at_index(last_included_index) {
            Some(term) => term,
            None => return Ok(()),
        };
        let data = serde_json::to_vec(&self.state_machine)?;
        let snapshot = RaftSnapshot {
            last_included_index,
            last_included_term,
            data: data.into_iter().collect(),
        };
        self.storage.save_snapshot(self.id.as_str(), &snapshot)?;
        self.storage
            .compact_log_prefix(self.id.as_str(), last_included_index)?;
        let drop = (last_included_index - self.log_base_index) as usize;
        let drop = drop.min(self.log.len());
        let _: SmallVec<[PersistedLogEntry; 32]> = self.log.drain(..drop).collect::<SmallVec<_>>();
        self.log_base_index = last_included_index;
        self.log_base_term = last_included_term;
        Ok(())
    }

    pub(super) fn snapshot(&mut self) -> Result<RaftSnapshot> {
        let last_included_index = self.last_applied;
        let last_included_term = self
            .term_at_index(last_included_index)
            .unwrap_or(self.log_base_term);
        let data = serde_json::to_vec(&self.state_machine)?;
        let snapshot = RaftSnapshot {
            last_included_index,
            last_included_term,
            data: data.into_iter().collect(),
        };
        self.storage.save_snapshot(self.id.as_str(), &snapshot)?;
        Ok(snapshot)
    }
}

/// Returns the majority threshold for `participants` members.
pub(super) fn quorum_size(participants: usize) -> usize {
    (participants / 2) + 1
}
