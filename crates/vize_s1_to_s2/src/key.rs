//! The S1 surface page as a keyed artifact (P5-1a).
//!
//! S1 cannot name `vize_davinci` (the stage-dependency table keeps it on
//! S0 alone), so its [`KeyedArtifact`] view lives in this conversion crate,
//! the one place that already joins S1 and the shared infrastructure. The
//! S2 page keys itself (`impl KeyedArtifact for S2Folio` in `vize_s2`).
//!
//! S1's page is its lossless render: every token slice in canonical order,
//! `Missing` holes included as empty pieces. Each piece is fed
//! length-prefixed, so the key covers the token segmentation (the tree's
//! terminal structure), not just the concatenated bytes. Token slices carry
//! no offsets, so the page is block-relative by construction and the block
//! start never enters the hash.

use vize_davinci::key::{KeySink, KeyedArtifact, schema};
use vize_davinci::stage::Stage;
use vize_s1::SurfaceTree;

/// An S1 surface tree viewed as its keyed page.
///
/// `ArtifactKey::of(&SurfacePage(&tree), block.start())` is the S1 key of
/// the block `tree` was parsed from.
#[derive(Debug, Clone, Copy)]
pub struct SurfacePage<'t, 'a>(pub &'t SurfaceTree<'a>);

impl KeyedArtifact for SurfacePage<'_, '_> {
    const STAGE: Stage = Stage::Surface;
    const SCHEMA_VERSION: u32 = schema::S1_SURFACE;

    fn feed_key(&self, sink: &mut KeySink) {
        vize_s1::render(self.0, &mut |piece| sink.feed_str(piece));
    }
}
