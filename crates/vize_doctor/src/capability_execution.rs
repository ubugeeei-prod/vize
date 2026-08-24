//! Cache-backed execution for independently reusable analysis capabilities.

mod telemetry;
#[cfg(test)]
mod tests;

use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display},
};

use crate::{
    CapabilityCacheIdentity, CapabilityCacheKey, CapabilitySnapshot, CapabilitySnapshotError,
    ContentFingerprint, DoctorFinding,
};

pub use telemetry::{CapabilityExecutionCacheStatus, CapabilityExecutionTelemetry};

/// Cache storage for validated capability snapshots.
///
/// Implementations may be local, remote, in-memory, or layered. The execution
/// helper treats every returned value as untrusted: cache hits and post-store
/// acknowledgements are accepted only when their embedded identity exactly
/// matches the requested identity.
pub trait CapabilitySnapshotCache {
    /// Error returned by the cache backend.
    type Error;

    /// Loads a snapshot stored under the derived capability cache key.
    fn load_snapshot(
        &mut self,
        cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error>;

    /// Stores a validated snapshot and returns the snapshot accepted by storage.
    ///
    /// Returning the accepted value lets callers detect conflicting writes from
    /// untrusted or concurrent backends before reporting a cache miss.
    fn store_snapshot(
        &mut self,
        snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error>;
}

/// Outcome of one cache-backed capability execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityExecutionOutcome {
    /// The cache supplied a trusted snapshot and analysis was not invoked.
    CacheHit {
        /// Identity-bound snapshot loaded from cache.
        snapshot: CapabilitySnapshot,
    },
    /// Analysis ran and its validated snapshot was accepted by the cache.
    CacheMiss {
        /// Identity-bound snapshot produced by analysis and stored in cache.
        snapshot: CapabilitySnapshot,
    },
}

impl CapabilityExecutionOutcome {
    /// Returns whether this outcome reused cached output.
    pub const fn is_cache_hit(&self) -> bool {
        matches!(self, Self::CacheHit { .. })
    }

    /// Returns whether this outcome ran analysis and stored its output.
    pub const fn is_cache_miss(&self) -> bool {
        matches!(self, Self::CacheMiss { .. })
    }

    /// Returns the trusted snapshot for this outcome.
    pub const fn snapshot(&self) -> &CapabilitySnapshot {
        match self {
            Self::CacheHit { snapshot } | Self::CacheMiss { snapshot } => snapshot,
        }
    }

    /// Consumes the outcome and returns its trusted snapshot.
    pub fn into_snapshot(self) -> CapabilitySnapshot {
        match self {
            Self::CacheHit { snapshot } | Self::CacheMiss { snapshot } => snapshot,
        }
    }
}

/// Failure while loading, running, validating, or storing a capability result.
#[derive(Debug, PartialEq, Eq)]
pub enum CapabilityExecutionError<CacheError, AnalysisError> {
    /// The cache backend failed while loading a possible hit.
    CacheLoad {
        /// Backend error.
        source: CacheError,
    },
    /// The cache backend failed while storing a validated miss.
    CacheStore {
        /// Backend error.
        source: CacheError,
    },
    /// The analysis runner failed before a cacheable snapshot could be built.
    Analysis {
        /// Runner error.
        source: AnalysisError,
    },
    /// Analysis produced findings that did not satisfy the snapshot contract.
    Snapshot {
        /// Snapshot validation error.
        source: CapabilitySnapshotError,
    },
    /// A cache hit returned a snapshot for a different identity.
    CacheHitIdentityMismatch {
        /// Requested identity.
        expected: Box<CapabilityCacheIdentity>,
        /// Identity embedded in the cached snapshot.
        actual: Box<CapabilityCacheIdentity>,
    },
    /// A store acknowledgement returned a snapshot for a different identity.
    StoredSnapshotIdentityMismatch {
        /// Requested identity.
        expected: Box<CapabilityCacheIdentity>,
        /// Identity embedded in the stored snapshot.
        actual: Box<CapabilityCacheIdentity>,
    },
    /// A store acknowledgement returned different output for the requested identity.
    StoredSnapshotOutputConflict {
        /// Cache key shared by the requested identity and stored snapshot.
        cache_key: CapabilityCacheKey,
        /// Output fingerprint produced by the current analysis run.
        expected: ContentFingerprint,
        /// Output fingerprint returned by storage.
        actual: ContentFingerprint,
    },
}

impl<CacheError, AnalysisError> Display for CapabilityExecutionError<CacheError, AnalysisError>
where
    CacheError: Display,
    AnalysisError: Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CacheLoad { source } => {
                write!(formatter, "capability snapshot cache load failed: {source}")
            }
            Self::CacheStore { source } => {
                write!(
                    formatter,
                    "capability snapshot cache store failed: {source}"
                )
            }
            Self::Analysis { source } => {
                write!(
                    formatter,
                    "capability analysis failed before producing a cacheable snapshot: {source}"
                )
            }
            Self::Snapshot { source } => {
                write!(
                    formatter,
                    "capability analysis produced an invalid snapshot: {source}"
                )
            }
            Self::CacheHitIdentityMismatch { .. } => formatter
                .write_str("cached capability snapshot is not bound to the requested identity"),
            Self::StoredSnapshotIdentityMismatch { .. } => formatter
                .write_str("stored capability snapshot is not bound to the requested identity"),
            Self::StoredSnapshotOutputConflict {
                cache_key,
                expected,
                actual,
            } => write!(
                formatter,
                "stored capability snapshot for {cache_key} has output fingerprint {actual}; expected {expected}"
            ),
        }
    }
}

impl<CacheError, AnalysisError> Error for CapabilityExecutionError<CacheError, AnalysisError>
where
    CacheError: Error + 'static,
    AnalysisError: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CacheLoad { source } | Self::CacheStore { source } => Some(source),
            Self::Analysis { source } => Some(source),
            Self::Snapshot { source } => Some(source),
            Self::CacheHitIdentityMismatch { .. }
            | Self::StoredSnapshotIdentityMismatch { .. }
            | Self::StoredSnapshotOutputConflict { .. } => None,
        }
    }
}

/// Executes a capability with fail-closed snapshot reuse.
///
/// A trusted hit returns without invoking `analyze`. On a miss, the runner's
/// findings are normalized and validated as a [`CapabilitySnapshot`], stored,
/// and then checked against the cache's acknowledgement. The function returns a
/// miss only after storage accepts the exact identity and output.
pub fn execute_cached_capability<Cache, Analyze, Findings, AnalysisError>(
    cache: &mut Cache,
    identity: CapabilityCacheIdentity,
    analyze: Analyze,
) -> Result<CapabilityExecutionOutcome, CapabilityExecutionError<Cache::Error, AnalysisError>>
where
    Cache: CapabilitySnapshotCache,
    Analyze: FnOnce(&CapabilityCacheIdentity) -> Result<Findings, AnalysisError>,
    Findings: IntoIterator<Item = DoctorFinding>,
{
    let cache_key = identity.cache_key();
    if let Some(snapshot) = cache
        .load_snapshot(cache_key)
        .map_err(|source| CapabilityExecutionError::CacheLoad { source })?
    {
        if snapshot.identity() != &identity {
            return Err(CapabilityExecutionError::CacheHitIdentityMismatch {
                expected: Box::new(identity),
                actual: Box::new(snapshot.identity().clone()),
            });
        }
        return Ok(CapabilityExecutionOutcome::CacheHit { snapshot });
    }

    let findings =
        analyze(&identity).map_err(|source| CapabilityExecutionError::Analysis { source })?;
    let snapshot = CapabilitySnapshot::try_new(identity.clone(), findings)
        .map_err(|source| CapabilityExecutionError::Snapshot { source })?;
    let stored = cache
        .store_snapshot(&snapshot)
        .map_err(|source| CapabilityExecutionError::CacheStore { source })?;

    if stored.identity() != &identity {
        return Err(CapabilityExecutionError::StoredSnapshotIdentityMismatch {
            expected: Box::new(identity),
            actual: Box::new(stored.identity().clone()),
        });
    }
    if stored.output_fingerprint() != snapshot.output_fingerprint()
        || stored.findings() != snapshot.findings()
    {
        return Err(CapabilityExecutionError::StoredSnapshotOutputConflict {
            cache_key: snapshot.cache_key(),
            expected: snapshot.output_fingerprint(),
            actual: stored.output_fingerprint(),
        });
    }

    Ok(CapabilityExecutionOutcome::CacheMiss { snapshot: stored })
}

/// Deterministic in-memory capability snapshot cache.
#[derive(Debug, Clone, Default)]
pub struct MemoryCapabilitySnapshotCache {
    snapshots: BTreeMap<CapabilityCacheKey, CapabilitySnapshot>,
}

impl MemoryCapabilitySnapshotCache {
    /// Creates an empty memory cache.
    pub const fn new() -> Self {
        Self {
            snapshots: BTreeMap::new(),
        }
    }

    /// Returns the number of stored snapshots.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns whether the cache contains no snapshots.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

impl CapabilitySnapshotCache for MemoryCapabilitySnapshotCache {
    type Error = MemoryCapabilitySnapshotCacheError;

    fn load_snapshot(
        &mut self,
        cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        Ok(self.snapshots.get(&cache_key).cloned())
    }

    fn store_snapshot(
        &mut self,
        snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        let cache_key = snapshot.cache_key();
        match self.snapshots.get(&cache_key) {
            Some(existing) if existing == snapshot => Ok(existing.clone()),
            Some(existing) => Err(MemoryCapabilitySnapshotCacheError::ConflictingSnapshot {
                cache_key,
                existing_output: existing.output_fingerprint(),
                incoming_output: snapshot.output_fingerprint(),
            }),
            None => {
                self.snapshots.insert(cache_key, snapshot.clone());
                Ok(snapshot.clone())
            }
        }
    }
}

/// Failure while storing a snapshot in [`MemoryCapabilitySnapshotCache`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryCapabilitySnapshotCacheError {
    /// One cache key already names a different snapshot.
    ConflictingSnapshot {
        /// Conflicting cache key.
        cache_key: CapabilityCacheKey,
        /// Output fingerprint already stored.
        existing_output: ContentFingerprint,
        /// Output fingerprint rejected by the cache.
        incoming_output: ContentFingerprint,
    },
}

impl Display for MemoryCapabilitySnapshotCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingSnapshot {
                cache_key,
                existing_output,
                incoming_output,
            } => write!(
                formatter,
                "capability snapshot cache key {cache_key} already stores output {existing_output}; rejected {incoming_output}"
            ),
        }
    }
}

impl Error for MemoryCapabilitySnapshotCacheError {}
