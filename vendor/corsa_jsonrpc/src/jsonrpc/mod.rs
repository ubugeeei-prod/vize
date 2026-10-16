//! JSON-RPC message, framing, and connection helpers.
//!
//! The types re-exported from here are deliberately small and transport-agnostic
//! so they can be shared between the stdio API client, the LSP client, and test
//! fixtures.

mod connection;
#[cfg(test)]
mod connection_tests;
mod frame;
mod id;
mod message;

pub use crate::RpcResponseError;
/// Thread-backed JSON-RPC connection and inbound event stream.
pub use connection::{
    InboundEvent, JsonRpcConnection, JsonRpcConnectionOptions, RpcHandler, RpcHandlerMap,
};
/// Low-level Content-Length frame helpers.
pub use frame::{read_frame, write_frame};
/// JSON-RPC request identifier.
pub use id::RequestId;
/// Raw JSON-RPC envelope used on the wire.
pub use message::RawMessage;
