//! A changed block's task: S1 whole, S2 by regions when the template splits.

use vize_davinci::key::ArtifactKey;
use vize_s1_to_s2::LegacyCaps;

use super::cancel::{CancelToken, Cancelled};
use super::region::{assemble, split_regions};
use super::shift::shifted;
use super::{BlockSnapshot, RegionSnapshot, Shared, SnapshotStats, Stages};
use crate::artifact::{BlockKind, BlockSource, PageArtifact, StageConfig, SurfaceArtifact};

/// A changed block's task: S1 whole, S2 by regions when the template splits.
pub(super) fn compute_block(
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
        return Ok((whole_block(source, surface, config, stages), Vec::new()));
    };
    let old_regions = old.map_or(&[][..], |old| old.regions.as_slice());
    let caps = LegacyCaps::for_version(config.vue_version);
    let mut regions: Vec<_> = Vec::with_capacity(syntaxes.len());
    let mut tokens = Vec::with_capacity(syntaxes.len());
    let mut used = vec![false; old_regions.len()];
    for syntax in syntaxes {
        token.check()?;
        let found = old_regions.iter().enumerate().find(|(index, old)| {
            used.get(*index) == Some(&false) && old.syntax.text == syntax.text
        });
        match found {
            Some((index, old)) => {
                if let Some(flag) = used.get_mut(index) {
                    *flag = true;
                }
                stats.regions.adopted += 1;
                tokens.push(token.child());
                if old.syntax.start == syntax.start {
                    regions.push(old.clone());
                } else {
                    // The same bytes moved inside the block: adopt the
                    // lowering with its spans moved, no stage work.
                    let Some(lowering) = shifted(&old.lowering, old.syntax.start, syntax.start)
                    else {
                        return Ok((whole_block(source, surface, config, stages), Vec::new()));
                    };
                    regions.push(Shared::new(RegionSnapshot { syntax, lowering }));
                }
            }
            None => {
                stats.regions.computed += 1;
                let region_token = token.child();
                let Some(lowering) = (stages.region)(source.text.as_str(), &syntax, caps) else {
                    // A region that cannot be framed in its block lowers whole.
                    return Ok((whole_block(source, surface, config, stages), Vec::new()));
                };
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

/// A block lowered whole: no regions, S2 straight from the page stage.
fn whole_block(
    source: BlockSource,
    surface: Option<SurfaceArtifact>,
    config: StageConfig,
    stages: &Stages,
) -> BlockSnapshot {
    let page = (stages.page)(&source, config);
    BlockSnapshot {
        source,
        surface,
        page,
        regions: Vec::new(),
    }
}
