//! Mapping between host-config category names and the rules they cover.
//!
//! Config exposes a small set of coarse categories (`correctness`, `style`,
//! `a11y`, `security`, `perf`, `suspicious`); these helpers decide whether a
//! given rule belongs to a category, combining the rule's [`RuleCategory`] with
//! explicit per-rule name lists where the category does not line up 1:1.

use crate::rule::RuleCategory;

pub(super) fn rule_matches_config_category(
    rule_name: &str,
    rule_category: RuleCategory,
    config_category: &str,
) -> bool {
    match config_category {
        "correctness" => matches!(rule_category, RuleCategory::Essential),
        "style" => {
            is_style_rule_name(rule_name)
                || matches!(rule_category, RuleCategory::StronglyRecommended)
        }
        "a11y" => matches!(rule_category, RuleCategory::Accessibility),
        "security" => is_security_rule_name(rule_name),
        "perf" => is_perf_rule_name(rule_name),
        "suspicious" => {
            matches!(
                rule_category,
                RuleCategory::Recommended | RuleCategory::HtmlConformance | RuleCategory::Ecosystem
            ) && !is_style_rule_name(rule_name)
                && !is_perf_rule_name(rule_name)
                && !is_security_rule_name(rule_name)
        }
        _ => false,
    }
}

fn is_style_rule_name(rule_name: &str) -> bool {
    matches!(
        rule_name,
        "vue/attribute-hyphenation"
            | "vue/attribute-order"
            | "vue/component-definition-name-casing"
            | "vue/component-name-in-template-casing"
            | "vue/html-quotes"
            | "vue/html-self-closing"
            | "vue/multi-word-component-names"
            | "vue/mustache-interpolation-spacing"
            | "vue/no-inline-style"
            | "vue/no-multi-spaces"
            | "vue/prefer-props-shorthand"
            | "vue/prefer-true-attribute-shorthand"
            | "vue/prop-name-casing"
            | "vue/require-scoped-style"
            | "vue/sfc-element-order"
            | "vue/single-style-block"
            | "vue/v-bind-style"
            | "vue/v-on-style"
            | "vue/v-slot-style"
            | "css/no-id-selectors"
            | "css/no-important"
            | "css/no-utility-classes"
            | "css/prefer-logical-properties"
            | "css/prefer-nested-selectors"
            | "css/prefer-slotted"
    )
}

fn is_security_rule_name(rule_name: &str) -> bool {
    matches!(
        rule_name,
        "vue/no-v-html"
            | "vue/no-unsafe-url"
            | "vue/no-unsandboxed-iframe"
            | "ssr/no-browser-globals-in-ssr"
            | "ssr/no-hydration-mismatch"
    )
}

fn is_perf_rule_name(rule_name: &str) -> bool {
    matches!(
        rule_name,
        "css/no-v-bind-performance"
            | "script/no-async-in-computed"
            | "script/no-next-tick"
            | "script/no-top-level-ref-in-script"
            | "type/no-floating-promises"
            | "type/no-reactivity-loss"
    )
}
