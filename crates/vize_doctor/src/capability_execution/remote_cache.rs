//! Provider-neutral remote persistence for validated capability snapshots.

mod object;
#[cfg(test)]
mod tests;

use std::{error::Error, fmt};

use crate::{
    CapabilityCacheKey, CapabilitySnapshot, ContentFingerprint,
    DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE, PersistentCapabilitySnapshot,
    PersistentCapabilitySnapshotError,
};

use super::CapabilitySnapshotCache;
pub use object::RemoteCapabilitySnapshotObject;

/// Object-store operations required by the remote snapshot cache.
///
/// Implementations own authentication, network retries, and provider-specific
/// addressing. Doctor supplies the canonical logical cache key, exact media
/// type, portable envelope bytes, and an explicit response ceiling.
///
/// [`Self::publish_object`] must be atomic and immutable: store `object` only
/// when its key is absent, never replace an existing object, and return the
/// exact winner. Both methods must reject a response larger than
/// `maximum_response_bytes` instead of truncating it. The cache validates those
/// invariants again before accepting a hit or store acknowledgement.
pub trait RemoteCapabilitySnapshotStore {
    /// Error returned by the provider-specific transport.
    type Error;

    /// Loads the object stored under one canonical capability cache key.
    fn load_object(
        &mut self,
        cache_key: CapabilityCacheKey,
        maximum_response_bytes: usize,
    ) -> Result<Option<RemoteCapabilitySnapshotObject>, Self::Error>;

    /// Publishes an immutable object and returns the accepted winner.
    fn publish_object(
        &mut self,
        object: &RemoteCapabilitySnapshotObject,
        maximum_response_bytes: usize,
    ) -> Result<RemoteCapabilitySnapshotObject, Self::Error>;
}

/// Capability snapshot cache backed by a provider-neutral remote object store.
///
/// The adapter owns portable envelope encoding and validation. The supplied
/// store owns only transport and conditional publication, keeping credentials,
/// URLs, retry policies, and provider metadata outside Doctor's cache format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCapabilitySnapshotCache<Store> {
    store: Store,
    maximum_snapshot_bytes: usize,
}

impl<Store> RemoteCapabilitySnapshotCache<Store> {
    /// Creates a remote cache with an explicit per-object response ceiling.
    pub const fn new(store: Store, maximum_snapshot_bytes: usize) -> Self {
        Self {
            store,
            maximum_snapshot_bytes,
        }
    }

    /// Returns the remote object store.
    pub const fn store(&self) -> &Store {
        &self.store
    }

    /// Returns the remote object store mutably.
    pub const fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }

    /// Consumes the cache and returns the remote object store.
    pub fn into_store(self) -> Store {
        self.store
    }

    /// Returns the maximum accepted bytes for one remote object.
    pub const fn maximum_snapshot_bytes(&self) -> usize {
        self.maximum_snapshot_bytes
    }

    fn decode_object<StoreError>(
        &self,
        requested_cache_key: CapabilityCacheKey,
        object: RemoteCapabilitySnapshotObject,
    ) -> Result<CapabilitySnapshot, RemoteCapabilitySnapshotCacheError<StoreError>> {
        if object.bytes().len() > self.maximum_snapshot_bytes {
            return Err(RemoteCapabilitySnapshotCacheError::Envelope {
                source: PersistentCapabilitySnapshotError::PayloadTooLarge {
                    actual: object.bytes().len(),
                    maximum: self.maximum_snapshot_bytes,
                },
            });
        }
        if object.cache_key() != requested_cache_key {
            return Err(RemoteCapabilitySnapshotCacheError::ObjectCacheKeyMismatch {
                requested: requested_cache_key,
                actual: object.cache_key(),
            });
        }
        if object.media_type() != DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE {
            return Err(
                RemoteCapabilitySnapshotCacheError::ObjectMediaTypeMismatch {
                    actual: object.media_type().into(),
                },
            );
        }

        PersistentCapabilitySnapshot::from_json_slice(
            requested_cache_key,
            object.bytes(),
            self.maximum_snapshot_bytes,
        )
        .map(PersistentCapabilitySnapshot::into_snapshot)
        .map_err(|source| RemoteCapabilitySnapshotCacheError::Envelope { source })
    }
}

impl<Store> CapabilitySnapshotCache for RemoteCapabilitySnapshotCache<Store>
where
    Store: RemoteCapabilitySnapshotStore,
{
    type Error = RemoteCapabilitySnapshotCacheError<Store::Error>;

    fn load_snapshot(
        &mut self,
        cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        let object = self
            .store
            .load_object(cache_key, self.maximum_snapshot_bytes)
            .map_err(|source| RemoteCapabilitySnapshotCacheError::Load { source })?;
        object
            .map(|object| self.decode_object(cache_key, object))
            .transpose()
    }

    fn store_snapshot(
        &mut self,
        snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        let envelope = PersistentCapabilitySnapshot::try_new(snapshot.clone())
            .map_err(|source| RemoteCapabilitySnapshotCacheError::Envelope { source })?;
        let bytes = envelope
            .to_json_bytes()
            .map_err(|source| RemoteCapabilitySnapshotCacheError::Envelope { source })?;
        if bytes.len() > self.maximum_snapshot_bytes {
            return Err(RemoteCapabilitySnapshotCacheError::Envelope {
                source: PersistentCapabilitySnapshotError::PayloadTooLarge {
                    actual: bytes.len(),
                    maximum: self.maximum_snapshot_bytes,
                },
            });
        }

        let cache_key = snapshot.cache_key();
        let object = RemoteCapabilitySnapshotObject::new(
            cache_key,
            DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE,
            bytes,
        );
        let accepted = self
            .store
            .publish_object(&object, self.maximum_snapshot_bytes)
            .map_err(|source| RemoteCapabilitySnapshotCacheError::Publish { source })?;
        let stored = self.decode_object(cache_key, accepted)?;
        if stored != *snapshot {
            return Err(RemoteCapabilitySnapshotCacheError::ConflictingSnapshot {
                cache_key,
                existing_output: stored.output_fingerprint(),
                incoming_output: snapshot.output_fingerprint(),
            });
        }
        Ok(stored)
    }
}

/// Failure while loading or publishing a remote capability snapshot.
#[derive(Debug)]
#[non_exhaustive]
pub enum RemoteCapabilitySnapshotCacheError<StoreError> {
    /// The remote store failed while loading a possible cache hit.
    Load {
        /// Provider-specific transport error.
        source: StoreError,
    },
    /// The remote store failed while conditionally publishing a cache miss.
    Publish {
        /// Provider-specific transport error.
        source: StoreError,
    },
    /// A remote response named a different logical object key.
    ObjectCacheKeyMismatch {
        /// Requested canonical cache key.
        requested: CapabilityCacheKey,
        /// Key declared by the returned object.
        actual: CapabilityCacheKey,
    },
    /// Remote transport metadata does not carry the persistent envelope media type.
    ObjectMediaTypeMismatch {
        /// Media type returned by the remote store.
        actual: vize_l0::String,
    },
    /// Persistent envelope encoding or validation failed.
    Envelope {
        /// Fail-closed envelope error.
        source: PersistentCapabilitySnapshotError,
    },
    /// One immutable cache key already names different validated output.
    ConflictingSnapshot {
        /// Conflicting capability cache key.
        cache_key: CapabilityCacheKey,
        /// Output fingerprint already stored.
        existing_output: ContentFingerprint,
        /// Output fingerprint rejected by the cache.
        incoming_output: ContentFingerprint,
    },
}

impl<StoreError> fmt::Display for RemoteCapabilitySnapshotCacheError<StoreError>
where
    StoreError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load { source } => write!(formatter, "remote cache object load failed: {source}"),
            Self::Publish { source } => {
                write!(
                    formatter,
                    "remote cache object publication failed: {source}"
                )
            }
            Self::ObjectCacheKeyMismatch { requested, actual } => write!(
                formatter,
                "remote cache returned object key {actual}; requested {requested}"
            ),
            Self::ObjectMediaTypeMismatch { actual } => write!(
                formatter,
                "remote cache returned unsupported media type {actual:?}"
            ),
            Self::Envelope { source } => source.fmt(formatter),
            Self::ConflictingSnapshot {
                cache_key,
                existing_output,
                incoming_output,
            } => write!(
                formatter,
                "remote capability snapshot key {cache_key} already stores output {existing_output}; rejected {incoming_output}"
            ),
        }
    }
}

impl<StoreError> Error for RemoteCapabilitySnapshotCacheError<StoreError>
where
    StoreError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Load { source } | Self::Publish { source } => Some(source),
            Self::Envelope { source } => Some(source),
            Self::ObjectCacheKeyMismatch { .. }
            | Self::ObjectMediaTypeMismatch { .. }
            | Self::ConflictingSnapshot { .. } => None,
        }
    }
}
