//! Fact groups over a [`PartialPage`](super::page::PartialPage).
//!
//! Holes and regions share stratum 0. Scope names sit at stratum 1 and are
//! a fact only when their introducing fragment was accepted as a region.

use vize_l0::Span;
use vize_l0::analysis::{
    Analysis, AnalysisConsumer, AnalysisProducer, AnalysisRegistry, AnalysisTable, AnalysisView,
    Demand, ProducerEntry, ids,
};
use vize_l0::pass::AnalysisId;

use super::page::PartialPage;
use super::{FragmentKind, HoleInfo, HoleKind, RegionInfo, ScopeKey, ScopeNameInfo, ScopeRole};

pub(crate) struct PartialHoles;
pub(crate) struct PartialRegions;
pub(crate) struct PartialScopes;

pub(crate) struct PartialTemplateAnalysis;

impl Analysis for PartialHoles {
    const ID: AnalysisId = ids::PARTIAL_HOLES;
    const NAME: &'static str = "partial-holes";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = HoleInfo;
}

impl Analysis for PartialRegions {
    const ID: AnalysisId = ids::PARTIAL_REGIONS;
    const NAME: &'static str = "partial-regions";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = RegionInfo;
}

impl Analysis for PartialScopes {
    const ID: AnalysisId = ids::PARTIAL_SCOPES;
    const NAME: &'static str = "partial-scopes";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE
        .with(ids::PARTIAL_HOLES)
        .with(ids::PARTIAL_REGIONS);
    type Key = ScopeKey;
    type Value = ScopeNameInfo;
}

impl AnalysisProducer<PartialPage> for PartialHoles {
    fn produce(page: &PartialPage, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        page.holes
            .iter()
            .enumerate()
            .map(|(index, hole)| (u32::try_from(index).unwrap_or(u32::MAX), *hole))
            .collect()
    }
}

impl AnalysisProducer<PartialPage> for PartialRegions {
    fn produce(page: &PartialPage, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        page.fragments
            .iter()
            .filter(|fragment| !covered(fragment.span, &page.holes))
            .map(|fragment| {
                (
                    fragment.node,
                    RegionInfo {
                        span: fragment.span,
                        kind: fragment.kind,
                    },
                )
            })
            .collect()
    }
}

impl AnalysisProducer<PartialPage> for PartialScopes {
    fn produce(page: &PartialPage, inputs: &AnalysisView<'_>) -> AnalysisTable<Self> {
        let (Ok(regions), Ok(holes)) =
            (inputs.get::<PartialRegions>(), inputs.get::<PartialHoles>())
        else {
            return AnalysisTable::default();
        };
        page.names
            .iter()
            .filter_map(|name| {
                let region = regions.get(&name.node)?;
                let fragment = page.fragments.iter().find(|item| item.node == name.node)?;
                if name_in_hole(name.span, holes) {
                    return None;
                }
                Some((
                    ScopeKey {
                        node: name.node,
                        index: name.index,
                    },
                    ScopeNameInfo {
                        name: name.name.clone(),
                        name_span: name.span,
                        visible: fragment.visible,
                        role: role_of(region.kind, name.index),
                    },
                ))
            })
            .collect()
    }
}

impl AnalysisConsumer for PartialTemplateAnalysis {
    const NAME: &'static str = "partial-template";
    const DEMAND: Demand = Demand::NONE
        .with(ids::PARTIAL_HOLES)
        .with(ids::PARTIAL_REGIONS)
        .with(ids::PARTIAL_SCOPES);
}

pub(crate) static REGISTRY: AnalysisRegistry<PartialPage> = AnalysisRegistry::new(&[
    ProducerEntry::of::<PartialHoles>(),
    ProducerEntry::of::<PartialRegions>(),
    ProducerEntry::of::<PartialScopes>(),
]);

fn role_of(kind: FragmentKind, index: u16) -> ScopeRole {
    match kind {
        FragmentKind::For => match index {
            0 => ScopeRole::Value,
            1 => ScopeRole::Key,
            _ => ScopeRole::Index,
        },
        FragmentKind::Binding => ScopeRole::Slot,
        _ => ScopeRole::Value,
    }
}

fn covered(span: Span, holes: &[HoleInfo]) -> bool {
    holes
        .iter()
        .any(|hole| hole.kind == HoleKind::Unexpected && covers(hole.span, span))
}

fn name_in_hole(span: Span, holes: &AnalysisTable<PartialHoles>) -> bool {
    !span.is_empty()
        && holes
            .iter()
            .any(|(_, hole)| hole.kind == HoleKind::Unexpected && covers(hole.span, span))
}

fn covers(hole: Span, span: Span) -> bool {
    !hole.is_empty() && hole.start <= span.start && span.end <= hole.end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower::lower;
    use crate::partial::page::collect;
    use vize_l0::Allocator;
    use vize_l0::analysis::{AnalysisManager, produced_count};
    use vize_l1::markup::parse;

    #[test]
    fn a_page_produces_each_partial_group_once() {
        let source = "<p></stray><i v-for=\"item in items\">{{ item }}</i>";
        let allocator = Allocator::new();
        let (tree, errors) = parse(&allocator, source);
        let mut lowered = lower(&allocator, &tree, &errors);
        let page = collect(&mut lowered, &tree);
        let before_holes = produced_count(ids::PARTIAL_HOLES);
        let before_regions = produced_count(ids::PARTIAL_REGIONS);
        let before_scopes = produced_count(ids::PARTIAL_SCOPES);
        let mut manager = AnalysisManager::new(&REGISTRY);
        manager
            .prepare::<PartialTemplateAnalysis>(&page)
            .expect("closed");
        manager
            .prepare::<PartialTemplateAnalysis>(&page)
            .expect("closed");
        assert_eq!(produced_count(ids::PARTIAL_HOLES), before_holes + 1);
        assert_eq!(produced_count(ids::PARTIAL_REGIONS), before_regions + 1);
        assert_eq!(produced_count(ids::PARTIAL_SCOPES), before_scopes + 1);
    }
}
