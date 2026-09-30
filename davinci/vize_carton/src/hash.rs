//! Fast hashing utilities using xxHash3.
//!
//! Provides high-performance hashing for HMR change detection
//! and content-based cache invalidation.

use crate::String;
use xxhash_rust::xxh3::{Xxh3Default, xxh3_64};

/// Compute a 64-bit hash of the given bytes using xxHash3.
///
/// This is the fastest hash algorithm available, suitable for
/// non-cryptographic purposes like change detection.
#[inline]
pub fn hash_bytes(data: &[u8]) -> u64 {
    xxh3_64(data)
}

/// Compute a 64-bit hash of the given string using xxHash3.
#[inline]
pub fn hash_str(data: &str) -> u64 {
    xxh3_64(data.as_bytes())
}

/// Convert a hash to a hex string (16 characters).
#[inline]
pub fn hash_to_hex(hash: u64) -> String {
    let mut out = String::with_capacity(16);
    for index in 0..16 {
        let shift = (15 - index) * 4;
        // A nibble is always a hex digit.
        if let Some(digit) = char::from_digit(((hash >> shift) & 0xF) as u32, 16) {
            out.push(digit);
        }
    }
    out
}

/// Compute hash of a string and return as hex.
#[inline]
pub fn content_hash(content: &str) -> String {
    hash_to_hex(hash_str(content))
}

/// Streaming 128-bit XXH3 over explicitly fed bytes: the platform-stable
/// fingerprint behind Davinci's stage artifact keys (`vize_davinci::key`).
///
/// The digest is the XXH3-128 value of the concatenated input. XXH3 is
/// specified over bytes, so the value is independent of endianness, pointer
/// width and the SIMD path the implementation picks: equal input yields an
/// equal digest on every platform (the property TS-43 checks on two CI
/// platforms). Callers must feed explicit fixed-width encodings, never
/// [`core::hash::Hash`], whose `usize` lengths differ between 32- and 64-bit
/// targets. 128 bits is the rustc `Fingerprint` width, chosen for the same
/// reason: accidental collisions stay negligible across a whole session of
/// cached artifacts.
#[derive(Clone)]
pub struct StableHasher128(Xxh3Default);

impl StableHasher128 {
    /// An empty hasher (the default XXH3 seed and secret).
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Xxh3Default::new())
    }

    /// Feed `bytes`; streaming in pieces equals hashing the concatenation.
    #[inline]
    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    /// The XXH3-128 digest of everything fed so far, as big-endian bytes
    /// (so the hex spelling reads as the canonical XXH3-128 value).
    #[inline]
    #[must_use]
    pub fn digest(&self) -> [u8; 16] {
        self.0.digest128().to_be_bytes()
    }
}

impl Default for StableHasher128 {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{StableHasher128, content_hash, hash_str, hash_to_hex};

    #[test]
    fn stable_hasher_streaming_equals_the_one_shot_digest() {
        let mut streamed = StableHasher128::new();
        streamed.update(b"<template>");
        streamed.update(b"");
        streamed.update(b"<div/></template>");
        let one_shot = xxhash_rust::xxh3::xxh3_128(b"<template><div/></template>");
        assert_eq!(streamed.digest(), one_shot.to_be_bytes());
    }

    #[test]
    fn stable_hasher_matches_the_xxh3_128_empty_input_vector() {
        // XXH3_128bits("") from the reference implementation's test vectors.
        assert_eq!(
            StableHasher128::default().digest(),
            0x99aa_06d3_0147_98d8_6001_c324_468d_497f_u128.to_be_bytes()
        );
    }

    #[test]
    fn test_hash_consistency() {
        let content = "Hello, World!";
        let hash1 = hash_str(content);
        let hash2 = hash_str(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_difference() {
        let hash1 = hash_str("Hello");
        let hash2 = hash_str("World");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hex_format() {
        let hash = hash_str("test");
        let hex = hash_to_hex(hash);
        assert_eq!(hex.len(), 16);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_content_hash() {
        let hash = content_hash("template content");
        assert_eq!(hash.len(), 16);
    }
}
