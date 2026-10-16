use corsa_core::fast::CompactString;
use serde::{Deserialize, Serialize};

/// JSON-RPC request identifier.
///
/// # Examples
///
/// ```
/// use corsa_jsonrpc::RequestId;
///
/// assert_eq!(RequestId::integer(9).to_string(), "9");
/// assert_eq!(RequestId::string("ping").to_string(), "ping");
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RequestId {
    /// Numeric request identifier.
    Integer(i64),
    /// String request identifier.
    String(CompactString),
}

impl RequestId {
    /// Creates an integer request identifier.
    pub fn integer(id: i64) -> Self {
        Self::Integer(id)
    }

    /// Creates a string request identifier.
    pub fn string(id: impl Into<CompactString>) -> Self {
        Self::String(id.into())
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(id) => write!(f, "{id}"),
            Self::String(id) => f.write_str(id),
        }
    }
}
