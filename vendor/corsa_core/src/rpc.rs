use crate::fast::CompactString;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC error payload returned by a failed response.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RpcResponseError {
    /// Numeric JSON-RPC error code.
    pub code: i64,
    /// Human-readable JSON-RPC error message.
    pub message: CompactString,
    /// Optional structured error data supplied by the peer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
