//! Rule-name sets that gate shared analysis work in the lint engine.
//!
//! A rule must appear in a set to receive the work that set unlocks:
//!
//! - [`SEMANTIC_TEMPLATE_RULES`]: rules whose `run_on_template` pass needs the
//!   template semantic analysis (`LintContext::has_analysis`). Omitting a rule
//!   here means its template pass returns before the rule body runs.
//! - [`SHARED_SFC_DESCRIPTOR_RULES`]: rules that consume SFC descriptor
//!   metadata, so the outer SFC path parses the descriptor once up front
//!   instead of letting each rule re-parse the file.

pub(super) const SEMANTIC_TEMPLATE_RULES: &[&str] = &[
    "vue/no-unused-vars",
    "vue/no-unused-components",
    "vue/require-component-registration",
    "vue/no-undefined-refs",
    "vue/no-mutating-props",
    "vue/no-unused-properties",
    "vue/prop-name-casing",
    "a11y/no-refer-to-non-existent-id",
    "ecosystem/router-link-require-to",
];

pub(super) const SHARED_SFC_DESCRIPTOR_RULES: &[&str] = &[
    "vue/no-mutating-props",
    "vue/no-reserved-component-names",
    "vue/no-unused-properties",
    "vue/no-unused-refs",
    "vue/prop-name-casing",
    "vue/sfc-element-order",
    "vue/require-scoped-style",
    "vue/single-style-block",
    "vue/warn-custom-block",
    "ecosystem/void-link-require-href",
    "ecosystem/void-link-valid-method",
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{SEMANTIC_TEMPLATE_RULES, SHARED_SFC_DESCRIPTOR_RULES};
    use crate::rule::RuleRegistry;

    #[test]
    fn engine_rule_name_sets_only_name_dispatchable_rules() {
        let mut available: BTreeSet<_> = RuleRegistry::with_all()
            .rule_names()
            .iter()
            .copied()
            .collect();
        available.extend(
            RuleRegistry::with_opt_in_rules()
                .rule_names()
                .iter()
                .copied(),
        );
        let missing: Vec<&str> = SEMANTIC_TEMPLATE_RULES
            .iter()
            .chain(SHARED_SFC_DESCRIPTOR_RULES)
            .copied()
            .filter(|name| !available.contains(name))
            .collect();
        assert_eq!(
            missing,
            Vec::<&str>::new(),
            "engine rule-name sets must only name rules a registry can instantiate"
        );
    }
}
