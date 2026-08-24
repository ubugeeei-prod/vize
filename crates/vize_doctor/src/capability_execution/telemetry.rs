use serde::Serialize;
use vize_carton::String;

use super::CapabilityExecutionOutcome;
use crate::{CapabilityCacheKey, ContentFingerprint};

/// Cache status recorded for one capability execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityExecutionCacheStatus {
    /// The cache supplied a trusted snapshot and analysis did not run.
    Hit,
    /// Analysis ran and the resulting snapshot was accepted by storage.
    Miss,
}

/// Machine-readable cache telemetry for one capability execution outcome.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityExecutionTelemetry {
    capability: String,
    cache_key: CapabilityCacheKey,
    cache_status: CapabilityExecutionCacheStatus,
    finding_count: usize,
    output_fingerprint: ContentFingerprint,
}

impl CapabilityExecutionTelemetry {
    pub(super) fn from_outcome(outcome: &CapabilityExecutionOutcome) -> Self {
        let snapshot = outcome.snapshot();
        Self {
            capability: snapshot.identity().capability().into(),
            cache_key: snapshot.cache_key(),
            cache_status: match outcome {
                CapabilityExecutionOutcome::CacheHit { .. } => CapabilityExecutionCacheStatus::Hit,
                CapabilityExecutionOutcome::CacheMiss { .. } => {
                    CapabilityExecutionCacheStatus::Miss
                }
            },
            finding_count: snapshot.findings().len(),
            output_fingerprint: snapshot.output_fingerprint(),
        }
    }

    /// Returns the stable analysis capability identifier.
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// Returns the cache key used by this capability execution.
    pub const fn cache_key(&self) -> CapabilityCacheKey {
        self.cache_key
    }

    /// Returns whether the execution reused cache output or stored a miss.
    pub const fn cache_status(&self) -> CapabilityExecutionCacheStatus {
        self.cache_status
    }

    /// Returns how many findings were present in the trusted snapshot.
    pub const fn finding_count(&self) -> usize {
        self.finding_count
    }

    /// Returns the stable fingerprint of the trusted snapshot output.
    pub const fn output_fingerprint(&self) -> ContentFingerprint {
        self.output_fingerprint
    }
}

impl CapabilityExecutionOutcome {
    /// Returns deterministic cache telemetry for this execution outcome.
    #[must_use]
    pub fn telemetry(&self) -> CapabilityExecutionTelemetry {
        CapabilityExecutionTelemetry::from_outcome(self)
    }
}
