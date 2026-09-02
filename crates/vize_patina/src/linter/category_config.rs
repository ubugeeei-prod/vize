//! Host category configuration across Patina's separate rule registries.

use super::{
    category_rules::{rule_matches_config_category, rule_name_matches_config_category},
    config::Linter,
};
use crate::Severity;
use vize_s0::String;

impl Linter {
    /// Disable every registered rule that belongs to one of the configured categories.
    #[inline]
    pub fn with_disabled_categories(mut self, categories: Vec<String>) -> Self {
        for category in categories {
            let template_rules = self
                .registry
                .rules()
                .iter()
                .filter(|rule| {
                    rule_matches_config_category(rule.meta().name, rule.meta().category, &category)
                })
                .map(|rule| String::from(rule.meta().name));
            self.disabled_rules.extend(template_rules);

            let script_rules = self
                .script_rules
                .iter()
                .copied()
                .filter(|rule_name| rule_name_matches_config_category(rule_name, &category))
                .map(String::from);
            self.disabled_rules.extend(script_rules);

            let musea_rules = self
                .musea_rules
                .iter()
                .copied()
                .filter(|rule_name| rule_name_matches_config_category(rule_name, &category))
                .map(String::from);
            self.disabled_rules.extend(musea_rules);
        }
        self
    }

    /// Apply category-level severity overrides to every registered matching rule.
    #[inline]
    pub fn with_category_severity_overrides(mut self, categories: Vec<(String, Severity)>) -> Self {
        for (category, severity) in categories {
            let template_rules = self
                .registry
                .rules()
                .iter()
                .filter(|rule| {
                    rule_matches_config_category(rule.meta().name, rule.meta().category, &category)
                })
                .map(|rule| (String::from(rule.meta().name), severity));
            self.severity_overrides.extend(template_rules);

            let script_rules = self
                .script_rules
                .iter()
                .copied()
                .filter(|rule_name| rule_name_matches_config_category(rule_name, &category))
                .map(|rule_name| (String::from(rule_name), severity));
            self.severity_overrides.extend(script_rules);

            let musea_rules = self
                .musea_rules
                .iter()
                .copied()
                .filter(|rule_name| rule_name_matches_config_category(rule_name, &category))
                .map(|rule_name| (String::from(rule_name), severity));
            self.severity_overrides.extend(musea_rules);
        }
        self
    }
}
