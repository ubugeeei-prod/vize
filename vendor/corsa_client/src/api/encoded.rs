use serde::{Deserialize, Serialize};

/// Opaque binary payload returned by binary Corsa endpoints.
///
/// # Examples
///
/// ```
/// use corsa_client::EncodedPayload;
///
/// let payload = EncodedPayload::new(vec![1, 2, 3]);
/// assert_eq!(payload.as_bytes(), &[1, 2, 3]);
/// assert_eq!(payload.clone().into_bytes(), vec![1, 2, 3]);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedPayload(Vec<u8>);

impl EncodedPayload {
    /// Wraps raw bytes returned by Corsa.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Borrows the payload as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes the wrapper and returns the underlying bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Formatting knobs accepted by the `printNode` endpoint.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PrintNodeOptions {
    /// Preserve original source newlines where the printer can do so safely.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub preserve_source_newlines: bool,
    /// Avoid ASCII-escaping non-ASCII characters in emitted text.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub never_ascii_escape: bool,
    /// Force literal termination when the source text would otherwise be
    /// unterminated.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub terminate_unterminated_literals: bool,
}
