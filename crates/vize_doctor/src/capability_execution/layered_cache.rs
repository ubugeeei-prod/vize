//! Local-first composition for compatible capability snapshot caches.

#[cfg(test)]
mod tests;

use std::{error::Error, fmt};

use crate::{CapabilityCacheKey, CapabilitySnapshot, ContentFingerprint};

use super::CapabilitySnapshotCache;

/// Cache tier that returned an invalid snapshot acknowledgement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayeredCapabilitySnapshotTier {
    /// Fast front cache checked before the durable backing cache.
    Front,
    /// Durable backing cache used after a front-cache miss.
    Backing,
}

impl fmt::Display for LayeredCapabilitySnapshotTier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Front => formatter.write_str("front"),
            Self::Backing => formatter.write_str("backing"),
        }
    }
}

/// Local-first capability cache with a durable compatible backing cache.
///
/// Loads consult `front` first. A validated backing hit is promoted into the
/// front cache before it is returned. Stores publish to the durable backing
/// cache first and only then populate the front cache. If front publication
/// fails after backing publication, a later load can recover by promoting the
/// durable winner.
///
/// Both caches are untrusted. Every hit and store acknowledgement must retain
/// the requested cache key, and promoted or stored acknowledgements must equal
/// the complete snapshot supplied to that tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayeredCapabilitySnapshotCache<Front, Backing> {
    front: Front,
    backing: Backing,
}

impl<Front, Backing> LayeredCapabilitySnapshotCache<Front, Backing> {
    /// Creates a local-first cache over one durable compatible backing cache.
    pub const fn new(front: Front, backing: Backing) -> Self {
        Self { front, backing }
    }

    /// Returns the fast front cache.
    pub const fn front(&self) -> &Front {
        &self.front
    }

    /// Returns the fast front cache mutably.
    pub const fn front_mut(&mut self) -> &mut Front {
        &mut self.front
    }

    /// Returns the durable backing cache.
    pub const fn backing(&self) -> &Backing {
        &self.backing
    }

    /// Returns the durable backing cache mutably.
    pub const fn backing_mut(&mut self) -> &mut Backing {
        &mut self.backing
    }

    /// Consumes the layered cache and returns both tiers.
    pub fn into_caches(self) -> (Front, Backing) {
        (self.front, self.backing)
    }

    fn validate_key<FrontError, BackingError>(
        tier: LayeredCapabilitySnapshotTier,
        requested: CapabilityCacheKey,
        snapshot: CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, LayeredCapabilitySnapshotCacheError<FrontError, BackingError>>
    {
        let actual = snapshot.cache_key();
        if actual != requested {
            return Err(LayeredCapabilitySnapshotCacheError::SnapshotKeyMismatch {
                tier,
                requested,
                actual,
            });
        }
        Ok(snapshot)
    }

    fn validate_acknowledgement<FrontError, BackingError>(
        tier: LayeredCapabilitySnapshotTier,
        expected: &CapabilitySnapshot,
        actual: CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, LayeredCapabilitySnapshotCacheError<FrontError, BackingError>>
    {
        let actual = Self::validate_key(tier, expected.cache_key(), actual)?;
        if actual != *expected {
            return Err(
                LayeredCapabilitySnapshotCacheError::SnapshotOutputConflict {
                    tier,
                    cache_key: expected.cache_key(),
                    expected_output: expected.output_fingerprint(),
                    actual_output: actual.output_fingerprint(),
                },
            );
        }
        Ok(actual)
    }
}

impl<Front, Backing> CapabilitySnapshotCache for LayeredCapabilitySnapshotCache<Front, Backing>
where
    Front: CapabilitySnapshotCache,
    Backing: CapabilitySnapshotCache,
{
    type Error = LayeredCapabilitySnapshotCacheError<Front::Error, Backing::Error>;

    fn load_snapshot(
        &mut self,
        cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        if let Some(snapshot) = self
            .front
            .load_snapshot(cache_key)
            .map_err(|source| LayeredCapabilitySnapshotCacheError::FrontLoad { source })?
        {
            return Self::validate_key(LayeredCapabilitySnapshotTier::Front, cache_key, snapshot)
                .map(Some);
        }

        let Some(snapshot) = self
            .backing
            .load_snapshot(cache_key)
            .map_err(|source| LayeredCapabilitySnapshotCacheError::BackingLoad { source })?
        else {
            return Ok(None);
        };
        let snapshot =
            Self::validate_key(LayeredCapabilitySnapshotTier::Backing, cache_key, snapshot)?;
        let promoted = self
            .front
            .store_snapshot(&snapshot)
            .map_err(|source| LayeredCapabilitySnapshotCacheError::FrontPromotion { source })?;
        Self::validate_acknowledgement(LayeredCapabilitySnapshotTier::Front, &snapshot, promoted)
            .map(Some)
    }

    fn store_snapshot(
        &mut self,
        snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        let durable = self
            .backing
            .store_snapshot(snapshot)
            .map_err(|source| LayeredCapabilitySnapshotCacheError::BackingStore { source })?;
        let durable = Self::validate_acknowledgement(
            LayeredCapabilitySnapshotTier::Backing,
            snapshot,
            durable,
        )?;
        let front = self
            .front
            .store_snapshot(&durable)
            .map_err(|source| LayeredCapabilitySnapshotCacheError::FrontStore { source })?;
        Self::validate_acknowledgement(LayeredCapabilitySnapshotTier::Front, &durable, front)
    }
}

/// Failure while coordinating compatible front and backing snapshot caches.
#[derive(Debug)]
#[non_exhaustive]
pub enum LayeredCapabilitySnapshotCacheError<FrontError, BackingError> {
    /// The front cache failed while loading a possible hit.
    FrontLoad {
        /// Front-cache error.
        source: FrontError,
    },
    /// The backing cache failed after a front-cache miss.
    BackingLoad {
        /// Backing-cache error.
        source: BackingError,
    },
    /// The front cache failed while promoting a backing-cache hit.
    FrontPromotion {
        /// Front-cache error.
        source: FrontError,
    },
    /// The backing cache failed while durably publishing a miss.
    BackingStore {
        /// Backing-cache error.
        source: BackingError,
    },
    /// The front cache failed after the backing cache accepted a miss.
    FrontStore {
        /// Front-cache error.
        source: FrontError,
    },
    /// A cache tier returned a snapshot under a different logical key.
    SnapshotKeyMismatch {
        /// Tier that returned the mismatched snapshot.
        tier: LayeredCapabilitySnapshotTier,
        /// Requested cache key.
        requested: CapabilityCacheKey,
        /// Cache key embedded in the returned snapshot.
        actual: CapabilityCacheKey,
    },
    /// A store acknowledgement did not preserve the complete supplied snapshot.
    SnapshotOutputConflict {
        /// Tier that returned divergent output.
        tier: LayeredCapabilitySnapshotTier,
        /// Cache key shared by both snapshots.
        cache_key: CapabilityCacheKey,
        /// Output supplied to the tier.
        expected_output: ContentFingerprint,
        /// Output returned by the tier.
        actual_output: ContentFingerprint,
    },
}

impl<FrontError, BackingError> fmt::Display
    for LayeredCapabilitySnapshotCacheError<FrontError, BackingError>
where
    FrontError: fmt::Display,
    BackingError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrontLoad { source } => write!(formatter, "front cache load failed: {source}"),
            Self::BackingLoad { source } => {
                write!(formatter, "backing cache load failed: {source}")
            }
            Self::FrontPromotion { source } => {
                write!(formatter, "front cache promotion failed: {source}")
            }
            Self::BackingStore { source } => {
                write!(formatter, "backing cache store failed: {source}")
            }
            Self::FrontStore { source } => write!(formatter, "front cache store failed: {source}"),
            Self::SnapshotKeyMismatch {
                tier,
                requested,
                actual,
            } => write!(
                formatter,
                "{tier} cache returned snapshot key {actual}; requested {requested}"
            ),
            Self::SnapshotOutputConflict {
                tier,
                cache_key,
                expected_output,
                actual_output,
            } => write!(
                formatter,
                "{tier} cache acknowledgement for {cache_key} returned output {actual_output}; expected {expected_output}"
            ),
        }
    }
}

impl<FrontError, BackingError> Error
    for LayeredCapabilitySnapshotCacheError<FrontError, BackingError>
where
    FrontError: Error + 'static,
    BackingError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FrontLoad { source }
            | Self::FrontPromotion { source }
            | Self::FrontStore { source } => Some(source),
            Self::BackingLoad { source } | Self::BackingStore { source } => Some(source),
            Self::SnapshotKeyMismatch { .. } | Self::SnapshotOutputConflict { .. } => None,
        }
    }
}
