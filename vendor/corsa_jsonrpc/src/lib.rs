//! JSON-RPC framing and transport utilities used by the Corsa integrations.
//!
//! The types in this crate are transport-focused rather than compiler-focused:
//! they help you read and write JSON-RPC 2.0 messages over stdio or sockets,
//! route locally handled callbacks, and surface unmatched inbound requests to
//! callers.
//!
//! Reach for [`JsonRpcConnection`] when you need a reusable, thread-backed
//! request/response channel, or for [`RawMessage`] and [`RequestId`] when you
//! need to work at the protocol boundary directly.

/// Re-exports structured operational events used by transport configs.
pub mod observability {
    pub use corsa_core::{CorsaEvent, CorsaObserver, SharedObserver};
}

pub use corsa_core::{
    CorsaError, CorsaEvent, CorsaObserver, Result, RpcResponseError, SharedObserver,
};

#[path = "jsonrpc/mod.rs"]
/// JSON-RPC 2.0 message and connection primitives.
pub mod jsonrpc;

pub use jsonrpc::*;
