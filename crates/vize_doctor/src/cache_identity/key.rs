use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{ContentFingerprint, ContentFingerprintParseError};

use super::CAPABILITY_CACHE_KEY_PREFIX;

/// Domain-separated digest naming one exact capability analysis.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityCacheKey(ContentFingerprint);

impl CapabilityCacheKey {
    pub(super) const fn from_fingerprint(fingerprint: ContentFingerprint) -> Self {
        Self(fingerprint)
    }

    /// Returns the underlying canonical SHA-256 fingerprint.
    pub const fn fingerprint(self) -> ContentFingerprint {
        self.0
    }
}

impl fmt::Display for CapabilityCacheKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{CAPABILITY_CACHE_KEY_PREFIX}{}", self.0)
    }
}

impl fmt::Debug for CapabilityCacheKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CapabilityCacheKey({self})")
    }
}

impl FromStr for CapabilityCacheKey {
    type Err = CapabilityCacheKeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let fingerprint = value
            .strip_prefix(CAPABILITY_CACHE_KEY_PREFIX)
            .ok_or(CapabilityCacheKeyParseError::InvalidPrefix)?
            .parse()
            .map_err(CapabilityCacheKeyParseError::InvalidFingerprint)?;
        Ok(Self(fingerprint))
    }
}

impl Serialize for CapabilityCacheKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for CapabilityCacheKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CacheKeyVisitor;

        impl de::Visitor<'_> for CacheKeyVisitor {
            type Value = CapabilityCacheKey;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a canonical vize-doctor-capability-v1 cache key")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(CacheKeyVisitor)
    }
}

/// Invalid serialized [`CapabilityCacheKey`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityCacheKeyParseError {
    /// The versioned Doctor capability prefix is absent or has different casing.
    InvalidPrefix,
    /// The embedded content fingerprint is not canonical.
    InvalidFingerprint(ContentFingerprintParseError),
}

impl fmt::Display for CapabilityCacheKeyParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(
                formatter,
                "capability cache key must start with {CAPABILITY_CACHE_KEY_PREFIX}"
            ),
            Self::InvalidFingerprint(error) => {
                write!(formatter, "invalid capability cache key: {error}")
            }
        }
    }
}

impl Error for CapabilityCacheKeyParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPrefix => None,
            Self::InvalidFingerprint(error) => Some(error),
        }
    }
}
