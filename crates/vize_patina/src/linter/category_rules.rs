//! Mapping between host-config category names and the rules they cover.
//!
//! Config exposes a small set of coarse categories (`correctness`, `style`,
//! `a11y`, `security`, `perf`, `suspicious`); these helpers decide whether a
//! given rule belongs to a category, combining the rule's [`RuleCategory`] with
//! explicit per-rule name lists where the category does not line up 1:1.

use crate::rule::RuleCategory;

/// Match categories that are defined by an explicit rule-name classification.
///
/// Built-in script and CSS rules do not use [`RuleCategory`], so callers that
/// own those registries use this name-only half of the category mapping.
pub(super) fn rule_name_matches_config_category(rule_name: &str, config_category: &str) -> bool {
    match config_category {
        "style" => is_style_rule_name(rule_name),
        "musea" => rule_name.starts_with("musea/"),
        "security" => is_security_rule_name(rule_name),
        "perf" => is_perf_rule_name(rule_name),
        _ => false,
    }
}

pub(super) fn rule_matches_config_category(
    rule_name: &str,
    rule_category: RuleCategory,
    config_category: &str,
) -> bool {
    match config_category {
        "correctness" => matches!(rule_category, RuleCategory::Essential),
        "style" => {
            rule_name_matches_config_category(rule_name, config_category)
                || matches!(rule_category, RuleCategory::StronglyRecommended)
        }
        "a11y" => matches!(rule_category, RuleCategory::Accessibility),
        "musea" => matches!(rule_category, RuleCategory::Musea),
        "security" | "perf" => rule_name_matches_config_category(rule_name, config_category),
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
            | "vue/no-static-inline-styles"
            | "vue/prefer-props-shorthand"
            | "vue/prefer-true-attribute-shorthand"
            | "vue/prop-name-casing"
            | "vue/require-scoped-style"
            | "vue/sfc-element-order"
            | "vue/single-style-block"
            | "vue/v-bind-style"
            | "vue/v-on-style"
            | "vue/v-slot-style"
            | "css/no-hardcoded-values"
            | "css/no-id-selectors"
            | "css/no-important"
            | "css/no-utility-classes"
            | "css/prefer-logical-properties"
            | "css/prefer-nested-selectors"
            | "css/prefer-slotted"
            | "nuxt/nuxt-config-keys-order"
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
        "css/no-display-none"
            | "css/no-v-bind-performance"
            | "css/require-font-display"
            | "script/no-async-in-computed"
            | "script/no-next-tick"
            | "script/no-top-level-ref-in-script"
            | "type/no-floating-promises"
            | "type/no-reactivity-loss"
    )
}

#[cfg(test)]
mod tests {
    use super::rule_name_matches_config_category;

    #[test]
    fn style_category_names_every_style_css_rule() {
        let style_css_rules = [
            "css/no-hardcoded-values",
            "css/no-id-selectors",
            "css/no-important",
            "css/no-utility-classes",
            "css/prefer-logical-properties",
            "css/prefer-nested-selectors",
            "css/prefer-slotted",
        ];

        for rule_name in style_css_rules {
            assert!(
                rule_name_matches_config_category(rule_name, "style"),
                "{rule_name} must belong to the style category",
            );
        }
    }

    #[test]
    fn perf_category_names_every_perf_css_rule() {
        let perf_css_rules = [
            "css/no-display-none",
            "css/no-v-bind-performance",
            "css/require-font-display",
        ];

        for rule_name in perf_css_rules {
            assert!(
                rule_name_matches_config_category(rule_name, "perf"),
                "{rule_name} must belong to the perf category",
            );
        }
    }
}
