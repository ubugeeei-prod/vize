use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    convert::Infallible,
    error::Error,
    fmt::{self, Display},
};

use super::{
    LayeredCapabilitySnapshotCache, LayeredCapabilitySnapshotCacheError,
    LayeredCapabilitySnapshotTier,
};
use crate::{
    AnalysisProvenance, CapabilityCacheIdentity, CapabilityCacheKey, CapabilitySnapshot,
    CapabilitySnapshotCache, ContentFingerprint, DoctorCategory, DoctorFinding, FindingAssessment,
    FindingConfidence, FindingImpact, FindingSeverity, HealthPenalty, RuleCost, SourceLocation,
    execute_cached_capability,
};

fn identity(capability: &str) -> CapabilityCacheIdentity {
    CapabilityCacheIdentity::from_fingerprints(
        capability,
        ContentFingerprint::digest("implementation"),
        ContentFingerprint::digest("configuration"),
        [("src/App.vue", ContentFingerprint::digest("source"))],
    )
    .unwrap()
}

fn finding(title: &str) -> DoctorFinding {
    DoctorFinding::new(
        "VIZE_DOCTOR_TEMPLATE_001",
        DoctorCategory::Correctness,
        FindingAssessment::new(
            FindingSeverity::Warning,
            FindingConfidence::Certain,
            FindingImpact::Medium,
            HealthPenalty::new(10, "test"),
        ),
        SourceLocation::new("src/App.vue", 4, 12),
        title,
        "Test message",
        AnalysisProvenance::new("template-semantics", RuleCost::Low)
            .with_invalidation_fingerprints(BTreeMap::from([(
                "src/App.vue".into(),
                ContentFingerprint::digest("source"),
            )])),
    )
}

fn snapshot(title: &str) -> CapabilitySnapshot {
    CapabilitySnapshot::try_new(identity("template-semantics"), [finding(title)]).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestCacheError {
    Load,
    Store,
}

impl Display for TestCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load => formatter.write_str("injected load failure"),
            Self::Store => formatter.write_str("injected store failure"),
        }
    }
}

impl Error for TestCacheError {}

#[derive(Debug)]
struct TestCache<'a> {
    snapshots: BTreeMap<CapabilityCacheKey, CapabilitySnapshot>,
    load_error: bool,
    store_error: bool,
    store_acknowledgement: Option<CapabilitySnapshot>,
    load_label: &'static str,
    store_label: &'static str,
    operations: &'a RefCell<Vec<&'static str>>,
}

impl<'a> TestCache<'a> {
    fn empty(
        load_label: &'static str,
        store_label: &'static str,
        operations: &'a RefCell<Vec<&'static str>>,
    ) -> Self {
        Self {
            snapshots: BTreeMap::new(),
            load_error: false,
            store_error: false,
            store_acknowledgement: None,
            load_label,
            store_label,
            operations,
        }
    }

    fn with_entry(
        requested: CapabilityCacheKey,
        snapshot: CapabilitySnapshot,
        load_label: &'static str,
        store_label: &'static str,
        operations: &'a RefCell<Vec<&'static str>>,
    ) -> Self {
        let mut cache = Self::empty(load_label, store_label, operations);
        cache.snapshots.insert(requested, snapshot);
        cache
    }
}

impl CapabilitySnapshotCache for TestCache<'_> {
    type Error = TestCacheError;

    fn load_snapshot(
        &mut self,
        cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        self.operations.borrow_mut().push(self.load_label);
        if self.load_error {
            return Err(TestCacheError::Load);
        }
        Ok(self.snapshots.get(&cache_key).cloned())
    }

    fn store_snapshot(
        &mut self,
        snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        self.operations.borrow_mut().push(self.store_label);
        if self.store_error {
            return Err(TestCacheError::Store);
        }
        if let Some(acknowledgement) = &self.store_acknowledgement {
            return Ok(acknowledgement.clone());
        }
        Ok(self
            .snapshots
            .entry(snapshot.cache_key())
            .or_insert_with(|| snapshot.clone())
            .clone())
    }
}

fn caches<'a>(operations: &'a RefCell<Vec<&'static str>>) -> (TestCache<'a>, TestCache<'a>) {
    (
        TestCache::empty("front-load", "front-store", operations),
        TestCache::empty("backing-load", "backing-store", operations),
    )
}

#[test]
fn front_hit_skips_the_backing_cache() {
    let expected = snapshot("Front output");
    let operations = RefCell::new(Vec::new());
    let (mut front, backing) = caches(&operations);
    front
        .snapshots
        .insert(expected.cache_key(), expected.clone());
    let mut cache = LayeredCapabilitySnapshotCache::new(front, backing);

    assert_eq!(
        cache.load_snapshot(expected.cache_key()).unwrap(),
        Some(expected)
    );
    assert_eq!(&*operations.borrow(), &["front-load"]);
}

#[test]
fn backing_hit_is_promoted_once_and_reused_from_the_front() {
    let expected = snapshot("Durable output");
    let cache_key = expected.cache_key();
    let operations = RefCell::new(Vec::new());
    let (front, mut backing) = caches(&operations);
    backing.snapshots.insert(cache_key, expected.clone());
    let mut cache = LayeredCapabilitySnapshotCache::new(front, backing);

    assert_eq!(
        cache.load_snapshot(cache_key).unwrap(),
        Some(expected.clone())
    );
    assert_eq!(
        &*operations.borrow(),
        &["front-load", "backing-load", "front-store"]
    );
    assert_eq!(cache.front().snapshots.get(&cache_key), Some(&expected));

    assert_eq!(cache.load_snapshot(cache_key).unwrap(), Some(expected));
    assert_eq!(
        &*operations.borrow(),
        &["front-load", "backing-load", "front-store", "front-load"]
    );
}

#[test]
fn execution_publishes_backing_before_front_and_then_hits_front() {
    let requested = identity("template-semantics");
    let operations = RefCell::new(Vec::new());
    let (front, backing) = caches(&operations);
    let mut cache = LayeredCapabilitySnapshotCache::new(front, backing);
    let calls = Cell::new(0);

    let miss = execute_cached_capability(&mut cache, requested.clone(), |_| {
        calls.set(calls.get() + 1);
        Ok::<_, Infallible>([finding("Produced output")])
    })
    .unwrap();
    assert!(miss.is_cache_miss());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        &*operations.borrow(),
        &["front-load", "backing-load", "backing-store", "front-store"]
    );

    let hit = execute_cached_capability(&mut cache, requested, |_| {
        calls.set(calls.get() + 1);
        Ok::<_, Infallible>(Vec::new())
    })
    .unwrap();
    assert!(hit.is_cache_hit());
    assert_eq!(calls.get(), 1);
    assert_eq!(operations.borrow().last(), Some(&"front-load"));
}

#[test]
fn rejects_mismatched_backing_key_without_poisoning_the_front() {
    let expected = snapshot("Expected output");
    let requested = expected.cache_key();
    let wrong = CapabilitySnapshot::try_new(identity("script-semantics"), []).unwrap();
    let operations = RefCell::new(Vec::new());
    let front = TestCache::empty("front-load", "front-store", &operations);
    let backing = TestCache::with_entry(
        requested,
        wrong.clone(),
        "backing-load",
        "backing-store",
        &operations,
    );
    let mut cache = LayeredCapabilitySnapshotCache::new(front, backing);

    assert!(matches!(
        cache.load_snapshot(requested).unwrap_err(),
        LayeredCapabilitySnapshotCacheError::SnapshotKeyMismatch {
            tier: LayeredCapabilitySnapshotTier::Backing,
            requested: actual_requested,
            actual,
        } if actual_requested == requested && actual == wrong.cache_key()
    ));
    assert_eq!(&*operations.borrow(), &["front-load", "backing-load"]);
    assert_eq!(cache.front().snapshots.get(&requested), None);
}

#[test]
fn rejects_divergent_store_acknowledgements_at_each_tier() {
    let expected = snapshot("Expected output");
    let divergent = snapshot("Divergent output");
    let operations = RefCell::new(Vec::new());
    let (front, mut backing) = caches(&operations);
    backing.store_acknowledgement = Some(divergent.clone());
    let mut backing_conflict = LayeredCapabilitySnapshotCache::new(front, backing);
    assert!(matches!(
        backing_conflict.store_snapshot(&expected).unwrap_err(),
        LayeredCapabilitySnapshotCacheError::SnapshotOutputConflict {
            tier: LayeredCapabilitySnapshotTier::Backing,
            cache_key,
            expected_output,
            actual_output,
        } if cache_key == expected.cache_key()
            && expected_output == expected.output_fingerprint()
            && actual_output == divergent.output_fingerprint()
    ));
    assert_eq!(&*operations.borrow(), &["backing-store"]);

    operations.borrow_mut().clear();
    let (mut front, backing) = caches(&operations);
    front.store_acknowledgement = Some(divergent.clone());
    let mut front_conflict = LayeredCapabilitySnapshotCache::new(front, backing);
    assert!(matches!(
        front_conflict.store_snapshot(&expected).unwrap_err(),
        LayeredCapabilitySnapshotCacheError::SnapshotOutputConflict {
            tier: LayeredCapabilitySnapshotTier::Front,
            actual_output,
            ..
        } if actual_output == divergent.output_fingerprint()
    ));
    assert_eq!(&*operations.borrow(), &["backing-store", "front-store"]);
}

#[test]
fn failed_front_store_recovers_from_the_durable_winner() {
    let expected = snapshot("Durable output");
    let cache_key = expected.cache_key();
    let operations = RefCell::new(Vec::new());
    let (mut front, backing) = caches(&operations);
    front.store_error = true;
    let mut cache = LayeredCapabilitySnapshotCache::new(front, backing);

    assert!(matches!(
        cache.store_snapshot(&expected).unwrap_err(),
        LayeredCapabilitySnapshotCacheError::FrontStore {
            source: TestCacheError::Store
        }
    ));
    assert_eq!(cache.backing().snapshots.get(&cache_key), Some(&expected));
    assert_eq!(cache.front().snapshots.get(&cache_key), None);

    cache.front_mut().store_error = false;
    assert_eq!(
        cache.load_snapshot(cache_key).unwrap(),
        Some(expected.clone())
    );
    assert_eq!(cache.front().snapshots.get(&cache_key), Some(&expected));
    assert_eq!(
        &*operations.borrow(),
        &[
            "backing-store",
            "front-store",
            "front-load",
            "backing-load",
            "front-store"
        ]
    );
}

#[test]
fn maps_front_and_backing_load_failures_without_fallback() {
    let requested = snapshot("Expected output").cache_key();
    let operations = RefCell::new(Vec::new());
    let (mut front, backing) = caches(&operations);
    front.load_error = true;
    let mut front_failure = LayeredCapabilitySnapshotCache::new(front, backing);
    assert!(matches!(
        front_failure.load_snapshot(requested).unwrap_err(),
        LayeredCapabilitySnapshotCacheError::FrontLoad {
            source: TestCacheError::Load
        }
    ));
    assert_eq!(&*operations.borrow(), &["front-load"]);

    operations.borrow_mut().clear();
    let (front, mut backing) = caches(&operations);
    backing.load_error = true;
    let mut backing_failure = LayeredCapabilitySnapshotCache::new(front, backing);
    assert!(matches!(
        backing_failure.load_snapshot(requested).unwrap_err(),
        LayeredCapabilitySnapshotCacheError::BackingLoad {
            source: TestCacheError::Load
        }
    ));
    assert_eq!(&*operations.borrow(), &["front-load", "backing-load"]);
}
