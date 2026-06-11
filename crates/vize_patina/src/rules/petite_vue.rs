//! petite-vue dialect lint rules.
//!
//! These rules only fire on documents detected as petite-vue (via
//! `ctx.is_petite_vue()`); they are opt-in and belong to no preset, so they
//! never affect normal Vue SFC linting.

mod no_unsupported_directive;
mod valid_v_scope;

use crate::rule::{Rule, RuleRegistry};

pub use no_unsupported_directive::NoUnsupportedDirective;
pub use valid_v_scope::ValidVScope;

/// Register petite-vue rules as explicit opt-in rules.
pub(crate) fn register_opt_in(registry: &mut RuleRegistry) {
    register_if_missing(registry, Box::new(NoUnsupportedDirective));
    register_if_missing(registry, Box::new(ValidVScope));
}

fn register_if_missing(registry: &mut RuleRegistry, rule: Box<dyn Rule>) {
    if !registry.has_rule(rule.meta().name) {
        registry.register(rule);
    }
}
