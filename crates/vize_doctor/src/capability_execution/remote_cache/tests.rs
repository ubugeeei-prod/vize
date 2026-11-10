use std::{
    cell::Cell,
    collections::BTreeMap,
    convert::Infallible,
    error::Error,
    fmt::{self, Display},
};

use super::{
    RemoteCapabilitySnapshotCache, RemoteCapabilitySnapshotCacheError,
    RemoteCapabilitySnapshotObject, RemoteCapabilitySnapshotStore,
};
use crate::{
    AnalysisProvenance, CapabilityCacheIdentity, CapabilityCacheKey, CapabilitySnapshot,
    CapabilitySnapshotCache, ContentFingerprint, DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE,
    DoctorCategory, DoctorFinding, FindingAssessment, FindingConfidence, FindingImpact,
    FindingSeverity, HealthPenalty, PersistentCapabilitySnapshot,
    PersistentCapabilitySnapshotError, RuleCost, SourceLocation, execute_cached_capability,
};

const MAXIMUM_BYTES: usize = 1024 * 1024;

fn identity(capability: &str) -> CapabilityCacheIdentity {
    CapabilityCacheIdentity::from_fingerprints(
        capability,
        ContentFingerprint::digest("implementation"),
        ContentFingerprint::digest("configuration"),
        [("src/App.vue", ContentFingerprint::digest("source"))],
    )
    .unwrap()
}

fn snapshot(title: &str) -> CapabilitySnapshot {
    let identity = identity("template-semantics");
    CapabilitySnapshot::try_new(
        identity,
        [DoctorFinding::new(
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
        )],
    )
    .unwrap()
}

fn envelope_object(snapshot: &CapabilitySnapshot) -> RemoteCapabilitySnapshotObject {
    RemoteCapabilitySnapshotObject::new(
        snapshot.cache_key(),
        DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE,
        PersistentCapabilitySnapshot::try_new(snapshot.clone())
            .unwrap()
            .to_json_bytes()
            .unwrap(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestStoreError {
    Load,
    Publish,
}

impl Display for TestStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load => formatter.write_str("injected load failure"),
            Self::Publish => formatter.write_str("injected publish failure"),
        }
    }
}

impl Error for TestStoreError {}

#[derive(Debug, Default)]
struct MemoryRemoteStore {
    objects: BTreeMap<CapabilityCacheKey, RemoteCapabilitySnapshotObject>,
    load_error: bool,
    publish_error: bool,
    load_maximum: Option<usize>,
    publish_maximum: Option<usize>,
}

impl MemoryRemoteStore {
    fn with_object(requested: CapabilityCacheKey, object: RemoteCapabilitySnapshotObject) -> Self {
        Self {
            objects: BTreeMap::from([(requested, object)]),
            ..Self::default()
        }
    }

    fn object(&self, cache_key: CapabilityCacheKey) -> &RemoteCapabilitySnapshotObject {
        &self.objects[&cache_key]
    }
}

impl RemoteCapabilitySnapshotStore for MemoryRemoteStore {
    type Error = TestStoreError;

    fn load_object(
        &mut self,
        cache_key: CapabilityCacheKey,
        maximum_response_bytes: usize,
    ) -> Result<Option<RemoteCapabilitySnapshotObject>, Self::Error> {
        self.load_maximum = Some(maximum_response_bytes);
        if self.load_error {
            return Err(TestStoreError::Load);
        }
        Ok(self.objects.get(&cache_key).cloned())
    }

    fn publish_object(
        &mut self,
        object: &RemoteCapabilitySnapshotObject,
        maximum_response_bytes: usize,
    ) -> Result<RemoteCapabilitySnapshotObject, Self::Error> {
        self.publish_maximum = Some(maximum_response_bytes);
        if self.publish_error {
            return Err(TestStoreError::Publish);
        }
        Ok(self
            .objects
            .entry(object.cache_key())
            .or_insert_with(|| object.clone())
            .clone())
    }
}

#[test]
fn publishes_exact_portable_envelope_and_reuses_it() {
    let expected = snapshot("Stable output");
    let cache_key = expected.cache_key();
    let expected_bytes = PersistentCapabilitySnapshot::try_new(expected.clone())
        .unwrap()
        .to_json_bytes()
        .unwrap();
    let mut writer =
        RemoteCapabilitySnapshotCache::new(MemoryRemoteStore::default(), MAXIMUM_BYTES);

    assert_eq!(writer.load_snapshot(cache_key).unwrap(), None);
    assert_eq!(writer.store_snapshot(&expected).unwrap(), expected);
    let store = writer.into_store();
    let object = store.object(cache_key);
    assert_eq!(object.cache_key(), cache_key);
    assert_eq!(
        object.media_type(),
        DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE
    );
    assert_eq!(object.bytes(), expected_bytes);
    assert_eq!(store.load_maximum, Some(MAXIMUM_BYTES));
    assert_eq!(store.publish_maximum, Some(MAXIMUM_BYTES));

    let mut reader = RemoteCapabilitySnapshotCache::new(store, MAXIMUM_BYTES);
    assert_eq!(reader.load_snapshot(cache_key).unwrap(), Some(expected));
}

#[test]
fn execution_reuses_remote_snapshot_across_cache_instances() {
    let requested = identity("template-semantics");
    let calls = Cell::new(0);
    let mut writer =
        RemoteCapabilitySnapshotCache::new(MemoryRemoteStore::default(), MAXIMUM_BYTES);
    let miss = execute_cached_capability(&mut writer, requested.clone(), |_| {
        calls.set(calls.get() + 1);
        Ok::<_, Infallible>(snapshot("Shared output").findings().to_vec())
    })
    .unwrap();
    assert!(miss.is_cache_miss());
    assert_eq!(calls.get(), 1);

    let mut reader = RemoteCapabilitySnapshotCache::new(writer.into_store(), MAXIMUM_BYTES);
    let hit = execute_cached_capability(&mut reader, requested, |_| {
        calls.set(calls.get() + 1);
        Ok::<_, Infallible>(Vec::new())
    })
    .unwrap();
    assert!(hit.is_cache_hit());
    assert_eq!(calls.get(), 1);
}

#[test]
fn rejects_mismatched_transport_metadata_before_decoding() {
    let expected = snapshot("Trusted output");
    let requested = expected.cache_key();
    let other_key = identity("script-semantics").cache_key();
    let wrong_key = RemoteCapabilitySnapshotObject::new(
        other_key,
        DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE,
        b"not-json".to_vec(),
    );
    let mut key_cache = RemoteCapabilitySnapshotCache::new(
        MemoryRemoteStore::with_object(requested, wrong_key),
        MAXIMUM_BYTES,
    );
    assert!(matches!(
        key_cache.load_snapshot(requested).unwrap_err(),
        RemoteCapabilitySnapshotCacheError::ObjectCacheKeyMismatch {
            requested: actual_requested,
            actual,
        } if actual_requested == requested && actual == other_key
    ));

    let wrong_media = RemoteCapabilitySnapshotObject::new(
        requested,
        "application/octet-stream",
        b"not-json".to_vec(),
    );
    let mut media_cache = RemoteCapabilitySnapshotCache::new(
        MemoryRemoteStore::with_object(requested, wrong_media),
        MAXIMUM_BYTES,
    );
    assert!(matches!(
        media_cache.load_snapshot(requested).unwrap_err(),
        RemoteCapabilitySnapshotCacheError::ObjectMediaTypeMismatch { actual }
            if actual == "application/octet-stream"
    ));
}

#[test]
fn rejects_oversized_corrupt_and_embedded_key_mismatched_objects() {
    let expected = snapshot("Trusted output");
    let requested = expected.cache_key();
    let oversized = RemoteCapabilitySnapshotObject::new(
        requested,
        DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE,
        vec![b'x'; 17],
    );
    let mut bounded = RemoteCapabilitySnapshotCache::new(
        MemoryRemoteStore::with_object(requested, oversized),
        16,
    );
    assert!(matches!(
        bounded.load_snapshot(requested).unwrap_err(),
        RemoteCapabilitySnapshotCacheError::Envelope {
            source: PersistentCapabilitySnapshotError::PayloadTooLarge {
                actual: 17,
                maximum: 16
            }
        }
    ));

    let corrupt = RemoteCapabilitySnapshotObject::new(
        requested,
        DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE,
        b"not-json".to_vec(),
    );
    let mut corrupt_cache = RemoteCapabilitySnapshotCache::new(
        MemoryRemoteStore::with_object(requested, corrupt),
        MAXIMUM_BYTES,
    );
    assert!(matches!(
        corrupt_cache.load_snapshot(requested).unwrap_err(),
        RemoteCapabilitySnapshotCacheError::Envelope {
            source: PersistentCapabilitySnapshotError::Deserialization { .. }
        }
    ));

    let other = CapabilitySnapshot::try_new(identity("script-semantics"), []).unwrap();
    let mut wrong_envelope = RemoteCapabilitySnapshotCache::new(
        MemoryRemoteStore::with_object(
            requested,
            RemoteCapabilitySnapshotObject::new(
                requested,
                DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE,
                envelope_object(&other).into_bytes(),
            ),
        ),
        MAXIMUM_BYTES,
    );
    assert!(matches!(
        wrong_envelope.load_snapshot(requested).unwrap_err(),
        RemoteCapabilitySnapshotCacheError::Envelope {
            source: PersistentCapabilitySnapshotError::RequestedCacheKeyMismatch {
                requested: actual_requested,
                actual,
            }
        } if actual_requested == requested && actual == other.cache_key()
    ));
}

#[test]
fn immutable_publication_is_idempotent_and_rejects_divergent_output() {
    let first = snapshot("First output");
    let second = snapshot("Second output");
    assert_eq!(first.cache_key(), second.cache_key());
    assert_ne!(first.output_fingerprint(), second.output_fingerprint());
    let mut cache = RemoteCapabilitySnapshotCache::new(MemoryRemoteStore::default(), MAXIMUM_BYTES);

    assert_eq!(cache.store_snapshot(&first).unwrap(), first);
    assert_eq!(cache.store_snapshot(&first).unwrap(), first);
    assert!(matches!(
        cache.store_snapshot(&second).unwrap_err(),
        RemoteCapabilitySnapshotCacheError::ConflictingSnapshot {
            cache_key,
            existing_output,
            incoming_output,
        } if cache_key == first.cache_key()
            && existing_output == first.output_fingerprint()
            && incoming_output == second.output_fingerprint()
    ));
}

#[test]
fn maps_provider_load_and_publish_failures() {
    let expected = snapshot("Trusted output");
    let mut load_cache = RemoteCapabilitySnapshotCache::new(
        MemoryRemoteStore {
            load_error: true,
            ..MemoryRemoteStore::default()
        },
        MAXIMUM_BYTES,
    );
    assert!(matches!(
        load_cache.load_snapshot(expected.cache_key()).unwrap_err(),
        RemoteCapabilitySnapshotCacheError::Load {
            source: TestStoreError::Load
        }
    ));

    let mut publish_cache = RemoteCapabilitySnapshotCache::new(
        MemoryRemoteStore {
            publish_error: true,
            ..MemoryRemoteStore::default()
        },
        MAXIMUM_BYTES,
    );
    assert!(matches!(
        publish_cache.store_snapshot(&expected).unwrap_err(),
        RemoteCapabilitySnapshotCacheError::Publish {
            source: TestStoreError::Publish
        }
    ));
}
