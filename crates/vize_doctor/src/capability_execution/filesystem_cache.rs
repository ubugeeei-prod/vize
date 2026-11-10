//! Atomic filesystem persistence for validated capability snapshots.

use std::{
    error::Error,
    ffi::OsString,
    fmt::{self, Display},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write as IoWrite},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use vize_l0::{String, append, cstr};

use crate::{
    CapabilityCacheKey, CapabilitySnapshot, ContentFingerprint, PersistentCapabilitySnapshot,
    PersistentCapabilitySnapshotError,
};

use super::CapabilitySnapshotCache;

/// Current on-disk directory layout for persistent capability snapshots.
pub const DOCTOR_FILESYSTEM_CAPABILITY_CACHE_LAYOUT_VERSION: &str = "v1";

const TEMPORARY_FILE_ATTEMPTS: usize = 64;
static NEXT_TEMPORARY_FILE_ID: AtomicU64 = AtomicU64::new(0);

/// Persistent local cache for validated capability snapshots.
///
/// Objects are stored below a caller-owned root using only the canonical cache
/// digest. The layout avoids characters that are invalid in Windows file names
/// and shards objects by the first digest byte. Stores write and synchronize a
/// unique temporary file before atomically linking it into the immutable final
/// location. Competing writers can therefore publish at most one object for a
/// cache key; later writers validate and acknowledge the winner.
///
/// Every load is bounded before JSON deserialization and validates the complete
/// [`PersistentCapabilitySnapshot`] envelope. The cache never treats malformed,
/// oversized, key-mismatched, or fingerprint-mismatched bytes as a miss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemCapabilitySnapshotCache {
    root: PathBuf,
    maximum_snapshot_bytes: usize,
}

impl FilesystemCapabilitySnapshotCache {
    /// Creates a filesystem cache below `root` with an explicit object ceiling.
    pub fn new(root: impl Into<PathBuf>, maximum_snapshot_bytes: usize) -> Self {
        Self {
            root: root.into(),
            maximum_snapshot_bytes,
        }
    }

    /// Returns the caller-owned cache root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the maximum accepted bytes for one persistent object.
    pub const fn maximum_snapshot_bytes(&self) -> usize {
        self.maximum_snapshot_bytes
    }

    fn object_path(&self, cache_key: CapabilityCacheKey) -> PathBuf {
        let digest = cache_key_digest(cache_key);
        let shard = cstr!("{:02x}", cache_key.fingerprint().as_bytes()[0]);
        let file_name = cstr!("{digest}.json");
        self.root
            .join(DOCTOR_FILESYSTEM_CAPABILITY_CACHE_LAYOUT_VERSION)
            .join(shard.as_str())
            .join(file_name.as_str())
    }

    fn load_object(
        &self,
        cache_key: CapabilityCacheKey,
        object_path: &Path,
    ) -> Result<Option<CapabilitySnapshot>, FilesystemCapabilitySnapshotCacheError> {
        let file = match File::open(object_path) {
            Ok(file) => file,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(FilesystemCapabilitySnapshotCacheError::Io {
                    operation: "open cache object",
                    path: object_path.to_path_buf(),
                    source,
                });
            }
        };
        let bytes = read_bounded(file, object_path, self.maximum_snapshot_bytes)?;
        let envelope = PersistentCapabilitySnapshot::from_json_slice(
            cache_key,
            &bytes,
            self.maximum_snapshot_bytes,
        )
        .map_err(|source| FilesystemCapabilitySnapshotCacheError::Envelope { source })?;
        Ok(Some(envelope.into_snapshot()))
    }

    fn publish_object(
        &self,
        object_path: &Path,
        bytes: &[u8],
    ) -> Result<(), FilesystemCapabilitySnapshotCacheError> {
        let Some(parent) = object_path.parent() else {
            return Err(FilesystemCapabilitySnapshotCacheError::Io {
                operation: "resolve cache object directory",
                path: object_path.to_path_buf(),
                source: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "cache object path has no parent",
                ),
            });
        };
        fs::create_dir_all(parent).map_err(|source| {
            FilesystemCapabilitySnapshotCacheError::Io {
                operation: "create cache directory",
                path: parent.to_path_buf(),
                source,
            }
        })?;

        let (temporary_path, mut temporary_file) = create_temporary_file(parent)?;
        let write_result = (|| {
            temporary_file.write_all(bytes).map_err(|source| {
                FilesystemCapabilitySnapshotCacheError::Io {
                    operation: "write temporary cache object",
                    path: temporary_path.clone(),
                    source,
                }
            })?;
            temporary_file.sync_all().map_err(|source| {
                FilesystemCapabilitySnapshotCacheError::Io {
                    operation: "synchronize temporary cache object",
                    path: temporary_path.clone(),
                    source,
                }
            })?;
            Ok(())
        })();
        drop(temporary_file);
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temporary_path);
            return Err(error);
        }

        let publish_result = fs::hard_link(&temporary_path, object_path);
        let cleanup_result = fs::remove_file(&temporary_path);
        match publish_result {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => {
                return Err(FilesystemCapabilitySnapshotCacheError::Io {
                    operation: "publish cache object",
                    path: object_path.to_path_buf(),
                    source,
                });
            }
        }
        cleanup_result.map_err(|source| FilesystemCapabilitySnapshotCacheError::Io {
            operation: "remove temporary cache object",
            path: temporary_path,
            source,
        })
    }
}

impl CapabilitySnapshotCache for FilesystemCapabilitySnapshotCache {
    type Error = FilesystemCapabilitySnapshotCacheError;

    fn load_snapshot(
        &mut self,
        cache_key: CapabilityCacheKey,
    ) -> Result<Option<CapabilitySnapshot>, Self::Error> {
        self.load_object(cache_key, &self.object_path(cache_key))
    }

    fn store_snapshot(
        &mut self,
        snapshot: &CapabilitySnapshot,
    ) -> Result<CapabilitySnapshot, Self::Error> {
        let envelope = PersistentCapabilitySnapshot::try_new(snapshot.clone())
            .map_err(|source| FilesystemCapabilitySnapshotCacheError::Envelope { source })?;
        let bytes = envelope
            .to_json_bytes()
            .map_err(|source| FilesystemCapabilitySnapshotCacheError::Envelope { source })?;
        if bytes.len() > self.maximum_snapshot_bytes {
            return Err(FilesystemCapabilitySnapshotCacheError::Envelope {
                source: PersistentCapabilitySnapshotError::PayloadTooLarge {
                    actual: bytes.len(),
                    maximum: self.maximum_snapshot_bytes,
                },
            });
        }

        let cache_key = snapshot.cache_key();
        let object_path = self.object_path(cache_key);
        self.publish_object(&object_path, &bytes)?;
        let stored = self.load_object(cache_key, &object_path)?.ok_or_else(|| {
            FilesystemCapabilitySnapshotCacheError::PublishedObjectMissing {
                path: object_path.clone(),
            }
        })?;
        if stored != *snapshot {
            return Err(
                FilesystemCapabilitySnapshotCacheError::ConflictingSnapshot {
                    cache_key,
                    existing_output: stored.output_fingerprint(),
                    incoming_output: snapshot.output_fingerprint(),
                },
            );
        }
        Ok(stored)
    }
}

/// Failure while reading or publishing a filesystem capability snapshot.
#[derive(Debug)]
#[non_exhaustive]
pub enum FilesystemCapabilitySnapshotCacheError {
    /// A filesystem operation failed.
    Io {
        /// Stable description of the failed operation.
        operation: &'static str,
        /// Path involved in the operation.
        path: PathBuf,
        /// Underlying operating-system error.
        source: io::Error,
    },
    /// Persistent envelope encoding or validation failed.
    Envelope {
        /// Fail-closed envelope error.
        source: PersistentCapabilitySnapshotError,
    },
    /// A successfully published object disappeared before acknowledgement.
    PublishedObjectMissing {
        /// Expected final object path.
        path: PathBuf,
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

impl Display for FilesystemCapabilitySnapshotCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} at {}: {source}",
                path.display()
            ),
            Self::Envelope { source } => source.fmt(formatter),
            Self::PublishedObjectMissing { path } => write!(
                formatter,
                "published capability snapshot disappeared from {}",
                path.display()
            ),
            Self::ConflictingSnapshot {
                cache_key,
                existing_output,
                incoming_output,
            } => write!(
                formatter,
                "filesystem capability snapshot key {cache_key} already stores output {existing_output}; rejected {incoming_output}"
            ),
        }
    }
}

impl Error for FilesystemCapabilitySnapshotCacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Envelope { source } => Some(source),
            Self::PublishedObjectMissing { .. } | Self::ConflictingSnapshot { .. } => None,
        }
    }
}

fn read_bounded(
    file: File,
    path: &Path,
    maximum_bytes: usize,
) -> Result<Vec<u8>, FilesystemCapabilitySnapshotCacheError> {
    let limit = u64::try_from(maximum_bytes)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let mut bytes = Vec::new();
    file.take(limit).read_to_end(&mut bytes).map_err(|source| {
        FilesystemCapabilitySnapshotCacheError::Io {
            operation: "read cache object",
            path: path.to_path_buf(),
            source,
        }
    })?;
    if bytes.len() > maximum_bytes {
        return Err(FilesystemCapabilitySnapshotCacheError::Envelope {
            source: PersistentCapabilitySnapshotError::PayloadTooLarge {
                actual: bytes.len(),
                maximum: maximum_bytes,
            },
        });
    }
    Ok(bytes)
}

fn create_temporary_file(
    parent: &Path,
) -> Result<(PathBuf, File), FilesystemCapabilitySnapshotCacheError> {
    let mut last_collision = None;
    for _ in 0..TEMPORARY_FILE_ATTEMPTS {
        let id = NEXT_TEMPORARY_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = OsString::from(".vize-doctor-cache-");
        temporary_name.push(cstr!("{}-{id}.tmp", std::process::id()).as_str());
        let temporary_path = parent.join(temporary_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => return Ok((temporary_path, file)),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                last_collision = Some(source);
            }
            Err(source) => {
                return Err(FilesystemCapabilitySnapshotCacheError::Io {
                    operation: "create temporary cache object",
                    path: temporary_path,
                    source,
                });
            }
        }
    }

    Err(FilesystemCapabilitySnapshotCacheError::Io {
        operation: "create unique temporary cache object",
        path: parent.to_path_buf(),
        source: last_collision.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                "temporary cache object collision limit reached",
            )
        }),
    })
}

fn cache_key_digest(cache_key: CapabilityCacheKey) -> String {
    let mut digest = String::with_capacity(64);
    for byte in cache_key.fingerprint().as_bytes() {
        append!(digest, "{byte:02x}");
    }
    digest
}

#[cfg(test)]
mod tests {
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
                    .with_invalidation_fingerprints(BTreeMap::from([(
                        "src/App.vue".into(),
                        source,
                    )])),
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
}
