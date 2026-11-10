use std::{collections::BTreeMap, fs, thread};

use tempfile::tempdir;

use super::{
    DOCTOR_FILESYSTEM_CAPABILITY_CACHE_LAYOUT_VERSION, FilesystemCapabilitySnapshotCache,
    FilesystemCapabilitySnapshotCacheError,
};
use crate::{
    AnalysisProvenance, CapabilityCacheIdentity, CapabilitySnapshot, CapabilitySnapshotCache,
    ContentFingerprint, DoctorCategory, DoctorFinding, FindingAssessment, FindingConfidence,
    FindingImpact, FindingSeverity, HealthPenalty, PersistentCapabilitySnapshotError, RuleCost,
    SourceLocation,
};

const MAXIMUM_BYTES: usize = 1024 * 1024;

fn identity(source: ContentFingerprint) -> CapabilityCacheIdentity {
    CapabilityCacheIdentity::from_fingerprints(
        "template-semantics",
        ContentFingerprint::digest("implementation"),
        ContentFingerprint::digest("configuration"),
        [("src/App.vue", source)],
    )
    .unwrap()
}

fn snapshot(title: &str) -> CapabilitySnapshot {
    let source = ContentFingerprint::digest("source");
    CapabilitySnapshot::try_new(
        identity(source),
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
                .with_invalidation_fingerprints(BTreeMap::from([("src/App.vue".into(), source)])),
        )],
    )
    .unwrap()
}

#[test]
fn persists_portable_bytes_and_reuses_them_across_instances() {
    let directory = tempdir().unwrap();
    let expected = snapshot("Stable output");
    let cache_key = expected.cache_key();
    let mut writer = FilesystemCapabilitySnapshotCache::new(directory.path(), MAXIMUM_BYTES);

    assert_eq!(writer.load_snapshot(cache_key).unwrap(), None);
    assert_eq!(writer.store_snapshot(&expected).unwrap(), expected);

    let object_path = writer.object_path(cache_key);
    let digest = super::cache_key_digest(cache_key);
    let shard = vize_l0::cstr!("{:02x}", cache_key.fingerprint().as_bytes()[0]);
    let file_name = vize_l0::cstr!("{digest}.json");
    assert_eq!(
        object_path,
        directory
            .path()
            .join(DOCTOR_FILESYSTEM_CAPABILITY_CACHE_LAYOUT_VERSION)
            .join(shard.as_str())
            .join(file_name.as_str())
    );
    let expected_bytes = crate::PersistentCapabilitySnapshot::try_new(expected.clone())
        .unwrap()
        .to_json_bytes()
        .unwrap();
    assert_eq!(fs::read(object_path).unwrap(), expected_bytes);

    let mut reader = FilesystemCapabilitySnapshotCache::new(directory.path(), MAXIMUM_BYTES);
    assert_eq!(reader.load_snapshot(cache_key).unwrap(), Some(expected));
}

#[test]
fn rejects_oversized_corrupt_and_key_mismatched_objects() {
    let directory = tempdir().unwrap();
    let expected = snapshot("Trusted output");
    let cache_key = expected.cache_key();
    let mut cache = FilesystemCapabilitySnapshotCache::new(directory.path(), MAXIMUM_BYTES);
    cache.store_snapshot(&expected).unwrap();
    let object_path = cache.object_path(cache_key);

    fs::write(&object_path, b"not-json").unwrap();
    assert!(matches!(
        cache.load_snapshot(cache_key).unwrap_err(),
        FilesystemCapabilitySnapshotCacheError::Envelope {
            source: PersistentCapabilitySnapshotError::Deserialization { .. }
        }
    ));

    fs::write(&object_path, vec![b'x'; 17]).unwrap();
    let mut bounded = FilesystemCapabilitySnapshotCache::new(directory.path(), 16);
    assert!(matches!(
        bounded.load_snapshot(cache_key).unwrap_err(),
        FilesystemCapabilitySnapshotCacheError::Envelope {
            source: PersistentCapabilitySnapshotError::PayloadTooLarge {
                actual: 17,
                maximum: 16
            }
        }
    ));

    let other = CapabilitySnapshot::try_new(
        CapabilityCacheIdentity::from_fingerprints(
            "script-semantics",
            ContentFingerprint::digest("implementation"),
            ContentFingerprint::digest("configuration"),
            [("src/App.vue", ContentFingerprint::digest("source"))],
        )
        .unwrap(),
        [],
    )
    .unwrap();
    let other_bytes = crate::PersistentCapabilitySnapshot::try_new(other.clone())
        .unwrap()
        .to_json_bytes()
        .unwrap();
    fs::write(&object_path, other_bytes).unwrap();
    assert!(matches!(
        cache.load_snapshot(cache_key).unwrap_err(),
        FilesystemCapabilitySnapshotCacheError::Envelope {
            source: PersistentCapabilitySnapshotError::RequestedCacheKeyMismatch {
                requested,
                actual
            }
        } if requested == cache_key && actual == other.cache_key()
    ));
}

#[test]
fn concurrent_writers_publish_one_immutable_snapshot() {
    let directory = tempdir().unwrap();
    let first = snapshot("First output");
    let second = snapshot("Second output");
    assert_eq!(first.cache_key(), second.cache_key());
    assert_ne!(first.output_fingerprint(), second.output_fingerprint());

    let results = thread::scope(|scope| {
        let root = directory.path();
        let handles = [first.clone(), second.clone()]
            .into_iter()
            .map(|candidate| {
                scope.spawn(move || {
                    let mut cache = FilesystemCapabilitySnapshotCache::new(root, MAXIMUM_BYTES);
                    cache.store_snapshot(&candidate)
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(
                result,
                Err(FilesystemCapabilitySnapshotCacheError::ConflictingSnapshot { .. })
            ))
            .count(),
        1
    );
    let winner = results.into_iter().find_map(Result::ok).unwrap();
    let mut reader = FilesystemCapabilitySnapshotCache::new(directory.path(), MAXIMUM_BYTES);
    assert_eq!(
        reader.load_snapshot(first.cache_key()).unwrap(),
        Some(winner)
    );

    let object_path = reader.object_path(first.cache_key());
    let expected_file_name = object_path.file_name().unwrap().to_owned();
    let parent = object_path.parent().unwrap();
    let entries = fs::read_dir(parent)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(entries, [expected_file_name]);
}

#[test]
fn identical_restores_are_idempotent() {
    let directory = tempdir().unwrap();
    let expected = snapshot("Stable output");
    let mut cache = FilesystemCapabilitySnapshotCache::new(directory.path(), MAXIMUM_BYTES);

    assert_eq!(cache.store_snapshot(&expected).unwrap(), expected);
    assert_eq!(cache.store_snapshot(&expected).unwrap(), expected);
}
