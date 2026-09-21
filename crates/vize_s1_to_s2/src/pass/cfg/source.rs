//! The consumer entry: complexity facts of one template block inside a
//! larger authored file (an SFC), with file-absolute spans, and the
//! default thresholds the metric spec pins.
//!
//! Lint (`vue/max-template-complexity`) and cross-file analysis
//! (`vize_croquis_cf`) both start from a file and a template byte range;
//! this is the one place that turns that pair into facts, so neither
//! consumer re-derives the S1 → S2 → pass pipeline.

use vize_s0::{Allocator, SourceRoot};

use super::ComplexityFacts;
use crate::lower::lower_source_block;

/// Default warning threshold on own cyclomatic complexity: a component
/// warns when its cyclomatic complexity is **strictly above** this value
/// (the full-corpus p95, `complexity-metrics.md`).
pub const CYCLOMATIC_WARN_ABOVE: u32 = 11;

/// Default warning threshold on own cognitive complexity, strictly above
/// (the full-corpus p95, `complexity-metrics.md`).
pub const COGNITIVE_WARN_ABOVE: u32 = 16;

/// Facts of the template occupying `source[start..end]`, spans relative to
/// `source`; `None` when the range is not a valid slice of `source`.
///
/// Parses and lowers the block into a private arena, runs the pass, and
/// returns the owned facts (the arena is dropped before returning).
#[must_use]
pub fn run_template_range(source: &str, start: u32, end: u32) -> Option<ComplexityFacts> {
    let template = source.get(start as usize..end as usize)?;
    let root = SourceRoot::new(source).ok()?;
    let block = root.block(template, start).ok()?;
    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, template);
    let lowered = lower_source_block(&allocator, &tree, &errors, block);
    Some(super::run(&lowered))
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
    use super::{COGNITIVE_WARN_ABOVE, CYCLOMATIC_WARN_ABOVE, run_template_range};

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
        let facts = run_template_range(
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
        assert_eq!(run_template_range("<p/>", 2, 9), None);
    }
}
