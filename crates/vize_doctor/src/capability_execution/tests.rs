use std::{
    cell::Cell,
    collections::BTreeMap,
    convert::Infallible,
    error::Error,
    fmt::{self, Display},
};

use super::{
    CapabilityExecutionCacheStatus, CapabilityExecutionError, CapabilitySnapshotCache,
    MemoryCapabilitySnapshotCache, MemoryCapabilitySnapshotCacheError, execute_cached_capability,
};
use crate::{
    AnalysisProvenance, CapabilityCacheIdentity, CapabilityCacheKey, CapabilitySnapshot,
    ContentFingerprint, DoctorCategory, DoctorFinding, FindingAssessment, FindingConfidence,
    FindingImpact, FindingSeverity, HealthPenalty, RuleCost, SourceLocation,
};

fn fingerprint(value: &str) -> ContentFingerprint {
    ContentFingerprint::digest(value)
}

fn identity(capability: &str, source: &str) -> CapabilityCacheIdentity {
    CapabilityCacheIdentity::from_fingerprints(
        capability,
        fingerprint("implementation"),
        fingerprint("configuration"),
        [("src/App.vue", fingerprint(source))],
    )
    .unwrap()
}

fn finding(capability: &str, code: &str, source: &str) -> DoctorFinding {
    DoctorFinding::new(
        code,
        DoctorCategory::Correctness,
        FindingAssessment::new(
            FindingSeverity::Warning,
            FindingConfidence::Certain,
            FindingImpact::Medium,
            HealthPenalty::new(10, "test"),
        ),
        SourceLocation::new("src/App.vue", 0, 1),
        "Test finding",
        "Test message",
        AnalysisProvenance::new(capability, RuleCost::Low).with_invalidation_fingerprints(
            BTreeMap::from([("src/App.vue".into(), fingerprint(source))]),
        ),
    )
}

fn snapshot(capability: &str, code: &str, source: &str) -> CapabilitySnapshot {
    CapabilitySnapshot::try_new(
        identity(capability, source),
        [finding(capability, code, source)],
    )
    .unwrap()
}

fn unreachable_analysis(
    _identity: &CapabilityCacheIdentity,
) -> Result<[DoctorFinding; 0], Infallible> {
    panic!("analysis must not run on a trusted cache path")
}

#[test]
fn cache_hit_is_identity_bound_and_does_not_run_analysis() {
    let identity = identity("template-semantics", "source-a");
    let snapshot = CapabilitySnapshot::try_new(
        identity.clone(),
        [finding("template-semantics", "VIZE_A", "source-a")],
    )
    .unwrap();
    let mut cache = MemoryCapabilitySnapshotCache::new();
    cache.store_snapshot(&snapshot).unwrap();

    let outcome = execute_cached_capability(&mut cache, identity, unreachable_analysis).unwrap();

    assert!(outcome.is_cache_hit());
    assert!(!outcome.is_cache_miss());
    assert_eq!(outcome.snapshot(), &snapshot);
}

#[test]
fn miss_runs_analysis_once_and_returns_after_store() {
    let identity = identity("template-semantics", "source-a");
    let calls = Cell::new(0);
    let mut cache = MemoryCapabilitySnapshotCache::new();

    let outcome = execute_cached_capability(&mut cache, identity.clone(), |requested| {
        calls.set(calls.get() + 1);
        assert_eq!(requested, &identity);
        Ok::<_, Infallible>([finding("template-semantics", "VIZE_A", "source-a")])
    })
    .unwrap();

    assert!(outcome.is_cache_miss());
    assert_eq!(calls.get(), 1);
    assert_eq!(cache.len(), 1);

    let cached = execute_cached_capability(&mut cache, identity, unreachable_analysis).unwrap();
    assert!(cached.is_cache_hit());
    assert_eq!(calls.get(), 1);
}

#[test]
fn execution_outcome_reports_stable_cache_telemetry() {
    let identity = identity("template-semantics", "source-a");
    let mut cache = MemoryCapabilitySnapshotCache::new();

    let miss = execute_cached_capability(&mut cache, identity.clone(), |_| {
        Ok::<_, Infallible>([finding("template-semantics", "VIZE_A", "source-a")])
    })
    .unwrap();
    let miss_telemetry = miss.telemetry();
    assert_eq!(
        miss_telemetry.cache_status(),
        CapabilityExecutionCacheStatus::Miss
    );
    assert_eq!(miss_telemetry.cache_key(), identity.cache_key());
    assert_eq!(miss_telemetry.finding_count(), 1);
    assert_eq!(
        miss_telemetry.output_fingerprint(),
        miss.snapshot().output_fingerprint()
    );

    let hit = execute_cached_capability(&mut cache, identity, unreachable_analysis).unwrap();
    assert_eq!(
        hit.telemetry().cache_status(),
        CapabilityExecutionCacheStatus::Hit
    );
    assert_eq!(
        serde_json::to_value(hit.telemetry()).unwrap()["findingCount"],
        1
    );
}

#[test]
fn untrusted_hit_with_wrong_identity_fails_without_analysis() {
    let requested = identity("template-semantics", "source-a");
    let wrong = snapshot("type-semantics", "VIZE_TYPE", "source-b");
    let mut cache = PoisonedLoadCache { snapshot: wrong };

    let error =
        execute_cached_capability(&mut cache, requested.clone(), unreachable_analysis).unwrap_err();

    assert!(matches!(
        error,
        CapabilityExecutionError::CacheHitIdentityMismatch { expected, actual }
            if *expected == requested && actual.capability() == "type-semantics"
    ));
}

#[test]
fn store_acknowledgement_with_divergent_output_fails_closed() {
    let identity = identity("template-semantics", "source-a");
    let divergent = CapabilitySnapshot::try_new(
        identity.clone(),
        [finding("template-semantics", "VIZE_B", "source-a")],
    )
    .unwrap();
    let mut cache = DivergentStoreCache {
        stored: divergent.clone(),
    };

    let error = execute_cached_capability(&mut cache, identity.clone(), |_| {
        Ok::<_, Infallible>([finding("template-semantics", "VIZE_A", "source-a")])
    })
    .unwrap_err();

    assert!(matches!(
        error,
        CapabilityExecutionError::StoredSnapshotOutputConflict {
            cache_key,
            actual,
            ..
        } if cache_key == identity.cache_key() && actual == divergent.output_fingerprint()
    ));
}

#[test]
fn store_errors_do_not_return_cache_misses() {
    let identity = identity("template-semantics", "source-a");
    let mut cache = FailingStoreCache;

    let error = execute_cached_capability(&mut cache, identity, |_| {
        Ok::<_, Infallible>([finding("template-semantics", "VIZE_A", "source-a")])
    })
    .unwrap_err();

    assert!(matches!(
        error,
        CapabilityExecutionError::CacheStore {
            source: TestCacheError::Store
        }
    ));
}

#[test]
fn invalid_analysis_output_is_not_stored() {
    let identity = identity("template-semantics", "source-a");
    let mut cache = MemoryCapabilitySnapshotCache::new();

    let error = execute_cached_capability(&mut cache, identity, |_| {
        Ok::<_, Infallible>([finding("type-semantics", "VIZE_TYPE", "source-a")])
    })
    .unwrap_err();

    assert!(matches!(error, CapabilityExecutionError::Snapshot { .. }));
    assert!(cache.is_empty());
}

#[test]
fn memory_cache_rejects_divergent_output_but_accepts_identical_restores() {
    let first = snapshot("template-semantics", "VIZE_A", "source-a");
    let second = CapabilitySnapshot::try_new(
        first.identity().clone(),
        [finding("template-semantics", "VIZE_B", "source-a")],
    )
    .unwrap();
    let mut cache = MemoryCapabilitySnapshotCache::new();

    assert_eq!(cache.store_snapshot(&first).unwrap(), first);
    assert_eq!(cache.store_snapshot(&first).unwrap(), first);

    let error = cache.store_snapshot(&second).unwrap_err();
    assert!(matches!(
        error,
        MemoryCapabilitySnapshotCacheError::ConflictingSnapshot {
            cache_key,
            existing_output,
            incoming_output,
        } if cache_key == first.cache_key()
            && existing_output == first.output_fingerprint()
            && incoming_output == second.output_fingerprint()
    ));
}

struct PoisonedLoadCache {
    snapshot: CapabilitySnapshot,
}

impl CapabilitySnapshotCache for PoisonedLoadCache {
    type Error = Infallible;

    fn load_snapshot(
        &mut self,
        _cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        Ok(Some(self.snapshot.clone()))
    }

    fn store_snapshot(
        &mut self,
        _snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        unreachable!("cache hit rejection must not store")
    }
}

struct DivergentStoreCache {
    stored: CapabilitySnapshot,
}

impl CapabilitySnapshotCache for DivergentStoreCache {
    type Error = Infallible;

    fn load_snapshot(
        &mut self,
        _cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        Ok(None)
    }

    fn store_snapshot(
        &mut self,
        _snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        Ok(self.stored.clone())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TestCacheError {
    Store,
}

impl Display for TestCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store => formatter.write_str("store failed"),
        }
    }
}

impl Error for TestCacheError {}

struct FailingStoreCache;

impl CapabilitySnapshotCache for FailingStoreCache {
    type Error = TestCacheError;

    fn load_snapshot(
        &mut self,
        _cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        Ok(None)
    }

    fn store_snapshot(
        &mut self,
        _snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        Err(TestCacheError::Store)
    }
}
