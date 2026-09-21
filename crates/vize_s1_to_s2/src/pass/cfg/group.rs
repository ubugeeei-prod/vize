//! [`ComplexityFacts`] as a P4-1a fact group: the typed query surface every
//! consumer reads template complexity through (charter #5/#8).
//!
//! The artifact is one template block's text (`str`); the table
//! holds exactly one row, keyed by `()`, with **block-relative** spans. A
//! consumer that knows where the block sits in its file shifts the spans
//! ([`ComplexityFacts::shifted`]). Consumers declare the group in their
//! [`FactConsumer::DEMAND`](vize_davinci::fact::FactConsumer::DEMAND), so a
//! read outside that declaration trips the TS-35 detector.

use vize_davinci::fact::{
    Demand, FactGroup, FactProducer, FactRegistry, FactTable, FactView, ProducerEntry,
};
use vize_davinci::pass::AnalysisId;
use vize_s0::{Allocator, Span};

use super::ComplexityFacts;
use crate::lower::lower;

/// The template-complexity fact group.
///
/// Id 48: fact-group ids are declared by the crate that computes them
/// (`pass::preserved`), and the S2 analyses of this crate take the upper
/// quarter of the 64-id space so they never meet the pass manager's own
/// low ids or the Croquis groups the P4-3 waves register.
pub struct TemplateComplexityGroup;

impl FactGroup for TemplateComplexityGroup {
    const ID: AnalysisId = AnalysisId::new(48);
    const NAME: &'static str = "template-complexity";
    const STRATUM: u8 = 0;
    const DEPENDS: Demand = Demand::NONE;
    type Key = ();
    type Value = ComplexityFacts;
}

impl FactProducer<str> for TemplateComplexityGroup {
    fn produce(template: &str, _: &FactView<'_>) -> FactTable<Self> {
        let allocator = Allocator::new();
        let (tree, errors) = vize_s1::parse(&allocator, template);
        let lowered = lower(&allocator, &tree, &errors);
        [((), super::run(&lowered))].into_iter().collect()
    }
}

/// The registry template-complexity consumers compute through
/// (stratification-checked at compile time).
pub const TEMPLATE_FACTS: FactRegistry<str> =
    FactRegistry::new(&[ProducerEntry::of::<TemplateComplexityGroup>()]);

impl ComplexityFacts {
    /// The same facts with every span moved `offset` bytes forward: a
    /// block-relative table placed in its file.
    #[must_use]
    pub fn shifted(mut self, offset: u32) -> Self {
        for row in &mut self.contributions {
            row.span = Span::new(
                row.span.start.saturating_add(offset),
                row.span.end.saturating_add(offset),
            );
        }
        self
    }
}
