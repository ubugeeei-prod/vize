//! Rules that judge Davinci facts rather than walking a syntax tree.
//!
//! Each rule reads facts produced by a Davinci analysis (an S2 pass or a
//! fact group) and reports a property of those facts, so its precision is
//! the precision of the analysis, stated as a tier in its docs.

mod max_template_complexity;

pub use max_template_complexity::MaxTemplateComplexity;

use crate::rule::RuleRegistry;

/// Register the fact rules the opinionated preset carries.
pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(MaxTemplateComplexity));
}
