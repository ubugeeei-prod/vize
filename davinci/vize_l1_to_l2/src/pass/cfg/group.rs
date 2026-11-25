//! [`ComplexityAnalysis`] as a P4-1a fact group: the typed query surface every
//! consumer reads template complexity through (charter #5/#8).
//!
//! The artifact is one template block's text (`str`); the table
//! holds exactly one row, keyed by `()`, with **block-relative** spans. A
//! consumer that knows where the block sits in its file shifts the spans
//! ([`ComplexityAnalysis::shifted`]). Consumers declare the group in their
//! [`AnalysisConsumer::DEMAND`](vize_l0::analysis::AnalysisConsumer::DEMAND), so a
//! read outside that declaration trips the TS-35 detector.

use vize_l0::Allocator;
use vize_l0::analysis::{
    Analysis, AnalysisProducer, AnalysisRegistry, AnalysisTable, AnalysisView, Demand,
    ProducerEntry,
};
use vize_l0::pass::AnalysisId;

use super::ComplexityAnalysis;
use crate::lower::lower;

/// The template-complexity fact group.
///
/// Id 48: fact-group ids are declared by the crate that computes them
/// (`pass::preserved`), and the L2 analyses of this crate take the upper
/// quarter of the 64-id space so they never meet the pass manager's own
/// low ids or the Croquis groups the P4-3 waves register.
pub struct TemplateComplexityGroup;

impl Analysis for TemplateComplexityGroup {
    const ID: AnalysisId = AnalysisId::new(48);
    const NAME: &'static str = "template-complexity";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = ();
    type Value = ComplexityAnalysis;
}

impl AnalysisProducer<str> for TemplateComplexityGroup {
    fn produce(template: &str, _: &AnalysisView<'_>) -> AnalysisTable<Self> {
        let allocator = Allocator::new();
        let (tree, errors) = vize_l1::markup::parse(&allocator, template);
        let lowered = lower(&allocator, &tree, &errors);
        [((), super::run(&lowered))].into_iter().collect()
    }
}

/// The registry template-complexity consumers compute through
/// (stratification-checked at compile time).
pub const TEMPLATE_FACTS: AnalysisRegistry<str> =
    AnalysisRegistry::new(&[ProducerEntry::of::<TemplateComplexityGroup>()]);
