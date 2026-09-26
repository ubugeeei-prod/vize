//! Rules that judge Davinci facts rather than walking a syntax tree.
//!
//! Each rule reads facts produced by a Davinci analysis (an L2 pass or a
//! fact group) and reports a property of those facts, so its precision is
//! the precision of the analysis, stated as a tier in its docs.

mod max_template_complexity;
mod unused_setup_bindings;

pub use max_template_complexity::MaxTemplateComplexity;
pub use unused_setup_bindings::NoUnusedSetupBindings;

use vize_davinci::fact::FactConsumer;

use crate::rule::RuleRegistry;

/// Register the fact rules a project enables by name.
///
/// `vue/max-template-complexity` belongs to no preset: its thresholds are
/// the corpus p95, so about one real component in twenty exceeds them, and
/// a preset that suddenly warned on those would churn every project that
/// adopted it (charter #23). A project opts in by naming the rule.
pub(crate) fn register_opt_in(registry: &mut RuleRegistry) {
    if !registry.has_rule(NoUnusedSetupBindings::NAME) {
        registry.register(Box::new(NoUnusedSetupBindings));
    }
    if !registry.has_rule(MaxTemplateComplexity::NAME) {
        registry.register(Box::new(MaxTemplateComplexity));
    }
}
