use std::{error::Error, fmt};

use vize_s0::String;

use super::DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION;

/// Invalid capability cache identity or non-canonical wire data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityCacheIdentityError {
    /// The wire contract version is not supported by this implementation.
    UnsupportedContractVersion {
        /// Version found in the wire payload.
        actual: u32,
    },
    /// The capability does not use the stable lowercase identifier grammar.
    InvalidCapabilityId {
        /// Rejected capability identifier.
        capability: String,
    },
    /// An input has a platform-dependent or ambiguous logical identifier.
    InvalidInputId {
        /// Rejected input identifier.
        input: String,
    },
    /// Two fingerprints claim the same logical input identifier.
    DuplicateInput {
        /// Duplicated input identifier.
        input: String,
    },
    /// Wire inputs are not in strict canonical identifier order.
    NonCanonicalInputOrder {
        /// Identifier that appeared first in the payload.
        previous: String,
        /// Out-of-order identifier found after it.
        current: String,
    },
}

impl fmt::Display for CapabilityCacheIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedContractVersion { actual } => write!(
                formatter,
                "unsupported capability cache identity version {actual}; expected {DOCTOR_CAPABILITY_CACHE_IDENTITY_VERSION}"
            ),
            Self::InvalidCapabilityId { capability } => write!(
                formatter,
                "invalid capability cache identifier {capability:?}"
            ),
            Self::InvalidInputId { input } => {
                write!(formatter, "invalid capability cache input {input:?}")
            }
            Self::DuplicateInput { input } => {
                write!(formatter, "duplicate capability cache input {input:?}")
            }
            Self::NonCanonicalInputOrder { previous, current } => write!(
                formatter,
                "capability cache inputs are not canonical: {current:?} follows {previous:?}"
            ),
        }
    }
}

impl Error for CapabilityCacheIdentityError {}
