//! Vue ecosystem lint rules.
//!
//! These rules cover the conventions that make first-party and de-facto Vue
//! ecosystem packages feel native in editors and production builds. They stay in
//! Rust so hosts can share one fast rule implementation across the CLI, LSP, and
//! JavaScript bindings.

mod i18n_no_missing_key;
mod nuxt_prefer_nuxt_link;
mod router_link_require_to;
mod void_link_require_href;
mod void_link_valid_method;
mod vue_router_prefer_named_link;

use crate::rule::RuleRegistry;

pub use i18n_no_missing_key::VueI18nNoMissingKey;
pub use nuxt_prefer_nuxt_link::NuxtPreferNuxtLink;
pub use router_link_require_to::RouterLinkRequireTo;
pub use void_link_require_href::VoidLinkRequireHref;
pub use void_link_valid_method::VoidLinkValidMethod;
pub use vue_router_prefer_named_link::VueRouterPreferNamedLink;

pub(crate) const TEMPLATE_RULE_COUNT: usize = 6;

pub(crate) fn register(registry: &mut RuleRegistry) {
    register_if_missing(registry, Box::new(RouterLinkRequireTo));
    register_if_missing(registry, Box::new(VueRouterPreferNamedLink));
    register_if_missing(registry, Box::new(NuxtPreferNuxtLink));
    register_if_missing(registry, Box::new(VueI18nNoMissingKey));
    register_if_missing(registry, Box::new(VoidLinkRequireHref));
    register_if_missing(registry, Box::new(VoidLinkValidMethod));
}

pub(crate) fn register_opt_in(registry: &mut RuleRegistry) {
    register(registry);
}

fn register_if_missing(registry: &mut RuleRegistry, rule: Box<dyn crate::rule::Rule>) {
    if !registry.has_rule(rule.meta().name) {
        registry.register(rule);
    }
}
