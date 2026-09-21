//! [`KeySink`] — the one place a key's bytes are encoded.
//!
//! Every key starts with the same domain prefix — a fixed tag, the stage's
//! physical id and the recipe version — so keys of different stages or
//! versions can never collide even on equal pages. After the prefix a
//! keyer feeds either raw page text (the [`fmt::Write`] impl, for a folio
//! `Full` print) or typed fields (length-prefixed strings, little-endian
//! integers, rebased spans). Both are platform-stable encodings.

use core::fmt;

use vize_s0::Span;
use vize_s0::hash::StableHasher128;

use super::{ArtifactKey, rebase};
use crate::stage::Stage;

/// The domain tag every artifact key hash starts with.
const DOMAIN: &[u8] = b"vize.davinci.artifact-key\0";

/// Field tags: a span inside its block, and one reaching outside it.
const SPAN_RELATIVE: u8 = 0;
const SPAN_OUTSIDE: u8 = 1;

/// A streaming key encoder for one artifact of one block.
///
/// Created by [`ArtifactKey::of`] (or directly, for keyers that are not a
/// [`KeyedArtifact`](super::KeyedArtifact) type) and consumed by
/// [`KeySink::finish`].
#[derive(Clone)]
pub struct KeySink {
    hasher: StableHasher128,
    stage: Stage,
    schema_version: u32,
    block_start: u32,
}

impl KeySink {
    /// A sink for a `stage` artifact under recipe `schema_version`, whose
    /// spans are rebased to `block_start`.
    #[must_use]
    pub fn new(stage: Stage, schema_version: u32, block_start: u32) -> Self {
        let mut hasher = StableHasher128::new();
        hasher.update(DOMAIN);
        let id = stage.physical_id().as_bytes();
        hasher.update(&(id.len() as u32).to_le_bytes());
        hasher.update(id);
        hasher.update(&schema_version.to_le_bytes());
        Self {
            hasher,
            stage,
            schema_version,
            block_start,
        }
    }

    /// The file-absolute start offset spans are rebased to.
    #[must_use]
    pub const fn block_start(&self) -> u32 {
        self.block_start
    }

    /// Feed one tag byte (an enum discriminant or a presence marker).
    pub fn feed_tag(&mut self, tag: u8) {
        self.hasher.update(&[tag]);
    }

    /// Feed a `u32`, little-endian.
    pub fn feed_u32(&mut self, value: u32) {
        self.hasher.update(&value.to_le_bytes());
    }

    /// Feed a string field: its byte length (`u64`, little-endian), then its
    /// bytes. The length prefix keeps adjacent fields unambiguous.
    pub fn feed_str(&mut self, value: &str) {
        self.hasher.update(&(value.len() as u64).to_le_bytes());
        self.hasher.update(value.as_bytes());
    }

    /// Feed a span rebased to [`block_start`](Self::block_start); a span
    /// reaching before the block is fed absolute under its own marker.
    pub fn feed_span(&mut self, span: Span) {
        let (tag, span) = match rebase(span, self.block_start) {
            Some(relative) => (SPAN_RELATIVE, relative),
            None => (SPAN_OUTSIDE, span),
        };
        self.feed_tag(tag);
        self.feed_u32(span.start);
        self.feed_u32(span.end);
    }

    /// The finished key.
    #[must_use]
    pub fn finish(self) -> ArtifactKey {
        ArtifactKey {
            stage: self.stage,
            schema_version: self.schema_version,
            hash: self.hasher.digest(),
        }
    }
}

/// Raw page text: a folio `Full` print streams through this. Never fails.
impl fmt::Write for KeySink {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.hasher.update(text.as_bytes());
        Ok(())
    }
}
