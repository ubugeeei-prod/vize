//! Conversion-owned entry points for L2 template-complexity facts.
//!
//! The backend-neutral analysis lives in
//! [`vize_l2::analysis::template_complexity`]. This module keeps the
//! lowering adapter and source fact registry beside the L1 to L2 conversion.

use crate::lower::Lowered;

pub mod group;
mod source;

pub use group::{TEMPLATE_FACTS, TemplateComplexityGroup};
pub use source::template_facts;
pub use vize_l2::analysis::template_complexity::{
    COGNITIVE_WARN_ABOVE, CYCLOMATIC_WARN_ABOVE, ComplexityFacts, Contribution, DESC, DecisionKind,
    NAME, print_facts,
};

/// Run the neutral L2 analysis over one transitional lowering result.
///
/// # Panics
///
/// Panics only when the L2 walk and the lowering's minted node count diverge.
#[must_use]
pub fn run(lowered: &Lowered<'_>) -> ComplexityFacts {
    vize_l2::analysis::template_complexity::run(&lowered.root, lowered.op_count)
}
