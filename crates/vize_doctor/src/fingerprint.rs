//! Stable content identities for incremental analysis and remote caches.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use sha2::{Digest, Sha256};

/// Prefix of every serialized [`ContentFingerprint`].
pub const CONTENT_FINGERPRINT_PREFIX: &str = "sha256:";

const DIGEST_BYTES: usize = 32;
const HEX_BYTES: usize = DIGEST_BYTES * 2;
const SERIALIZED_BYTES: usize = CONTENT_FINGERPRINT_PREFIX.len() + HEX_BYTES;

/// Immutable SHA-256 identity of an exact byte sequence.
///
/// The stable wire form is `sha256:` followed by exactly 64 lowercase
/// hexadecimal digits. [`Self::digest`] hashes the supplied bytes directly,
/// without path, platform, locale, or process metadata, so independent local
/// and remote analyzers produce the same identity for identical content.
///
/// Parsing is deliberately strict: uppercase hexadecimal, surrounding
/// whitespace, missing prefixes, and alternate algorithms are rejected. This
/// prevents multiple cache keys from naming the same digest.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentFingerprint([u8; DIGEST_BYTES]);

impl ContentFingerprint {
    /// Hash an exact byte sequence with SHA-256.
    #[must_use]
    pub fn digest(content: impl AsRef<[u8]>) -> Self {
        Self(Sha256::digest(content.as_ref()).into())
    }

    /// Construct an identity from an already verified SHA-256 digest.
    #[must_use]
    pub const fn from_digest(digest: [u8; DIGEST_BYTES]) -> Self {
        Self(digest)
    }

    /// Return the exact 32 digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }
}

impl fmt::Display for ContentFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(CONTENT_FINGERPRINT_PREFIX)?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for ContentFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ContentFingerprint({self})")
    }
}

impl FromStr for ContentFingerprint {
    type Err = ContentFingerprintParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !value.starts_with(CONTENT_FINGERPRINT_PREFIX) {
            return Err(ContentFingerprintParseError::InvalidPrefix);
        }
        if value.len() != SERIALIZED_BYTES {
            return Err(ContentFingerprintParseError::InvalidLength {
                actual: value.len(),
            });
        }

        let hex = &value.as_bytes()[CONTENT_FINGERPRINT_PREFIX.len()..];
        let mut digest = [0; DIGEST_BYTES];
        for (index, pair) in hex.as_chunks::<2>().0.iter().enumerate() {
            let high =
                decode_lower_hex(pair[0]).ok_or(ContentFingerprintParseError::InvalidHex {
                    index: CONTENT_FINGERPRINT_PREFIX.len() + index * 2,
                })?;
            let low =
                decode_lower_hex(pair[1]).ok_or(ContentFingerprintParseError::InvalidHex {
                    index: CONTENT_FINGERPRINT_PREFIX.len() + index * 2 + 1,
                })?;
            digest[index] = high << 4 | low;
        }
        Ok(Self(digest))
    }
}

impl Serialize for ContentFingerprint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ContentFingerprint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FingerprintVisitor;

        impl de::Visitor<'_> for FingerprintVisitor {
            type Value = ContentFingerprint;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a canonical sha256: content fingerprint")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(FingerprintVisitor)
    }
}

/// Invalid canonical content-fingerprint text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFingerprintParseError {
    /// The required `sha256:` prefix is absent or has different casing.
    InvalidPrefix,
    /// The complete UTF-8 representation is not exactly 71 bytes.
    InvalidLength {
        /// Observed byte length.
        actual: usize,
    },
    /// One byte is not a lowercase hexadecimal digit.
    InvalidHex {
        /// Zero-based byte offset in the complete representation.
        index: usize,
    },
}

impl fmt::Display for ContentFingerprintParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => {
                formatter.write_str("content fingerprint must start with sha256:")
            }
            Self::InvalidLength { actual } => write!(
                formatter,
                "content fingerprint must be {SERIALIZED_BYTES} bytes, found {actual}"
            ),
            Self::InvalidHex { index } => write!(
                formatter,
                "content fingerprint has non-lowercase-hex byte at offset {index}"
            ),
        }
    }
}

impl std::error::Error for ContentFingerprintParseError {}

const fn decode_lower_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentFingerprint, ContentFingerprintParseError};
    use vize_carton::{String, ToCompactString, cstr};

    #[test]
    fn digest_matches_standard_sha256_vectors() {
        assert_eq!(
            ContentFingerprint::digest([]).to_compact_string(),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            ContentFingerprint::digest("abc").to_compact_string(),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn canonical_text_and_json_round_trip_exact_digest_bytes() {
        let fingerprint = ContentFingerprint::digest("<template>\n  <p>保存</p>\n</template>");
        let text = fingerprint.to_compact_string();
        assert_eq!(text.parse::<ContentFingerprint>().unwrap(), fingerprint);

        let json = serde_json::to_string(&fingerprint).unwrap();
        assert_eq!(json, cstr!("\"{text}\""));
        assert_eq!(
            serde_json::from_str::<ContentFingerprint>(&json).unwrap(),
            fingerprint
        );
        assert_eq!(
            cstr!("{fingerprint:?}"),
            cstr!("ContentFingerprint({text})")
        );
    }

    #[test]
    fn parser_rejects_every_noncanonical_boundary() {
        let valid = ContentFingerprint::digest("abc").to_compact_string();
        let cases: [(String, ContentFingerprintParseError); 6] = [
            (
                valid.replacen("sha256:", "SHA256:", 1).into(),
                ContentFingerprintParseError::InvalidPrefix,
            ),
            (
                valid.trim_start_matches("sha256:").into(),
                ContentFingerprintParseError::InvalidPrefix,
            ),
            (
                cstr!("{valid}0"),
                ContentFingerprintParseError::InvalidLength { actual: 72 },
            ),
            (
                valid[..valid.len() - 1].into(),
                ContentFingerprintParseError::InvalidLength { actual: 70 },
            ),
            (
                valid.replacen('b', "B", 1).into(),
                ContentFingerprintParseError::InvalidHex { index: 7 },
            ),
            (
                valid.replacen('b', "g", 1).into(),
                ContentFingerprintParseError::InvalidHex { index: 7 },
            ),
        ];

        for (text, expected) in cases {
            assert_eq!(text.parse::<ContentFingerprint>().unwrap_err(), expected);
            assert!(serde_json::from_str::<ContentFingerprint>(&cstr!("\"{text}\"")).is_err());
        }
    }
}
