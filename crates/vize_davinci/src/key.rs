//! Stage artifact keys (Davinci P5-1a): the content identity every cache of
//! a stage artifact keys on.
//!
//! An [`ArtifactKey`] names one stage artifact of one SFC block — its S0
//! source block, its S1 surface page, its S2 page — by content, in the shape
//! of Doctor's `cache_identity` (domain-separated, versioned, explicit):
//!
//! - **Normalized structure, not presentation.** The hash walks the
//!   artifact's canonical page: the Disegno folio `Full` form for S2, the
//!   lossless render for S1 (S1's page — a lossless tree's page *is* its
//!   token sequence). `Full` is the injective form the folio laws pin, so
//!   two artifacts share a key exactly when their pages are equal.
//! - **Span-relative.** Every span enters the hash rebased to the start of
//!   its block ([`rebase`], the rustc relative-span import); the block's
//!   absolute position is S0 side-table data *outside* the key. An edit
//!   above a block therefore changes zero keys of that block, and identical
//!   block content keys identically at any offset.
//! - **Versioned.** `schema_version` sits inside every key *and* inside the
//!   hashed domain prefix. Changing a page grammar or a key recipe bumps the
//!   stage's entry in [`schema`], which changes every key of that stage.
//! - **Platform-stable.** Every value is fed as an explicit little-endian
//!   fixed-width encoding into XXH3-128 ([`StableHasher128`]); nothing goes
//!   through `core::hash::Hash`, whose `usize` lengths differ by target. The
//!   TS-43 goldens pin the exact keys, and the Linux and macOS CI lanes both
//!   check them.
//!
//! Ambient inputs — tsconfig content, toolchain and Corsa versions, feature
//! flags, platform — are not artifact content; they fold in through the key
//! manifest (P5-1b), never by widening a page.
//!
//! [`StableHasher128`]: vize_s0::hash::StableHasher128

use core::fmt;

use vize_s0::Span;

use crate::stage::Stage;

mod block;
pub mod manifest;
pub(crate) mod sink;
#[cfg(test)]
mod tests;

pub use block::source_block_key;
pub use manifest::{AmbientInput, CachedArtifact, InputSet, KeyManifest, ManifestError};
pub use sink::KeySink;

/// Key-recipe versions, one per keyed stage artifact.
///
/// A version names both the page grammar the hash walks and the feeding
/// recipe. Bump it in the same change that alters either; the TS-43 goldens
/// fail otherwise, which is the point.
pub mod schema {
    /// S0 source block: block kind, header attributes as a sorted set, and
    /// the content bytes, each length-prefixed.
    pub const SOURCE_BLOCK: u32 = 1;
    /// S1 surface page: the lossless render, one length-prefixed piece per
    /// token slice (`leading`, then `text`), in canonical token order.
    pub const S1_SURFACE: u32 = 1;
    /// S2 page: the Disegno folio `Full` form with every span rebased to
    /// the block start.
    pub const S2_PAGE: u32 = 1;
}

/// A stage artifact with a content key.
///
/// Implementations feed an **injective** encoding of the artifact's
/// canonical page: a folio `Full` print (injective by the folio laws)
/// streamed through [`fmt::Write`], or length-prefixed typed fields. Spans
/// go through [`KeySink::feed_span`] or [`rebase`] against
/// [`KeySink::block_start`], never raw.
pub trait KeyedArtifact {
    /// The stage this artifact belongs to.
    const STAGE: Stage;
    /// The key recipe version (one of the [`schema`] constants).
    const SCHEMA_VERSION: u32;

    /// Feed the artifact's canonical page into `sink`.
    fn feed_key(&self, sink: &mut KeySink);
}

/// The content key of one stage artifact of one block.
///
/// Equal keys mean equal canonical pages (up to the 128-bit collision
/// bound) under the same stage and recipe version. Keys are plain data:
/// `Copy`, totally ordered, and printed as `s2.v1:<32 hex digits>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactKey {
    stage: Stage,
    schema_version: u32,
    hash: [u8; 16],
}

impl ArtifactKey {
    /// Key `artifact`, rebasing its spans to `block_start` (the block's
    /// file-absolute start offset; `0` for a block parsed as its own root).
    #[must_use]
    pub fn of<A: KeyedArtifact + ?Sized>(artifact: &A, block_start: u32) -> Self {
        let mut sink = KeySink::new(A::STAGE, A::SCHEMA_VERSION, block_start);
        artifact.feed_key(&mut sink);
        sink.finish()
    }

    /// The stage the keyed artifact belongs to.
    #[must_use]
    pub const fn stage(&self) -> Stage {
        self.stage
    }

    /// The key recipe version the hash was computed under.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// The XXH3-128 digest, big-endian.
    #[must_use]
    pub const fn hash(&self) -> [u8; 16] {
        self.hash
    }
}

impl fmt::Display for ArtifactKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.v{}:", self.stage.physical_id(), self.schema_version)?;
        for byte in self.hash {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Rebase a file-absolute `span` to block-relative offsets.
///
/// `None` when the span reaches before `block_start` — an out-of-block span
/// has no relative form, so keyers feed it absolute under a distinct marker
/// instead of letting [`Span::to_block_relative`] saturate two different
/// spans onto one. Well-formed stage artifacts never produce one.
#[inline]
#[must_use]
pub const fn rebase(span: Span, block_start: u32) -> Option<Span> {
    if span.start < block_start || span.end < block_start {
        return None;
    }
    Some(span.to_block_relative(block_start))
}
