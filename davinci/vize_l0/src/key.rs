//! Span-relative content keys for cached level artifacts.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use core::fmt;

use crate::level::Level;
use crate::span::Span;

/// A level artifact with a content key.
pub trait KeyedArtifact {
    /// The level this artifact belongs to.
    const LEVEL: Level;
    /// The key recipe version.
    const SCHEMA_VERSION: u32;

    /// Feed an injective encoding of the artifact's canonical page.
    fn feed_key(&self, sink: &mut KeySink);
}

/// The content key of one artifact of one block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactKey {
    level: Level,
    schema_version: u32,
    hash: [u8; 16],
}

impl ArtifactKey {
    /// Key `artifact`, rebasing its spans to `block_start`.
    #[must_use]
    pub fn of<A: KeyedArtifact + ?Sized>(artifact: &A, block_start: u32) -> Self {
        let _ = (artifact, block_start);
        todo!()
    }

    /// The level of the keyed artifact.
    #[must_use]
    pub const fn level(&self) -> Level {
        self.level
    }

    /// The key recipe version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// The 128-bit content hash.
    #[must_use]
    pub const fn hash(&self) -> [u8; 16] {
        self.hash
    }
}

/// The hashing sink a [`KeyedArtifact`] feeds.
#[derive(Debug)]
pub struct KeySink {
    block_start: u32,
}

impl KeySink {
    /// The block start spans are rebased against.
    #[must_use]
    pub const fn block_start(&self) -> u32 {
        self.block_start
    }

    /// Feed raw bytes.
    pub fn feed_bytes(&mut self, bytes: &[u8]) {
        let _ = bytes;
        todo!()
    }

    /// Feed a span, rebased to the block start.
    pub fn feed_span(&mut self, span: Span) {
        let _ = span;
        todo!()
    }
}

impl fmt::Write for KeySink {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.feed_bytes(text.as_bytes());
        Ok(())
    }
}
