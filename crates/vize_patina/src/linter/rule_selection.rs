use super::config::{Linter, is_type_rule};
use crate::preset::LintPreset;
use vize_s0::{FxHashSet, String};

impl Linter {
    /// Set enabled rules (if None, all rules are enabled).
    ///
    /// Pass a list of rule names to enable only those rules.
    /// Rules not in the list will be skipped during linting.
    #[inline]
    pub fn with_enabled_rules(mut self, rules: Option<Vec<String>>) -> Self {
        if rules.is_some() {
            if matches!(self.preset, Some(LintPreset::Incremental)) {
                self.registry = crate::RuleRegistry::with_preset(LintPreset::Opinionated);
            }
            self.registry.register_opt_in_rules();
            self.script_rules = super::script_rules::all_builtin_script_rule_names();
            self.css_rules = super::css_rules::all_builtin_css_rule_names();
            self.musea_rules = super::musea_rules::all_builtin_musea_rule_names();
        }
        if rules.as_ref().is_some_and(|rules| has_type_rule(rules)) {
            self.type_aware_enabled = true;
        }
        self.enabled_rules = rules.map(|r| r.into_iter().collect());
        self
    }

    /// Enable additional opt-in rules while preserving the active preset's rules.
    #[inline]
    pub fn with_additional_rules(mut self, rules: Vec<String>) -> Self {
        if rules.is_empty() {
            return self;
        }

        let mut enabled_rules = self.enabled_rules.take().unwrap_or_else(|| {
            let mut names = self
                .registry
                .rule_names()
                .iter()
                .map(|name| String::from(*name))
                .collect::<FxHashSet<_>>();
            names.extend(self.script_rules.iter().map(|name| String::from(*name)));
            names.extend(self.css_rules.iter().map(|name| String::from(*name)));
            names.extend(self.musea_rules.iter().map(|name| String::from(*name)));
            names
        });

        if matches!(self.preset, Some(LintPreset::Incremental)) {
            self.registry = crate::RuleRegistry::with_preset(LintPreset::Opinionated);
        }
        if has_type_rule(&rules) {
            self.type_aware_enabled = true;
        }
        self.registry.register_opt_in_rules();
        self.script_rules = super::script_rules::all_builtin_script_rule_names();
        self.css_rules = super::css_rules::all_builtin_css_rule_names();
        self.musea_rules = super::musea_rules::all_builtin_musea_rule_names();
        enabled_rules.extend(rules);
        self.enabled_rules = Some(enabled_rules);
        self
    }
}

fn has_type_rule(rules: &[String]) -> bool {
    rules.iter().any(|rule| is_type_rule(rule.as_str()))
}
