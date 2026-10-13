//! Pluggable transport for Raft RPC messages.
//!
//! The transport trait is intentionally minimal: a single fallible "deliver
//! this message to that peer" entry point. The Raft cluster does not assume
//! ordered delivery, exactly-once semantics, or even that the message ever
//! arrives — Raft tolerates retries, reorderings, and drops by design.
//!
//! Two built-in implementations are provided:
//!
//! - [`InProcessTransport`] for tests and single-process deployments
//! - [`ChannelTransport`] for sender-thread / receiver-thread topologies
//!
//! Production deployments wrap their preferred network library (TCP, QUIC,
//! gRPC, etc.) behind the [`RaftTransport`] trait and feed received messages
//! into [`crate::orchestrator::raft::RaftCluster::step`].

use super::messages::RaftMessage;
use crate::{CorsaError, Result};
use corsa_core::fast::{CompactString, FastMap, SmallVec, compact_format};
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

/// Transport-side API exposed to the Raft cluster.
///
/// Implementations must be thread-safe — the cluster may invoke `send` from
/// multiple worker threads concurrently.
pub trait RaftTransport: Send + Sync {
    /// Delivers `message` to `to` on behalf of `from`.
    ///
    /// The transport is free to drop, delay, or reorder messages. Raft
    /// remains correct in the presence of any of these failures.
    fn send(&self, from: &str, to: &str, message: RaftMessage) -> Result<()>;
}

/// One queued message tagged with its sender for delivery to a specific peer.
type DeliveryQueue = VecDeque<(CompactString, RaftMessage)>;

/// In-process transport that appends every send to a shared message queue.
///
/// The owning [`super::RaftCluster`] periodically drains this queue and steps
/// the addressed Raft node. This is the transport used by tests and by the
/// synchronous in-process driver.
#[derive(Clone, Default)]
pub struct InProcessTransport {
    inboxes: Arc<Mutex<FastMap<CompactString, DeliveryQueue>>>,
}

impl InProcessTransport {
    /// Creates a new in-process transport.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a node so it has an inbox even before its first message.
    pub fn register(&self, node_id: &str) {
        self.inboxes
            .lock()
            .entry(CompactString::from(node_id))
            .or_default();
    }

    /// Removes a node's inbox (called when a node is removed from the cluster).
    pub fn unregister(&self, node_id: &str) {
        self.inboxes.lock().remove(node_id);
    }

    /// Pops the next pending `(from, message)` for `node_id`, if any.
    ///
    /// Returns `None` when the inbox is empty.
    pub fn next_for(&self, node_id: &str) -> Option<(CompactString, RaftMessage)> {
        self.inboxes
            .lock()
            .get_mut(node_id)
            .and_then(|inbox| inbox.pop_front())
    }

    /// Returns the total number of queued messages across every inbox.
    pub fn pending_count(&self) -> usize {
        self.inboxes
            .lock()
            .values()
            .map(|inbox| inbox.len())
            .sum::<usize>()
    }

    /// Returns identifiers of every node that has at least one queued message.
    pub fn nodes_with_pending(&self) -> SmallVec<[CompactString; 4]> {
        self.inboxes
            .lock()
            .iter()
            .filter(|(_, queue)| !queue.is_empty())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Drops every queued message; mainly useful for tests that want to
    /// simulate a network partition without tearing down the cluster.
    pub fn clear(&self) {
        for queue in self.inboxes.lock().values_mut() {
            queue.clear();
        }
    }
}

impl RaftTransport for InProcessTransport {
    fn send(&self, from: &str, to: &str, message: RaftMessage) -> Result<()> {
        let mut inboxes = self.inboxes.lock();
        let inbox = inboxes.entry(CompactString::from(to)).or_default();
        inbox.push_back((CompactString::from(from), message));
        Ok(())
    }
}

/// Channel-backed transport for cases where Raft nodes live in their own
/// threads and exchange messages over `std::sync::mpsc` channels.
///
/// Senders look up the target channel and push the message; receivers pull
/// from their own channel and pass each message to
/// [`super::RaftCluster::step`]. This is appropriate when the cluster runs
/// inside one process but the nodes are isolated for concurrency reasons.
#[derive(Clone)]
pub struct ChannelTransport {
    inner: Arc<ChannelInner>,
}

struct ChannelInner {
    senders: Mutex<FastMap<CompactString, mpsc::Sender<(CompactString, RaftMessage)>>>,
}

impl ChannelTransport {
    /// Creates an empty channel transport.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ChannelInner {
                senders: Mutex::new(FastMap::default()),
            }),
        }
    }

    /// Registers `node_id` and returns the receiver the node should poll.
    pub fn register(&self, node_id: &str) -> mpsc::Receiver<(CompactString, RaftMessage)> {
        let (tx, rx) = mpsc::channel();
        self.inner
            .senders
            .lock()
            .insert(CompactString::from(node_id), tx);
        rx
    }

    /// Removes a node's sender; further sends to it return an error.
    pub fn unregister(&self, node_id: &str) {
        self.inner.senders.lock().remove(node_id);
    }
}

impl Default for ChannelTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl RaftTransport for ChannelTransport {
    fn send(&self, from: &str, to: &str, message: RaftMessage) -> Result<()> {
        let senders = self.inner.senders.lock();
        match senders.get(to) {
            Some(sender) => sender
                .send((CompactString::from(from), message))
                .map_err(|_| {
                    CorsaError::Protocol(compact_format(format_args!(
                        "raft channel transport: peer `{to}` is closed"
                    )))
                }),
            None => Err(CorsaError::Protocol(compact_format(format_args!(
                "raft channel transport: unknown peer `{to}`"
            )))),
        }
    }
}

/// Helper that blocks the current thread until the in-process transport's
/// pending queue empties, or a deadline expires.
///
/// Production transports usually do not need this; it exists for tests that
/// want a deterministic "wait for the cluster to settle" primitive.
#[allow(dead_code)]
pub fn wait_for_drain(transport: &InProcessTransport, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while transport.pending_count() > 0 {
        if Instant::now() > deadline {
            return Err(CorsaError::Timeout(CompactString::from(
                "raft in-process transport did not drain before deadline",
            )));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
