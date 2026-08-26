use std::{error::Error, fmt};

use vize_s0::String;

use crate::{CapabilityCacheKey, ContentFingerprint};

use super::DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION;

/// A capability snapshot that cannot be trusted for cache reuse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CapabilitySnapshotError {
    /// The serialized snapshot version is not supported.
    UnsupportedFormatVersion {
        /// Version found in the payload.
        actual: u32,
    },
    /// The serialized cache key does not match the embedded identity.
    CacheKeyMismatch {
        /// Key declared by the payload.
        declared: CapabilityCacheKey,
        /// Key derived from the embedded identity.
        derived: CapabilityCacheKey,
    },
    /// The serialized findings do not match their declared content identity.
    OutputFingerprintMismatch {
        /// Finding fingerprint declared by the payload.
        declared: ContentFingerprint,
        /// Fingerprint derived from normalized findings.
        derived: ContentFingerprint,
    },
    /// Canonical finding serialization could not be completed.
    ///
    /// The failure is kept as a formatted message rather than the originating
    /// `serde_json::Error`, which is neither `Clone`, `PartialEq`, nor `Eq` and
    /// so cannot be stored in this enum.
    OutputSerialization {
        /// Actionable serialization failure.
        message: String,
    },
    /// A finding was emitted by a different capability.
    CapabilityMismatch {
        /// Stable finding code.
        finding_code: String,
        /// Capability named by the snapshot identity.
        expected: String,
        /// Capability named by the finding provenance.
        actual: String,
    },
    /// A finding declares an invalidation input without an exact fingerprint.
    MissingFingerprint {
        /// Stable finding code.
        finding_code: String,
        /// Unfingerprinted logical input.
        input: String,
    },
    /// A finding depends on an input absent from the capability identity.
    UndeclaredIdentityInput {
        /// Stable finding code.
        finding_code: String,
        /// Input missing from the identity.
        input: String,
    },
    /// A finding and its capability identity disagree about exact input content.
    FingerprintMismatch {
        /// Stable finding code.
        finding_code: String,
        /// Logical input with conflicting content identities.
        input: String,
        /// Fingerprint declared by the capability identity.
        expected: ContentFingerprint,
        /// Fingerprint attached to the finding provenance.
        actual: ContentFingerprint,
    },
    /// A finding contains a fingerprint for an undeclared provenance input.
    OrphanFingerprint {
        /// Stable finding code.
        finding_code: String,
        /// Fingerprinted but undeclared input.
        input: String,
    },
}

impl fmt::Display for CapabilitySnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormatVersion { actual } => write!(
                formatter,
                "unsupported capability snapshot version {actual}; expected {DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION}"
            ),
            Self::CacheKeyMismatch { declared, derived } => write!(
                formatter,
                "capability snapshot cache key {declared} does not match derived key {derived}"
            ),
            Self::OutputFingerprintMismatch { declared, derived } => write!(
                formatter,
                "capability snapshot output fingerprint {declared} does not match derived fingerprint {derived}"
            ),
            Self::OutputSerialization { message } => {
                write!(
                    formatter,
                    "capability snapshot output could not be fingerprinted: {message}"
                )
            }
            Self::CapabilityMismatch {
                finding_code,
                expected,
                actual,
            } => write!(
                formatter,
                "finding {finding_code} names capability {actual:?}; expected {expected:?}"
            ),
            Self::MissingFingerprint {
                finding_code,
                input,
            } => write!(
                formatter,
                "finding {finding_code} has no fingerprint for invalidation input {input:?}"
            ),
            Self::UndeclaredIdentityInput {
                finding_code,
                input,
            } => write!(
                formatter,
                "finding {finding_code} input {input:?} is absent from the capability identity"
            ),
            Self::FingerprintMismatch {
                finding_code,
                input,
                expected,
                actual,
            } => write!(
                formatter,
                "finding {finding_code} input {input:?} has fingerprint {actual}; expected {expected}"
            ),
            Self::OrphanFingerprint {
                finding_code,
                input,
            } => write!(
                formatter,
                "finding {finding_code} has an orphan fingerprint for {input:?}"
            ),
        }
    }
}

impl Error for CapabilitySnapshotError {}
