//! The consumer entry: complexity facts of one template block inside a
//! larger authored file (an SFC), with file-absolute spans, and the
//! default thresholds the metric spec pins.
//!
//! Lint (`vue/max-template-complexity`) and cross-file analysis
//! (`vize_croquis_cf`) both start from a file and a template byte range;
//! this is the one place that turns that pair into facts, so neither
//! consumer re-derives the S1 → S2 → pass pipeline. The facts are read
//! through the P4-1a fact API ([`super::group`]) under the consumer's own
//! declared demand.

use vize_davinci::fact::{FactConsumer, FactManager};

use super::ComplexityFacts;
use super::group::{TEMPLATE_FACTS, TemplateComplexityGroup};

/// Default warning threshold on own cyclomatic complexity: a component
/// warns when its cyclomatic complexity is **strictly above** this value
/// (the full-corpus p95, `complexity-metrics.md`).
pub const CYCLOMATIC_WARN_ABOVE: u32 = 11;

/// Default warning threshold on own cognitive complexity, strictly above
/// (the full-corpus p95, `complexity-metrics.md`).
pub const COGNITIVE_WARN_ABOVE: u32 = 16;

/// Consumer `C`'s read of the facts of the template occupying
/// `source[start..end]`, spans relative to `source`; `None` when the range
/// is not a valid slice of `source`.
///
/// Computes the [`TemplateComplexityGroup`] for the block through a fact
/// manager, reads it through `C`'s declared view, and places the owned
/// facts in the file.
///
/// # Panics
///
/// Panics when `C` demands a group the template registry does not compute,
/// or does not declare the complexity group (debug builds): a consumer
/// declaration bug, never an input property.
#[must_use]
pub fn template_facts<C: FactConsumer>(
    source: &str,
    start: u32,
    end: u32,
) -> Option<ComplexityFacts> {
    let template = source.get(start as usize..end as usize)?;
    let mut manager = FactManager::new(&TEMPLATE_FACTS);
    let view = manager
        .prepare::<C>(template)
        .expect("the template registry computes every group a template consumer demands");
    let table = view
        .get::<TemplateComplexityGroup>()
        .expect("a template-complexity consumer declares the complexity group");
    table.get(&()).cloned().map(|facts| facts.shifted(start))
}

impl ComplexityFacts {
    /// Whether the own scores exceed the default thresholds.
    #[must_use]
    pub const fn exceeds_default_thresholds(&self) -> bool {
        self.cyclomatic > CYCLOMATIC_WARN_ABOVE || self.cognitive > COGNITIVE_WARN_ABOVE
    }
}

#[cfg(test)]
mod tests {
    use super::super::group::TemplateComplexityGroup;
    use super::{COGNITIVE_WARN_ABOVE, CYCLOMATIC_WARN_ABOVE, template_facts};
    use vize_davinci::fact::{Demand, FactConsumer, FactGroup};

    struct Probe;
    impl FactConsumer for Probe {
        const NAME: &'static str = "cfg-source-probe";
        const DEMAND: Demand = Demand::NONE.with(TemplateComplexityGroup::ID);
    }

    #[test]
    fn the_thresholds_are_the_ones_the_spec_pins() {
        let spec = include_str!("../../../../../davinci-road/plan/complexity-metrics.md");
        let cyclomatic = alloc::format!("cyclomatic complexity exceeds {CYCLOMATIC_WARN_ABOVE}**");
        let cognitive = alloc::format!("cognitive\ncomplexity exceeds {COGNITIVE_WARN_ABOVE}**");
        assert!(spec.contains(cyclomatic.as_str()), "{cyclomatic}");
        assert!(spec.contains(cognitive.as_str()), "{cognitive}");
    }

    #[test]
    fn a_block_keeps_file_absolute_spans() {
        let source = "<script>x</script>\n<template><p v-if=\"a\">x</p></template>";
        let start = source.find("<template>").expect("template") + "<template>".len();
        let end = source.find("</template>").expect("close");
        let facts = template_facts::<Probe>(
            source,
            u32::try_from(start).expect("small"),
            u32::try_from(end).expect("small"),
        )
        .expect("the range is a slice");
        let row = facts.contributions.first().expect("one v-if row");
        assert_eq!(row.span.slice(source), "a");
        assert_eq!((facts.cyclomatic, facts.cognitive), (2, 1));
        assert!(!facts.exceeds_default_thresholds());
    }

    #[test]
    fn a_range_outside_the_source_is_refused() {
        assert_eq!(template_facts::<Probe>("<p/>", 2, 9), None);
    }
}
