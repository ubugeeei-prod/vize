//! Store-neutral wire envelope for persistent local and remote caches.

use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vize_l0::{String, ToCompactString};

use crate::{CapabilityCacheKey, ContentFingerprint};

use super::CapabilitySnapshot;

/// Current persistent capability-snapshot envelope format.
pub const DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_FORMAT_VERSION: u32 = 1;

/// Media type shared by local files and remote cache objects.
pub const DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE: &str =
    "application/vnd.vize.doctor.capability-snapshot+json;version=1";

const PAYLOAD_FINGERPRINT_DOMAIN: &[u8] = b"vize-doctor\0persistent-capability-snapshot\0json-v1\0";

/// Portable, integrity-checked representation of one capability snapshot.
///
/// The envelope contains no filesystem paths, timestamps, host metadata, or
/// backend-specific fields. A local cache file and a remote object therefore
/// use identical bytes for the same [`CapabilitySnapshot`]. Decoding is
/// deliberately explicit: callers supply the requested cache key and a byte
/// ceiling, and the nested snapshot runs its complete identity, provenance,
/// and output-fingerprint validation before this type is returned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PersistentCapabilitySnapshot {
    format_version: u32,
    media_type: String,
    cache_key: CapabilityCacheKey,
    payload_fingerprint: ContentFingerprint,
    snapshot: CapabilitySnapshot,
}

impl PersistentCapabilitySnapshot {
    /// Wraps a validated snapshot in the current persistent wire contract.
    pub fn try_new(
        snapshot: CapabilitySnapshot,
    ) -> Result<Self, PersistentCapabilitySnapshotError> {
        let cache_key = snapshot.cache_key();
        let payload_fingerprint = fingerprint_snapshot(&snapshot)?;
        Ok(Self {
            format_version: DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_FORMAT_VERSION,
            media_type: DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE.into(),
            cache_key,
            payload_fingerprint,
            snapshot,
        })
    }

    /// Returns the persistent envelope format version.
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Returns the exact media type expected by persistent cache transports.
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Returns the object key under which the envelope must be stored.
    pub const fn cache_key(&self) -> CapabilityCacheKey {
        self.cache_key
    }

    /// Returns the canonical identity of the complete nested snapshot payload.
    pub const fn payload_fingerprint(&self) -> ContentFingerprint {
        self.payload_fingerprint
    }

    /// Returns the validated capability snapshot.
    pub const fn snapshot(&self) -> &CapabilitySnapshot {
        &self.snapshot
    }

    /// Consumes the envelope and returns the validated capability snapshot.
    pub fn into_snapshot(self) -> CapabilitySnapshot {
        self.snapshot
    }

    /// Serializes deterministic JSON bytes for a local file or remote object.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, PersistentCapabilitySnapshotError> {
        serde_json::to_vec(self).map_err(|error| PersistentCapabilitySnapshotError::Serialization {
            message: error.to_compact_string(),
        })
    }

    /// Decodes and validates an untrusted persistent cache object.
    ///
    /// `maximum_bytes` is checked before JSON parsing so a backend cannot make
    /// the Doctor allocate or deserialize an unexpectedly large object. The
    /// requested cache key is compared independently with both the envelope
    /// and its validated nested snapshot.
    pub fn from_json_slice(
        requested_cache_key: CapabilityCacheKey,
        bytes: &[u8],
        maximum_bytes: usize,
    ) -> Result<Self, PersistentCapabilitySnapshotError> {
        if bytes.len() > maximum_bytes {
            return Err(PersistentCapabilitySnapshotError::PayloadTooLarge {
                actual: bytes.len(),
                maximum: maximum_bytes,
            });
        }

        let wire =
            serde_json::from_slice::<PersistentCapabilitySnapshotWire>(bytes).map_err(|error| {
                PersistentCapabilitySnapshotError::Deserialization {
                    message: error.to_compact_string(),
                }
            })?;
        if wire.format_version != DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_FORMAT_VERSION {
            return Err(
                PersistentCapabilitySnapshotError::UnsupportedFormatVersion {
                    actual: wire.format_version,
                },
            );
        }
        if wire.media_type != DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE {
            return Err(PersistentCapabilitySnapshotError::UnsupportedMediaType {
                actual: wire.media_type,
            });
        }

        let snapshot_cache_key = wire.snapshot.cache_key();
        if wire.cache_key != snapshot_cache_key {
            return Err(
                PersistentCapabilitySnapshotError::DeclaredCacheKeyMismatch {
                    declared: wire.cache_key,
                    snapshot: snapshot_cache_key,
                },
            );
        }
        if wire.cache_key != requested_cache_key {
            return Err(
                PersistentCapabilitySnapshotError::RequestedCacheKeyMismatch {
                    requested: requested_cache_key,
                    actual: wire.cache_key,
                },
            );
        }

        let payload_fingerprint = fingerprint_snapshot(&wire.snapshot)?;
        if wire.payload_fingerprint != payload_fingerprint {
            return Err(
                PersistentCapabilitySnapshotError::PayloadFingerprintMismatch {
                    declared: wire.payload_fingerprint,
                    derived: payload_fingerprint,
                },
            );
        }

        Ok(Self {
            format_version: wire.format_version,
            media_type: wire.media_type,
            cache_key: wire.cache_key,
            payload_fingerprint,
            snapshot: wire.snapshot,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistentCapabilitySnapshotWire {
    format_version: u32,
    media_type: String,
    cache_key: CapabilityCacheKey,
    payload_fingerprint: ContentFingerprint,
    snapshot: CapabilitySnapshot,
}

/// An untrusted persistent cache object that cannot be reused safely.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PersistentCapabilitySnapshotError {
    /// The object exceeds the caller's pre-deserialization byte ceiling.
    PayloadTooLarge {
        /// Object size supplied by the cache backend.
        actual: usize,
        /// Maximum accepted object size.
        maximum: usize,
    },
    /// JSON encoding of a trusted snapshot failed.
    Serialization {
        /// Actionable serializer message.
        message: String,
    },
    /// The object is not valid envelope JSON or contains an invalid snapshot.
    Deserialization {
        /// Actionable parser or nested validation message.
        message: String,
    },
    /// The persistent envelope version is not supported.
    UnsupportedFormatVersion {
        /// Version found in the object.
        actual: u32,
    },
    /// The transport media type does not name the current contract.
    UnsupportedMediaType {
        /// Media type found in the object.
        actual: String,
    },
    /// The envelope key disagrees with the nested snapshot identity.
    DeclaredCacheKeyMismatch {
        /// Key declared by the envelope.
        declared: CapabilityCacheKey,
        /// Key derived from the nested snapshot.
        snapshot: CapabilityCacheKey,
    },
    /// The object was returned for a different requested cache key.
    RequestedCacheKeyMismatch {
        /// Key requested from storage.
        requested: CapabilityCacheKey,
        /// Key declared by the returned object.
        actual: CapabilityCacheKey,
    },
    /// The complete nested snapshot does not match its declared fingerprint.
    PayloadFingerprintMismatch {
        /// Fingerprint declared by the envelope.
        declared: ContentFingerprint,
        /// Fingerprint derived from the validated nested snapshot.
        derived: ContentFingerprint,
    },
}

impl fmt::Display for PersistentCapabilitySnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLarge { actual, maximum } => write!(
                formatter,
                "persistent capability snapshot is {actual} bytes; maximum is {maximum}"
            ),
            Self::Serialization { message } => write!(
                formatter,
                "persistent capability snapshot serialization failed: {message}"
            ),
            Self::Deserialization { message } => write!(
                formatter,
                "persistent capability snapshot deserialization failed: {message}"
            ),
            Self::UnsupportedFormatVersion { actual } => write!(
                formatter,
                "unsupported persistent capability snapshot version {actual}; expected {DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_FORMAT_VERSION}"
            ),
            Self::UnsupportedMediaType { actual } => write!(
                formatter,
                "unsupported persistent capability snapshot media type {actual:?}"
            ),
            Self::DeclaredCacheKeyMismatch { declared, snapshot } => write!(
                formatter,
                "persistent capability snapshot key {declared} does not match nested snapshot key {snapshot}"
            ),
            Self::RequestedCacheKeyMismatch { requested, actual } => write!(
                formatter,
                "persistent cache returned snapshot key {actual}; requested {requested}"
            ),
            Self::PayloadFingerprintMismatch { declared, derived } => write!(
                formatter,
                "persistent capability snapshot fingerprint {declared} does not match derived fingerprint {derived}"
            ),
        }
    }
}

impl Error for PersistentCapabilitySnapshotError {}

fn fingerprint_snapshot(
    snapshot: &CapabilitySnapshot,
) -> Result<ContentFingerprint, PersistentCapabilitySnapshotError> {
    let mut writer = DigestWriter(Sha256::new());
    writer.0.update(PAYLOAD_FINGERPRINT_DOMAIN);
    writer
        .0
        .update(DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_FORMAT_VERSION.to_le_bytes());
    serde_json::to_writer(&mut writer, snapshot).map_err(|error| {
        PersistentCapabilitySnapshotError::Serialization {
            message: error.to_compact_string(),
        }
    })?;
    Ok(ContentFingerprint::from_digest(writer.0.finalize().into()))
}

struct DigestWriter(Sha256);

impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{
        DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_FORMAT_VERSION,
        DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE, PersistentCapabilitySnapshot,
        PersistentCapabilitySnapshotError,
    };
    use crate::{
        AnalysisProvenance, CapabilityCacheIdentity, CapabilitySnapshot, ContentFingerprint,
        DoctorCategory, DoctorFinding, FindingAssessment, FindingConfidence, FindingImpact,
        FindingSeverity, HealthPenalty, RuleCost, SourceLocation,
    };

    fn identity(capability: &str, source: &str) -> CapabilityCacheIdentity {
        CapabilityCacheIdentity::from_fingerprints(
            capability,
            ContentFingerprint::digest("implementation"),
            ContentFingerprint::digest("configuration"),
            [("src/App.vue", ContentFingerprint::digest(source))],
        )
        .unwrap()
    }

    fn snapshot() -> CapabilitySnapshot {
        let source = ContentFingerprint::digest("source");
        CapabilitySnapshot::try_new(
            identity("template-semantics", "source"),
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
                "Test finding",
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
    fn identical_snapshots_produce_portable_deterministic_bytes() {
        let snapshot = snapshot();
        let envelope = PersistentCapabilitySnapshot::try_new(snapshot.clone()).unwrap();
        let first = envelope.to_json_bytes().unwrap();
        let second = PersistentCapabilitySnapshot::try_new(snapshot.clone())
            .unwrap()
            .to_json_bytes()
            .unwrap();

        assert_eq!(first, second);
        assert_eq!(envelope.format_version(), 1);
        assert_eq!(
            envelope.media_type(),
            DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE
        );
        assert_eq!(envelope.cache_key(), snapshot.cache_key());
        assert_eq!(envelope.snapshot(), &snapshot);

        let decoded = PersistentCapabilitySnapshot::from_json_slice(
            snapshot.cache_key(),
            &first,
            first.len(),
        )
        .unwrap();
        assert_eq!(decoded, envelope);
        assert_eq!(decoded.into_snapshot(), snapshot);
    }

    #[test]
    fn decoder_rejects_transport_identity_and_integrity_mismatches() {
        let snapshot = snapshot();
        let envelope = PersistentCapabilitySnapshot::try_new(snapshot.clone()).unwrap();
        let bytes = envelope.to_json_bytes().unwrap();

        assert!(matches!(
            PersistentCapabilitySnapshot::from_json_slice(
                snapshot.cache_key(),
                &bytes,
                bytes.len() - 1,
            ),
            Err(PersistentCapabilitySnapshotError::PayloadTooLarge { .. })
        ));

        let other_key = identity("type-semantics", "source").cache_key();
        assert!(matches!(
            PersistentCapabilitySnapshot::from_json_slice(other_key, &bytes, bytes.len()),
            Err(PersistentCapabilitySnapshotError::RequestedCacheKeyMismatch { .. })
        ));

        let mut value = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap();
        value["formatVersion"] = json!(DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_FORMAT_VERSION + 1);
        let changed = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            PersistentCapabilitySnapshot::from_json_slice(
                snapshot.cache_key(),
                &changed,
                changed.len(),
            ),
            Err(PersistentCapabilitySnapshotError::UnsupportedFormatVersion { .. })
        ));

        value["formatVersion"] = json!(DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_FORMAT_VERSION);
        value["mediaType"] = json!("application/json");
        let changed = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            PersistentCapabilitySnapshot::from_json_slice(
                snapshot.cache_key(),
                &changed,
                changed.len(),
            ),
            Err(PersistentCapabilitySnapshotError::UnsupportedMediaType { .. })
        ));

        value["mediaType"] = json!(DOCTOR_PERSISTENT_CAPABILITY_SNAPSHOT_MEDIA_TYPE);
        value["cacheKey"] = serde_json::to_value(other_key).unwrap();
        let changed = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            PersistentCapabilitySnapshot::from_json_slice(
                snapshot.cache_key(),
                &changed,
                changed.len(),
            ),
            Err(PersistentCapabilitySnapshotError::DeclaredCacheKeyMismatch { .. })
        ));

        value["cacheKey"] = serde_json::to_value(snapshot.cache_key()).unwrap();
        value["payloadFingerprint"] =
            serde_json::to_value(ContentFingerprint::digest("tampered")).unwrap();
        let changed = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            PersistentCapabilitySnapshot::from_json_slice(
                snapshot.cache_key(),
                &changed,
                changed.len(),
            ),
            Err(PersistentCapabilitySnapshotError::PayloadFingerprintMismatch { .. })
        ));
    }

    #[test]
    fn decoder_rejects_unknown_fields_and_invalid_nested_snapshots() {
        let snapshot = snapshot();
        let envelope = PersistentCapabilitySnapshot::try_new(snapshot.clone()).unwrap();
        let mut value = serde_json::to_value(envelope).unwrap();

        value["unexpected"] = json!(true);
        let changed = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            PersistentCapabilitySnapshot::from_json_slice(
                snapshot.cache_key(),
                &changed,
                changed.len(),
            ),
            Err(PersistentCapabilitySnapshotError::Deserialization { .. })
        ));

        value.as_object_mut().unwrap().remove("unexpected");
        value["snapshot"]["findings"][0]["message"] = json!("tampered");
        let changed = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            PersistentCapabilitySnapshot::from_json_slice(
                snapshot.cache_key(),
                &changed,
                changed.len(),
            ),
            Err(PersistentCapabilitySnapshotError::Deserialization { .. })
        ));
    }
}
