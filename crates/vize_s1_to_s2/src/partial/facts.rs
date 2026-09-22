//! Fact groups over a [`PartialPage`](super::page::PartialPage).
//!
//! Holes and regions share stratum 0. Scope names sit at stratum 1 and are
//! a fact only when their introducing fragment was accepted as a region.

use vize_davinci::fact::{
    ids, Demand, FactConsumer, FactGroup, FactProducer, FactRegistry, FactTable, FactView,
    ProducerEntry,
};
use vize_davinci::pass::AnalysisId;
use vize_s0::Span;

use super::page::PartialPage;
use super::{FragmentKind, HoleFact, HoleKind, RegionFact, ScopeKey, ScopeNameFact, ScopeRole};

pub(crate) struct PartialHoles;
pub(crate) struct PartialRegions;
pub(crate) struct PartialScopes;

pub(crate) struct TemplateFacts;

impl FactGroup for PartialHoles {
    const ID: AnalysisId = ids::PARTIAL_HOLES;
    const NAME: &'static str = "partial-holes";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = HoleFact;
}

impl FactGroup for PartialRegions {
    const ID: AnalysisId = ids::PARTIAL_REGIONS;
    const NAME: &'static str = "partial-regions";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = u32;
    type Value = RegionFact;
}

impl FactGroup for PartialScopes {
    const ID: AnalysisId = ids::PARTIAL_SCOPES;
    const NAME: &'static str = "partial-scopes";
    const STRATUM: u8 = 1;
    const DEPENDS: Demand = Demand::NONE
        .with(ids::PARTIAL_HOLES)
        .with(ids::PARTIAL_REGIONS);
    type Key = ScopeKey;
    type Value = ScopeNameFact;
}

impl FactProducer<PartialPage> for PartialHoles {
    fn produce(page: &PartialPage, _: &FactView<'_>) -> FactTable<Self> {
        page.holes
            .iter()
            .enumerate()
            .map(|(index, hole)| (u32::try_from(index).unwrap_or(u32::MAX), *hole))
            .collect()
    }
}

impl FactProducer<PartialPage> for PartialRegions {
    fn produce(page: &PartialPage, _: &FactView<'_>) -> FactTable<Self> {
        page.fragments
            .iter()
            .filter(|fragment| !covered(fragment.span, &page.holes))
            .map(|fragment| {
                (
                    fragment.node,
                    RegionFact {
                        span: fragment.span,
                        kind: fragment.kind,
                    },
                )
            })
            .collect()
    }
}

impl FactProducer<PartialPage> for PartialScopes {
    fn produce(page: &PartialPage, inputs: &FactView<'_>) -> FactTable<Self> {
        let regions = inputs.get::<PartialRegions>().expect("declared");
        let holes = inputs.get::<PartialHoles>().expect("declared");
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
                    ScopeNameFact {
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

impl FactConsumer for TemplateFacts {
    const NAME: &'static str = "partial-template";
    const DEMAND: Demand = Demand::NONE
        .with(ids::PARTIAL_HOLES)
        .with(ids::PARTIAL_REGIONS)
        .with(ids::PARTIAL_SCOPES);
}

pub(crate) static REGISTRY: FactRegistry<PartialPage> = FactRegistry::new(&[
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

fn covered(span: Span, holes: &[HoleFact]) -> bool {
    holes
        .iter()
        .any(|hole| hole.kind == HoleKind::Unexpected && covers(hole.span, span))
}

fn name_in_hole(span: Span, holes: &FactTable<PartialHoles>) -> bool {
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
    use vize_davinci::fact::{produced_count, FactManager};
    use vize_s0::Allocator;
    use vize_s1::parse;

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
        let mut manager = FactManager::new(&REGISTRY);
        manager.prepare::<TemplateFacts>(&page).expect("closed");
        manager.prepare::<TemplateFacts>(&page).expect("closed");
        assert_eq!(produced_count(ids::PARTIAL_HOLES), before_holes + 1);
        assert_eq!(produced_count(ids::PARTIAL_REGIONS), before_regions + 1);
        assert_eq!(produced_count(ids::PARTIAL_SCOPES), before_scopes + 1);
    }
}
