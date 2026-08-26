//! Canonical cache identities and precise invalidation for analysis capabilities.

mod error;
mod invalidation;
mod key;
#[cfg(test)]
mod tests;

use std::cmp::Ordering;

use serde::{Deserialize, Deserializer, Serialize, de};
use sha2::{Digest, Sha256};
use vize_s0::String;

use crate::{ContentFingerprint, contract::is_stable_id};

pub use error::CapabilityCacheIdentityError;
pub use invalidation::{CapabilityInvalidation, CapabilityInvalidationTelemetry};
pub use key::{CapabilityCacheKey, CapabilityCacheKeyParseError};

/// Current wire and hashing contract for [`CapabilityCacheIdentity`].
pub const DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION: u32 = 1;

/// Prefix of every serialized [`CapabilityCacheKey`].
pub const CAPABILITY_CACHE_KEY_PREFIX: &str = "vize-doctor-capability-v1:";

const CACHE_KEY_DOMAIN: &[u8] = b"vize-doctor\0capability-cache-identity\0sha256\0";

/// Stable logical input and exact content identity for one analysis capability.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityCacheInput {
    id: String,
    fingerprint: ContentFingerprint,
}

impl CapabilityCacheInput {
    /// Creates a canonical input identity.
    ///
    /// Identifiers are trimmed, slash-normalized logical names. Empty path
    /// segments, `.` and `..`, backslashes, and control characters are
    /// rejected so one input cannot acquire platform-specific aliases.
    pub fn try_new(
        id: impl Into<String>,
        fingerprint: ContentFingerprint,
    ) -> Result<Self, CapabilityCacheIdentityError> {
        let id = id.into();
        if !is_stable_input_id(&id) {
            return Err(CapabilityCacheIdentityError::InvalidInputId { input: id });
        }
        Ok(Self { id, fingerprint })
    }

    /// Returns the canonical logical input identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the exact content identity used for invalidation.
    pub const fn fingerprint(&self) -> ContentFingerprint {
        self.fingerprint
    }
}

/// Complete identity of one independently reusable analysis capability.
///
/// The identity separates implementation, configuration, and input content so
/// invalidation can explain exactly which boundary changed. Input ordering is
/// canonicalized and duplicate logical identifiers are rejected. Producers
/// must include the complete discovery boundary: an omitted input is a promise
/// that its content cannot affect this capability's output.
///
/// The contract intentionally excludes timestamps, absolute paths, process
/// identifiers, and host metadata. Local and remote workers therefore derive
/// the same [`CapabilityCacheKey`] from the same declared analysis state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityCacheIdentity {
    contract_version: u32,
    capability: String,
    implementation_fingerprint: ContentFingerprint,
    configuration_fingerprint: ContentFingerprint,
    inputs: Vec<CapabilityCacheInput>,
}

impl CapabilityCacheIdentity {
    /// Creates an identity and canonicalizes input order.
    pub fn try_new(
        capability: impl Into<String>,
        implementation_fingerprint: ContentFingerprint,
        configuration_fingerprint: ContentFingerprint,
        inputs: impl IntoIterator<Item = CapabilityCacheInput>,
    ) -> Result<Self, CapabilityCacheIdentityError> {
        let capability = capability.into();
        if !is_stable_id(&capability) {
            return Err(CapabilityCacheIdentityError::InvalidCapabilityId { capability });
        }

        let mut inputs = inputs.into_iter().collect::<Vec<_>>();
        inputs.sort_unstable_by(|left, right| left.id.cmp(&right.id));
        if let Some(duplicate) = inputs
            .windows(2)
            .find(|window| window[0].id == window[1].id)
        {
            return Err(CapabilityCacheIdentityError::DuplicateInput {
                input: duplicate[0].id.clone(),
            });
        }

        Ok(Self {
            contract_version: DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION,
            capability,
            implementation_fingerprint,
            configuration_fingerprint,
            inputs,
        })
    }

    /// Creates an identity directly from logical names and fingerprints.
    pub fn from_fingerprints<S>(
        capability: impl Into<String>,
        implementation_fingerprint: ContentFingerprint,
        configuration_fingerprint: ContentFingerprint,
        inputs: impl IntoIterator<Item = (S, ContentFingerprint)>,
    ) -> Result<Self, CapabilityCacheIdentityError>
    where
        S: Into<String>,
    {
        let inputs = inputs
            .into_iter()
            .map(|(id, fingerprint)| CapabilityCacheInput::try_new(id, fingerprint))
            .collect::<Result<Vec<_>, _>>()?;
        Self::try_new(
            capability,
            implementation_fingerprint,
            configuration_fingerprint,
            inputs,
        )
    }

    /// Returns the cache-identity contract version.
    pub const fn contract_version(&self) -> u32 {
        self.contract_version
    }

    /// Returns the stable analysis capability identifier.
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// Returns the identity of the capability implementation.
    pub const fn implementation_fingerprint(&self) -> ContentFingerprint {
        self.implementation_fingerprint
    }

    /// Returns the identity of all behavior-affecting configuration.
    pub const fn configuration_fingerprint(&self) -> ContentFingerprint {
        self.configuration_fingerprint
    }

    /// Returns inputs in strict logical-identifier order.
    pub fn inputs(&self) -> &[CapabilityCacheInput] {
        &self.inputs
    }

    /// Derives the domain-separated, platform-independent cache key.
    #[must_use]
    pub fn cache_key(&self) -> CapabilityCacheKey {
        let mut digest = Sha256::new();
        digest.update(CACHE_KEY_DOMAIN);
        digest.update(self.contract_version.to_be_bytes());
        update_field(&mut digest, 1, self.capability.as_bytes());
        update_field(&mut digest, 2, self.implementation_fingerprint.as_bytes());
        update_field(&mut digest, 3, self.configuration_fingerprint.as_bytes());
        digest.update((self.inputs.len() as u64).to_be_bytes());
        for input in &self.inputs {
            update_field(&mut digest, 4, input.id.as_bytes());
            update_field(&mut digest, 5, input.fingerprint.as_bytes());
        }
        CapabilityCacheKey::from_fingerprint(ContentFingerprint::from_digest(
            digest.finalize().into(),
        ))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CapabilityCacheIdentityWire {
    contract_version: u32,
    capability: String,
    implementation_fingerprint: ContentFingerprint,
    configuration_fingerprint: ContentFingerprint,
    inputs: Vec<CapabilityCacheInputWire>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CapabilityCacheInputWire {
    id: String,
    fingerprint: ContentFingerprint,
}

impl<'de> Deserialize<'de> for CapabilityCacheIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CapabilityCacheIdentityWire::deserialize(deserializer)?;
        if wire.contract_version != DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION {
            return Err(de::Error::custom(
                CapabilityCacheIdentityError::UnsupportedContractVersion {
                    actual: wire.contract_version,
                },
            ));
        }

        let inputs = wire
            .inputs
            .into_iter()
            .map(|input| CapabilityCacheInput::try_new(input.id, input.fingerprint))
            .collect::<Result<Vec<_>, _>>()
            .map_err(de::Error::custom)?;
        for pair in inputs.windows(2) {
            match pair[0].id.cmp(&pair[1].id) {
                Ordering::Less => {}
                Ordering::Equal => {
                    return Err(de::Error::custom(
                        CapabilityCacheIdentityError::DuplicateInput {
                            input: pair[0].id.clone(),
                        },
                    ));
                }
                Ordering::Greater => {
                    return Err(de::Error::custom(
                        CapabilityCacheIdentityError::NonCanonicalInputOrder {
                            previous: pair[0].id.clone(),
                            current: pair[1].id.clone(),
                        },
                    ));
                }
            }
        }

        Self::try_new(
            wire.capability,
            wire.implementation_fingerprint,
            wire.configuration_fingerprint,
            inputs,
        )
        .map_err(de::Error::custom)
    }
}

fn update_field(digest: &mut Sha256, tag: u8, bytes: &[u8]) {
    digest.update([tag]);
    digest.update((bytes.len() as u64).to_be_bytes());
    digest.update(bytes);
}

fn is_stable_input_id(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && !value.contains(['\\', '\0'])
        && !is_windows_drive_path(value)
        && !value.chars().any(char::is_control)
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn is_windows_drive_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}
