use corsa_core::fast::CompactString;
use std::{sync::Arc, time::Duration};

/// Fine-grained client request phase used by optional profiling hooks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiProfilePhase {
    /// Time spent serializing request parameters before transport I/O starts.
    SerializeParams,
    /// Time spent waiting on the underlying transport and upstream worker.
    Transport,
    /// Time spent decoding a typed JSON response after transport I/O completes.
    DeserializeResponse,
    /// Time spent decoding a binary payload wrapper such as base64.
    DecodeBinary,
}

impl ApiProfilePhase {
    /// Stable string label used by benchmark reports and logs.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SerializeParams => "serialize_params",
            Self::Transport => "transport",
            Self::DeserializeResponse => "deserialize_response",
            Self::DecodeBinary => "decode_binary",
        }
    }
}

/// Single profiling sample emitted by the optional API profiler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiProfileEvent {
    /// Request method being profiled.
    pub method: CompactString,
    /// Transport label such as `msgpack` or `jsonrpc`.
    pub transport: CompactString,
    /// Fine-grained phase inside the request lifecycle.
    pub phase: ApiProfilePhase,
    /// Measured duration for the phase.
    pub duration: Duration,
}

/// Sink for fine-grained API profiling samples.
pub trait ApiProfiler: Send + Sync + 'static {
    /// Receives a single profiling event.
    fn on_profile(&self, event: &ApiProfileEvent);
}

/// Shared profiler handle passed through spawn configs and clients.
pub type SharedProfiler = Arc<dyn ApiProfiler>;

pub(crate) fn profile(profiler: Option<&SharedProfiler>, event: ApiProfileEvent) {
    if let Some(profiler) = profiler {
        profiler.on_profile(&event);
    }
}
