//! The snapshot tree (P5-5): Lean-style snapshot tasks at an SFC's natural
//! joints — header → block → S2 region — below the salsa tier's block
//! firewall, with one reuse rule: **old syntax ≡ new syntax ⇒ adopt the old
//! subtree**.
//!
//! - **Header joint.** The block layout (kinds, ordinals, header attributes;
//!   not positions) plus the project configuration. A changed header
//!   restarts the file: every old task is cancelled, nothing is adopted
//!   (Lean's "header changed ⇒ restart").
//! - **Block joint.** A block whose [`BlockSource`] is unchanged is adopted
//!   whole — its S1 and S2 artifacts are shared, not recomputed — wherever it
//!   moved. A changed block's old task is cancelled and a new one computed.
//! - **Region joint.** A changed template block whose root splits into
//!   regions ([`region`]) re-lowers only the regions whose syntax changed:
//!   a region with the same bytes is adopted, its spans moved when it moved
//!   inside the block. The page is the concatenation, equal to the
//!   whole-block lowering (TS-42 compares it after every step).
//!
//! Every task owns a [`CancelToken`] that is a child of its parent's, so
//! cancelling a file cancels its blocks and regions; replaced subtrees are
//! cancelled, and a cancelled update stops between units of stage work.
//! Adopted subtrees are shared (`Arc`), never copied. [`isolate`] runs file
//! updates on worker threads under `catch_unwind`: a panicking stage degrades
//! its own file and every other file keeps answering.

pub mod cancel;
pub mod isolate;
pub mod region;
mod shift;

// Snapshots are immutable and shared between the tree that computed them and
// every later tree that adopts them, possibly on another worker thread.
#[allow(clippy::disallowed_types)]
type Shared<T> = std::sync::Arc<T>;

use vize_davinci::key::ArtifactKey;
use vize_s0::String;
use vize_s1_to_s2::LegacyCaps;

use crate::artifact::{
    BlockArtifacts, BlockKind, BlockSource, PageArtifact, StageConfig, SurfaceArtifact,
    page_artifact, split_blocks, surface_artifact,
};
use cancel::{CancelToken, Cancelled};
use region::{RegionLowering, RegionSyntax, assemble, lower_region, split_regions};
use shift::shifted;

/// The stage functions a snapshot update calls — the real ones by default;
/// a test substitutes a failing stage to exercise isolation.
#[derive(Debug, Clone, Copy)]
pub struct Stages {
    /// The S1 artifact of a block.
    pub surface: fn(&BlockSource) -> Option<SurfaceArtifact>,
    /// The S2 artifact of a block lowered whole.
    pub page: fn(&BlockSource, StageConfig) -> Option<PageArtifact>,
    /// The lowering of one template region.
    pub region: fn(&str, &RegionSyntax, LegacyCaps) -> RegionLowering,
}

impl Stages {
    /// The production stage functions.
    pub const DEFAULT: Self = Self {
        surface: surface_artifact,
        page: page_artifact,
        region: lower_region,
    };
}

/// Adopted / computed / cancelled tasks at one joint in one update.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JointCounts {
    /// Old subtrees adopted without running a stage.
    pub adopted: u32,
    /// New tasks run.
    pub computed: u32,
    /// Old tasks cancelled because their syntax changed.
    pub cancelled: u32,
}

/// The cache-hit accounting of one update (TS-46).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SnapshotStats {
    /// The header joint (one task per file).
    pub header: JointCounts,
    /// The block joint.
    pub blocks: JointCounts,
    /// The S2 region joint.
    pub regions: JointCounts,
}

impl core::ops::AddAssign for JointCounts {
    fn add_assign(&mut self, other: Self) {
        self.adopted += other.adopted;
        self.computed += other.computed;
        self.cancelled += other.cancelled;
    }
}

impl core::ops::AddAssign for SnapshotStats {
    fn add_assign(&mut self, other: Self) {
        self.header += other.header;
        self.blocks += other.blocks;
        self.regions += other.regions;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HeaderEntry {
    kind: BlockKind,
    ordinal: u32,
    attrs: Vec<(String, String)>,
}

/// A region's finished result: shared by every tree that adopts it.
#[derive(Debug)]
struct RegionSnapshot {
    syntax: RegionSyntax,
    lowering: RegionLowering,
}

/// A block's finished result: shared by every tree that adopts it.
#[derive(Debug)]
struct BlockSnapshot {
    source: BlockSource,
    surface: Option<SurfaceArtifact>,
    page: Option<PageArtifact>,
    regions: Vec<Shared<RegionSnapshot>>,
}

/// A block's place in one tree. Tokens live here, not in the shared
/// snapshots: an adopted subtree gets fresh tokens parented to the tree
/// that adopted it, so cancellation always cascades along the current tree.
#[derive(Debug, Clone)]
struct BlockEntry {
    kind: BlockKind,
    ordinal: u32,
    start: u32,
    token: CancelToken,
    region_tokens: Vec<CancelToken>,
    snapshot: Shared<BlockSnapshot>,
}

/// One file's snapshot tree.
#[derive(Debug, Clone)]
pub struct SnapshotTree {
    header: Vec<HeaderEntry>,
    config: StageConfig,
    token: CancelToken,
    blocks: Vec<BlockEntry>,
}

impl SnapshotTree {
    /// The tree for `text`, adopting from `previous` wherever the syntax at a
    /// joint is unchanged. `token` is the new file task's token; when it is
    /// cancelled the update stops with [`Cancelled`] and `previous` stays
    /// valid. Replaced tasks of `previous` are cancelled.
    pub fn update(
        previous: Option<&Self>,
        text: &str,
        config: StageConfig,
        stages: &Stages,
        token: CancelToken,
    ) -> Result<(Self, SnapshotStats), Cancelled> {
        token.check()?;
        let mut stats = SnapshotStats::default();
        let slots = split_blocks(text);
        let header: Vec<HeaderEntry> = slots
            .iter()
            .map(|slot| HeaderEntry {
                kind: slot.kind.clone(),
                ordinal: slot.ordinal,
                attrs: slot.source.attrs.clone(),
            })
            .collect();
        let adoptable = previous.filter(|old| old.header == header && old.config == config);
        match (previous, adoptable) {
            (_, Some(_)) => stats.header.adopted = 1,
            (Some(old), None) => {
                stats.header.computed = 1;
                stats.header.cancelled = 1;
                old.token.cancel();
            }
            (None, None) => stats.header.computed = 1,
        }
        let mut blocks = Vec::with_capacity(slots.len());
        for (index, slot) in slots.into_iter().enumerate() {
            let old = adoptable.map(|old| &old.blocks[index]);
            let block_token = token.child();
            let (snapshot, region_tokens) = match old {
                Some(old) if old.snapshot.source == slot.source => {
                    stats.blocks.adopted += 1;
                    let region_tokens = old
                        .region_tokens
                        .iter()
                        .map(|_| block_token.child())
                        .collect();
                    (old.snapshot.clone(), region_tokens)
                }
                _ => {
                    if let Some(old) = old {
                        old.token.cancel();
                        stats.blocks.cancelled += 1;
                    }
                    stats.blocks.computed += 1;
                    let old = old.map(|old| &old.snapshot);
                    let (computed, region_tokens) =
                        compute_block(slot.source, old, config, stages, &block_token, &mut stats)?;
                    (Shared::new(computed), region_tokens)
                }
            };
            blocks.push(BlockEntry {
                kind: slot.kind,
                ordinal: slot.ordinal,
                start: slot.start,
                token: block_token,
                region_tokens,
                snapshot,
            });
        }
        let tree = Self {
            header,
            config,
            token,
            blocks,
        };
        Ok((tree, stats))
    }

    /// Every artifact of every block, as TS-42 compares them.
    #[must_use]
    pub fn artifacts(&self) -> Vec<BlockArtifacts> {
        self.blocks
            .iter()
            .map(|entry| BlockArtifacts {
                kind: entry.kind.clone(),
                ordinal: entry.ordinal,
                start: entry.start,
                source_key: entry.snapshot.source.key,
                surface: entry.snapshot.surface.clone(),
                page: entry.snapshot.page.clone(),
            })
            .collect()
    }

    /// The file task's token.
    #[must_use]
    pub fn token(&self) -> &CancelToken {
        &self.token
    }

    /// Every task token of this tree in document order: the file's, then
    /// each block's followed by its regions'.
    #[must_use]
    pub fn task_tokens(&self) -> Vec<CancelToken> {
        let mut tokens = vec![self.token.clone()];
        for entry in &self.blocks {
            tokens.push(entry.token.clone());
            tokens.extend(entry.region_tokens.iter().cloned());
        }
        tokens
    }
}

/// A changed block's task: S1 whole, S2 by regions when the template splits.
fn compute_block(
    source: BlockSource,
    old: Option<&Shared<BlockSnapshot>>,
    config: StageConfig,
    stages: &Stages,
    token: &CancelToken,
    stats: &mut SnapshotStats,
) -> Result<(BlockSnapshot, Vec<CancelToken>), Cancelled> {
    token.check()?;
    let surface = (stages.surface)(&source);
    let decomposed = (source.kind == BlockKind::Template && surface.is_some())
        .then(|| split_regions(source.text.as_str()))
        .flatten();
    let Some(syntaxes) = decomposed else {
        token.check()?;
        let page = (stages.page)(&source, config);
        let snapshot = BlockSnapshot {
            source,
            surface,
            page,
            regions: Vec::new(),
        };
        return Ok((snapshot, Vec::new()));
    };
    let old_regions = old.map_or(&[][..], |old| old.regions.as_slice());
    let caps = LegacyCaps::for_version(config.vue_version);
    let mut regions: Vec<_> = Vec::with_capacity(syntaxes.len());
    let mut tokens = Vec::with_capacity(syntaxes.len());
    let mut used = vec![false; old_regions.len()];
    for syntax in syntaxes {
        token.check()?;
        let found = old_regions
            .iter()
            .enumerate()
            .find(|(index, old)| !used[*index] && old.syntax.text == syntax.text);
        match found {
            Some((index, old)) => {
                used[index] = true;
                stats.regions.adopted += 1;
                tokens.push(token.child());
                if old.syntax.start == syntax.start {
                    regions.push(old.clone());
                } else {
                    // The same bytes moved inside the block: adopt the
                    // lowering with its spans moved, no stage work.
                    let lowering = shifted(&old.lowering, old.syntax.start, syntax.start);
                    regions.push(Shared::new(RegionSnapshot { syntax, lowering }));
                }
            }
            None => {
                stats.regions.computed += 1;
                let region_token = token.child();
                let lowering = (stages.region)(source.text.as_str(), &syntax, caps);
                region_token.check()?;
                tokens.push(region_token);
                regions.push(Shared::new(RegionSnapshot { syntax, lowering }));
            }
        }
    }
    // The replaced block's token was cancelled by the caller, which cascaded
    // to every old region task; the ones not adopted are the cancelled work.
    stats.regions.cancelled += used.iter().filter(|used| !**used).count() as u32;
    let (folio, diagnostics) = assemble(regions.iter().map(|region| &region.lowering));
    let page = PageArtifact {
        key: ArtifactKey::of(&folio, 0),
        folio,
        diagnostics,
    };
    let snapshot = BlockSnapshot {
        source,
        surface,
        page: Some(page),
        regions,
    };
    Ok((snapshot, tokens))
}
