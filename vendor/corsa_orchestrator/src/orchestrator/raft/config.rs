//! Tunables that govern Raft timing, log compaction, and batching.
//!
//! The defaults here are chosen to be useful in both unit tests (where the
//! in-process transport drains messages synchronously and timers never fire)
//! and in real deployments backed by a network transport with a periodic
//! [`crate::orchestrator::raft::RaftCluster::tick`] driver.

use std::time::Duration;

/// Configuration for one [`crate::orchestrator::raft::RaftCluster`] instance.
///
/// All durations are interpreted in terms of monotonic time produced by
/// [`std::time::Instant::now`]. The cluster's `tick` method consults these
/// values when deciding whether to time out elections or fire heartbeats.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RaftConfig {
    /// Minimum amount of time a follower waits without contact from the
    /// leader before starting a new election. Real elections jitter between
    /// `election_timeout_min` and `election_timeout_max` to avoid split votes.
    pub election_timeout_min: Duration,
    /// Maximum amount of time a follower waits without contact from the
    /// leader before starting a new election.
    pub election_timeout_max: Duration,
    /// Interval between leader heartbeats.
    ///
    /// Must be strictly less than [`Self::election_timeout_min`] so a healthy
    /// leader never lets followers run their own election timeouts.
    pub heartbeat_interval: Duration,
    /// Maximum number of log entries shipped in a single `AppendEntries`
    /// message. Keeping this bounded keeps individual messages small enough
    /// for real transports.
    pub max_entries_per_append: usize,
    /// Number of committed-and-applied log entries to retain before the
    /// state machine is allowed to compact the log into a snapshot.
    ///
    /// Set to `usize::MAX` to disable automatic snapshotting. The Raft
    /// driver also exposes a manual snapshot trigger for advanced callers.
    pub snapshot_threshold: usize,
    /// Maximum number of pending messages a single in-process drain pump
    /// will deliver before aborting with a protocol error. This is a safety
    /// net against pathological loops.
    pub max_pending_messages: usize,
    /// Deterministic seed used by the in-process election timer jitter.
    ///
    /// Production network transports normally drive elections from real
    /// monotonic time and do not need a seed; the in-process synchronous
    /// driver uses this so tests stay reproducible.
    pub election_jitter_seed: u64,
}

impl RaftConfig {
    /// Defaults tuned for in-process tests and integration: short timeouts
    /// so a real `tick` loop converges quickly, deterministic jitter.
    pub const DEFAULT: Self = Self {
        election_timeout_min: Duration::from_millis(150),
        election_timeout_max: Duration::from_millis(300),
        heartbeat_interval: Duration::from_millis(50),
        max_entries_per_append: 64,
        snapshot_threshold: 1024,
        max_pending_messages: 65_536,
        election_jitter_seed: 0,
    };

    /// Returns the configured election-timeout range as a `(min, max)` pair.
    pub fn election_timeout_range(&self) -> (Duration, Duration) {
        let (min, max) = if self.election_timeout_min <= self.election_timeout_max {
            (self.election_timeout_min, self.election_timeout_max)
        } else {
            (self.election_timeout_max, self.election_timeout_min)
        };
        (min, max)
    }

    /// Returns whether the configuration satisfies Raft's mandatory ordering
    /// `heartbeat < election_timeout_min`.
    pub fn is_valid(&self) -> bool {
        let (min, max) = self.election_timeout_range();
        self.heartbeat_interval < min && min <= max
    }
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}
