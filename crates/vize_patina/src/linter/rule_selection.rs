use super::config::{Linter, is_type_rule};
use crate::preset::LintPreset;
use vize_l0::{FxHashSet, String};

impl Linter {
    /// Set enabled rules (if None, all rules are enabled).
    ///
    /// Pass a list of rule names to enable only those rules.
    /// Rules not in the list will be skipped during linting.
    #[inline]
    pub fn with_enabled_rules(mut self, rules: Option<Vec<String>>) -> Self {
        self.explicit_no_get_current_instance = rules.as_ref().is_some_and(|rules| {
            rules
                .iter()
                .any(|rule| rule == "script/no-get-current-instance")
        });
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
        self.explicit_no_get_current_instance |= rules
            .iter()
            .any(|rule| rule == "script/no-get-current-instance");

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
        register_configured_rules(&mut self.registry, &rules);
        enabled_rules.extend(rules);
        self.enabled_rules = Some(enabled_rules);
        self
    }
}

/// Instantiate rules named in config when the active preset never registered them.
///
/// Names stay limited to `rules`. Other rules from the catalog are not enabled.
fn register_configured_rules(registry: &mut crate::RuleRegistry, rules: &[String]) {
    let wanted: Vec<&str> = rules
        .iter()
        .map(String::as_str)
        .filter(|rule| !registry.has_rule(rule) && !rule_lives_outside_registry(rule))
        .collect();
    if wanted.is_empty() {
        return;
    }
    let mut catalog = crate::RuleRegistry::with_all();
    catalog.register_opt_in_rules();
    let mut extras = catalog.take_matching(&wanted);
    let still_missing: Vec<&str> = wanted
        .iter()
        .copied()
        .filter(|name| !extras.iter().any(|rule| rule.meta().name == *name))
        .collect();
    if !still_missing.is_empty() {
        let mut nuxt = crate::RuleRegistry::with_nuxt();
        nuxt.register_opt_in_rules();
        extras.extend(nuxt.take_matching(&still_missing));
    }
    let mut registered = false;
    for rule in extras {
        if registry.has_rule(rule.meta().name) {
            continue;
        }
        registry.register(rule);
        registered = true;
    }
    if registered {
        registry.mark_has_exit_element_rules();
    }
}

fn rule_lives_outside_registry(rule: &str) -> bool {
    super::script_rules::all_builtin_script_rule_names().contains(&rule)
        || super::css_rules::all_builtin_css_rule_names().contains(&rule)
        || super::musea_rules::all_builtin_musea_rule_names().contains(&rule)
}

fn has_type_rule(rules: &[String]) -> bool {
    rules.iter().any(|rule| is_type_rule(rule.as_str()))
}
