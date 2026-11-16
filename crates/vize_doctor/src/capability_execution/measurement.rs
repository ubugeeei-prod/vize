//! Process-local latency measurement for cache-backed capability execution.

#[cfg(test)]
mod tests;

use std::{cell::Cell, time::Instant};

use serde::Serialize;

use super::{
    CapabilityExecutionError, CapabilityExecutionOutcome, CapabilitySnapshotCache,
    execute_cached_capability,
};
use crate::{CapabilityCacheIdentity, CapabilityCacheKey, CapabilitySnapshot, DoctorFinding};

/// Wire version emitted by [`CapabilityExecutionLatencyTelemetry`].
pub const DOCTOR_CAPABILITY_EXECUTION_LATENCY_TELEMETRY_VERSION: u16 = 1;

/// Process-local stage latency for one successful capability execution.
///
/// Cache load time is always present. Analysis and cache store time are present
/// only for a cache miss. Total time also includes snapshot validation and the
/// small amount of orchestration work between measured stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityExecutionLatencyTelemetry {
    schema_version: u16,
    cache_load_time_ns: u64,
    analysis_time_ns: Option<u64>,
    cache_store_time_ns: Option<u64>,
    total_time_ns: u64,
}

impl CapabilityExecutionLatencyTelemetry {
    fn new(
        cache_load_time_ns: u64,
        analysis_time_ns: Option<u64>,
        cache_store_time_ns: Option<u64>,
        total_time_ns: u64,
    ) -> Self {
        Self {
            schema_version: DOCTOR_CAPABILITY_EXECUTION_LATENCY_TELEMETRY_VERSION,
            cache_load_time_ns,
            analysis_time_ns,
            cache_store_time_ns,
            total_time_ns,
        }
    }

    /// Returns the telemetry wire version.
    pub const fn schema_version(self) -> u16 {
        self.schema_version
    }

    /// Returns time spent asking the cache for a validated snapshot.
    pub const fn cache_load_time_ns(self) -> u64 {
        self.cache_load_time_ns
    }

    /// Returns analysis-runner time, or `None` when the cache supplied a hit.
    pub const fn analysis_time_ns(self) -> Option<u64> {
        self.analysis_time_ns
    }

    /// Returns cache-store time, or `None` when the cache supplied a hit.
    pub const fn cache_store_time_ns(self) -> Option<u64> {
        self.cache_store_time_ns
    }

    /// Returns end-to-end cache-backed execution time.
    pub const fn total_time_ns(self) -> u64 {
        self.total_time_ns
    }
}

/// Successful cache-backed execution paired with process-local latency.
///
/// The trusted outcome remains separate from nondeterministic wall-clock
/// measurement so callers can keep persisted reports reproducible while
/// exporting latency to observability or performance-budget consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasuredCapabilityExecution {
    outcome: CapabilityExecutionOutcome,
    latency: CapabilityExecutionLatencyTelemetry,
}

impl MeasuredCapabilityExecution {
    /// Returns the trusted cache-backed execution outcome.
    pub const fn outcome(&self) -> &CapabilityExecutionOutcome {
        &self.outcome
    }

    /// Consumes the measurement and returns the trusted execution outcome.
    pub fn into_outcome(self) -> CapabilityExecutionOutcome {
        self.outcome
    }

    /// Returns process-local stage latency for this execution.
    pub const fn latency(&self) -> CapabilityExecutionLatencyTelemetry {
        self.latency
    }
}

/// Executes a capability and records process-local cache and analysis latency.
///
/// This preserves every validation and fail-closed error contract of
/// [`execute_cached_capability`]. Measurements are returned only with a
/// successful trusted outcome; callers must not persist them inside a
/// deterministic [`crate::DoctorReport`].
pub fn execute_cached_capability_measured<Cache, Analyze, Findings, AnalysisError>(
    cache: &mut Cache,
    identity: CapabilityCacheIdentity,
    analyze: Analyze,
) -> Result<MeasuredCapabilityExecution, CapabilityExecutionError<Cache::Error, AnalysisError>>
where
    Cache: CapabilitySnapshotCache,
    Analyze: FnOnce(&CapabilityCacheIdentity) -> Result<Findings, AnalysisError>,
    Findings: IntoIterator<Item = DoctorFinding>,
{
    let epoch = Instant::now();
    execute_cached_capability_measured_with_clock(cache, identity, analyze, || {
        epoch.elapsed().as_nanos().try_into().unwrap_or(u64::MAX)
    })
}

fn execute_cached_capability_measured_with_clock<Cache, Analyze, Findings, AnalysisError, Clock>(
    cache: &mut Cache,
    identity: CapabilityCacheIdentity,
    analyze: Analyze,
    clock: Clock,
) -> Result<MeasuredCapabilityExecution, CapabilityExecutionError<Cache::Error, AnalysisError>>
where
    Cache: CapabilitySnapshotCache,
    Analyze: FnOnce(&CapabilityCacheIdentity) -> Result<Findings, AnalysisError>,
    Findings: IntoIterator<Item = DoctorFinding>,
    Clock: Fn() -> u64,
{
    let total_started = clock();
    let analysis_time_ns = Cell::new(None);
    let mut measured_cache = MeasuredCache::new(cache, &clock);
    let outcome = execute_cached_capability(&mut measured_cache, identity, |requested| {
        let started = clock();
        let result = analyze(requested);
        analysis_time_ns.set(Some(clock().saturating_sub(started)));
        result
    })?;
    let total_time_ns = clock().saturating_sub(total_started);
    let latency = CapabilityExecutionLatencyTelemetry::new(
        measured_cache.cache_load_time_ns,
        analysis_time_ns.get(),
        measured_cache.cache_store_time_ns,
        total_time_ns,
    );

    Ok(MeasuredCapabilityExecution { outcome, latency })
}

struct MeasuredCache<'cache, 'clock, Cache, Clock> {
    inner: &'cache mut Cache,
    clock: &'clock Clock,
    cache_load_time_ns: u64,
    cache_store_time_ns: Option<u64>,
}

impl<'cache, 'clock, Cache, Clock> MeasuredCache<'cache, 'clock, Cache, Clock> {
    const fn new(inner: &'cache mut Cache, clock: &'clock Clock) -> Self {
        Self {
            inner,
            clock,
            cache_load_time_ns: 0,
            cache_store_time_ns: None,
        }
    }
}

impl<Cache, Clock> CapabilitySnapshotCache for MeasuredCache<'_, '_, Cache, Clock>
where
    Cache: CapabilitySnapshotCache,
    Clock: Fn() -> u64,
{
    type Error = Cache::Error;

    fn load_snapshot(
        &mut self,
        cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        let started = (self.clock)();
        let result = self.inner.load_snapshot(cache_key);
        self.cache_load_time_ns = (self.clock)().saturating_sub(started);
        result
    }

    fn store_snapshot(
        &mut self,
        snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        let started = (self.clock)();
        let result = self.inner.store_snapshot(snapshot);
        self.cache_store_time_ns = Some((self.clock)().saturating_sub(started));
        result
    }
}
