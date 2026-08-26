//! Validated, independently reusable output from one analysis capability.

mod error;
#[cfg(test)]
mod tests;

use std::io::{self, Write};

use serde::{Deserialize, Deserializer, Serialize, de};
use sha2::{Digest, Sha256};
use vize_s0::{String, ToCompactString};

use crate::{
    CapabilityCacheIdentity, CapabilityCacheKey, ContentFingerprint, DoctorFinding, DoctorReport,
    report::normalize_findings,
};

pub use error::CapabilitySnapshotError;

/// Current serialized capability snapshot format.
pub const DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION: u32 = 1;

const OUTPUT_FINGERPRINT_DOMAIN: &[u8] = b"vize-doctor\0capability-output\0json-v1\0";

/// Cacheable findings produced by one exact analysis capability identity.
///
/// Every finding must name the same capability and carry an exact fingerprint
/// for every declared invalidation input. Those fingerprints must be present
/// and equal in [`Self::identity`]. The identity may contain additional inputs
/// that form the capability's complete discovery boundary.
///
/// This fail-closed relationship prevents a remote or in-memory cache from
/// reusing findings whose reported provenance is broader than their key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilitySnapshot {
    format_version: u32,
    cache_key: CapabilityCacheKey,
    output_fingerprint: ContentFingerprint,
    identity: CapabilityCacheIdentity,
    findings: Vec<DoctorFinding>,
}

impl CapabilitySnapshot {
    /// Creates a validated snapshot and deterministically normalizes findings.
    pub fn try_new(
        identity: CapabilityCacheIdentity,
        findings: impl IntoIterator<Item = DoctorFinding>,
    ) -> Result<Self, CapabilitySnapshotError> {
        let findings = findings.into_iter().collect::<Vec<_>>();
        validate_findings(&identity, &findings)?;
        let findings = normalize_findings(findings);
        let cache_key = identity.cache_key();
        let output_fingerprint = fingerprint_findings(&findings)?;
        Ok(Self {
            format_version: DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION,
            cache_key,
            output_fingerprint,
            identity,
            findings,
        })
    }

    /// Returns the capability snapshot wire version.
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    /// Returns the derived key under which this snapshot may be stored.
    pub const fn cache_key(&self) -> CapabilityCacheKey {
        self.cache_key
    }

    /// Returns the domain-separated identity of the normalized findings.
    pub const fn output_fingerprint(&self) -> ContentFingerprint {
        self.output_fingerprint
    }

    /// Returns the complete capability identity and discovery boundary.
    pub const fn identity(&self) -> &CapabilityCacheIdentity {
        &self.identity
    }

    /// Returns findings in deterministic Doctor report order.
    pub fn findings(&self) -> &[DoctorFinding] {
        &self.findings
    }

    /// Consumes the snapshot and returns its findings.
    pub fn into_findings(self) -> Vec<DoctorFinding> {
        self.findings
    }

    /// Consumes the snapshot and scores its findings for a workspace.
    pub fn into_report(self, workspace: impl Into<String>) -> DoctorReport {
        DoctorReport::from_normalized_findings(workspace, self.findings)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CapabilitySnapshotWire {
    format_version: u32,
    cache_key: CapabilityCacheKey,
    output_fingerprint: ContentFingerprint,
    identity: CapabilityCacheIdentity,
    findings: Vec<DoctorFinding>,
}

impl<'de> Deserialize<'de> for CapabilitySnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CapabilitySnapshotWire::deserialize(deserializer)?;
        if wire.format_version != DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION {
            return Err(de::Error::custom(
                CapabilitySnapshotError::UnsupportedFormatVersion {
                    actual: wire.format_version,
                },
            ));
        }
        let derived = wire.identity.cache_key();
        if wire.cache_key != derived {
            return Err(de::Error::custom(
                CapabilitySnapshotError::CacheKeyMismatch {
                    declared: wire.cache_key,
                    derived,
                },
            ));
        }
        let snapshot = Self::try_new(wire.identity, wire.findings).map_err(de::Error::custom)?;
        if wire.output_fingerprint != snapshot.output_fingerprint {
            return Err(de::Error::custom(
                CapabilitySnapshotError::OutputFingerprintMismatch {
                    declared: wire.output_fingerprint,
                    derived: snapshot.output_fingerprint,
                },
            ));
        }
        Ok(snapshot)
    }
}

fn validate_findings(
    identity: &CapabilityCacheIdentity,
    findings: &[DoctorFinding],
) -> Result<(), CapabilitySnapshotError> {
    for finding in findings {
        if finding.provenance.capability != identity.capability() {
            return Err(CapabilitySnapshotError::CapabilityMismatch {
                finding_code: finding.code.clone(),
                expected: identity.capability().into(),
                actual: finding.provenance.capability.clone(),
            });
        }

        for input in &finding.provenance.invalidation_inputs {
            let actual = finding
                .provenance
                .invalidation_fingerprints
                .get(input)
                .copied()
                .ok_or_else(|| CapabilitySnapshotError::MissingFingerprint {
                    finding_code: finding.code.clone(),
                    input: input.clone(),
                })?;
            let expected = identity_fingerprint(identity, input).ok_or_else(|| {
                CapabilitySnapshotError::UndeclaredIdentityInput {
                    finding_code: finding.code.clone(),
                    input: input.clone(),
                }
            })?;
            if actual != expected {
                return Err(CapabilitySnapshotError::FingerprintMismatch {
                    finding_code: finding.code.clone(),
                    input: input.clone(),
                    expected,
                    actual,
                });
            }
        }

        if let Some(input) = finding
            .provenance
            .invalidation_fingerprints
            .keys()
            .find(|input| {
                !finding
                    .provenance
                    .invalidation_inputs
                    .iter()
                    .any(|declared| declared == *input)
            })
        {
            return Err(CapabilitySnapshotError::OrphanFingerprint {
                finding_code: finding.code.clone(),
                input: input.clone(),
            });
        }
    }
    Ok(())
}

fn identity_fingerprint(
    identity: &CapabilityCacheIdentity,
    input: &str,
) -> Option<ContentFingerprint> {
    identity
        .inputs()
        .binary_search_by(|candidate| candidate.id().cmp(input))
        .ok()
        .map(|index| identity.inputs()[index].fingerprint())
}

fn fingerprint_findings(
    findings: &[DoctorFinding],
) -> Result<ContentFingerprint, CapabilitySnapshotError> {
    let mut writer = DigestWriter(Sha256::new());
    writer.0.update(OUTPUT_FINGERPRINT_DOMAIN);
    writer
        .0
        .update(DOCTOR_CAPABILITY_SNAPSHOT_FORMAT_VERSION.to_le_bytes());
    serde_json::to_writer(&mut writer, findings).map_err(|error| {
        CapabilitySnapshotError::OutputSerialization {
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
