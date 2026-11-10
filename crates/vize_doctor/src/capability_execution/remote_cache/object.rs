//! Transport object shared by remote capability snapshot stores.

use vize_l0::String;

use crate::CapabilityCacheKey;

/// Untrusted object returned by a remote snapshot store.
///
/// The logical key and media type are transport metadata rather than envelope
/// fields. [`super::RemoteCapabilitySnapshotCache`] independently validates
/// both before decoding `bytes`, so implementations can represent a malformed
/// provider response without bypassing Doctor's acceptance boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCapabilitySnapshotObject {
    cache_key: CapabilityCacheKey,
    media_type: String,
    bytes: Vec<u8>,
}

impl RemoteCapabilitySnapshotObject {
    /// Creates an untrusted remote object response or publication request.
    pub fn new(
        cache_key: CapabilityCacheKey,
        media_type: impl Into<String>,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            cache_key,
            media_type: media_type.into(),
            bytes,
        }
    }

    /// Returns the logical cache key carried by transport metadata.
    pub const fn cache_key(&self) -> CapabilityCacheKey {
        self.cache_key
    }

    /// Returns the media type carried by transport metadata.
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Returns the complete remote object bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Consumes the object and returns its complete bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}
