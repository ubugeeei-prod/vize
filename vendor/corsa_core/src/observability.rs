use crate::fast::CompactString;
use std::{sync::Arc, time::Duration};

/// Structured runtime events that embedders can observe.
///
/// The enum intentionally focuses on operationally relevant state transitions
/// such as timeouts, queue saturation, and cache eviction. Variants may be
/// added over time as the workspace exposes more subsystems.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CorsaEvent {
    /// A JSON-RPC request exceeded its configured timeout.
    JsonRpcRequestTimedOut {
        /// JSON-RPC method name that timed out.
        method: CompactString,
        /// Configured request timeout that was exceeded.
        timeout: Duration,
    },
    /// The JSON-RPC writer queue rejected a message because it was full.
    JsonRpcOutboundQueueFull,
    /// A locally registered JSON-RPC handler panicked while processing a peer message.
    JsonRpcLocalHandlerPanicked {
        /// JSON-RPC method name whose local handler panicked.
        method: CompactString,
    },
    /// Pending JSON-RPC requests were failed because the transport broke.
    JsonRpcPendingRequestsFailed {
        /// Transport error propagated to pending callers.
        error: CompactString,
        /// Number of pending requests failed together.
        count: usize,
    },
    /// A sync msgpack request exceeded its configured timeout.
    MsgpackRequestTimedOut {
        /// Msgpack API method name that timed out.
        method: CompactString,
        /// Configured request timeout that was exceeded.
        timeout: Duration,
    },
    /// The msgpack worker queue rejected a request because it was full.
    MsgpackWorkerQueueFull {
        /// Msgpack API method that could not be queued.
        method: CompactString,
    },
    /// The msgpack worker process was explicitly terminated.
    MsgpackWorkerTerminated {
        /// Human-readable termination reason.
        reason: CompactString,
    },
    /// A cached snapshot was evicted to stay within configured limits.
    OrchestratorSnapshotEvicted {
        /// Caller-provided snapshot cache key that was evicted.
        key: CompactString,
    },
    /// A cached result was evicted to stay within configured limits.
    OrchestratorResultEvicted {
        /// Caller-provided result cache key that was evicted.
        key: CompactString,
    },
}

/// Sink for structured operational events emitted by the workspace.
pub trait CorsaObserver: Send + Sync + 'static {
    /// Receives a single event.
    fn on_event(&self, event: &CorsaEvent);
}

/// Shared observer handle used across configs and transports.
pub type SharedObserver = Arc<dyn CorsaObserver>;

/// Emits an event when an observer is configured.
pub fn observe(observer: Option<&SharedObserver>, event: CorsaEvent) {
    if let Some(observer) = observer {
        observer.on_event(&event);
    }
}
