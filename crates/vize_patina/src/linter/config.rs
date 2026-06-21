//! Linter configuration and result types.
//!
//! Defines the `LintResult` output type and the `Linter` struct with its
//! builder-pattern configuration methods.

#[cfg(not(target_arch = "wasm32"))]
use super::corsa_session::CorsaTypeAwareSession;
use crate::{
    diagnostic::{HelpLevel, LintDiagnostic, Severity},
    preset::{
        LintPreset, builtin_css_rule_names, builtin_script_rule_names,
        ecosystem_builtin_script_rule_names,
    },
    rule::{RuleCategory, RuleRegistry},
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use vize_carton::{FxHashMap, FxHashSet, String, i18n::Locale};

/// Lint result for a single file.
#[derive(Debug, Clone)]
pub struct LintResult {
    /// Filename that was linted.
    pub filename: String,
    /// Collected diagnostics.
    pub diagnostics: Vec<LintDiagnostic>,
    /// Number of errors.
    pub error_count: usize,
    /// Number of warnings.
    pub warning_count: usize,
}

impl LintResult {
    /// Check if there are any errors.
    #[inline]
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Check if there are any diagnostics.
    #[inline]
    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

/// Main linter struct.
///
/// The linter is designed for high performance:
/// - Uses arena allocation for AST and context
/// - Pre-allocates vectors with expected capacity
/// - Minimizes allocations during traversal
pub struct Linter {
    /// Preset used to seed the rule registry, when applicable.
    pub(crate) preset: Option<LintPreset>,
    pub(crate) registry: RuleRegistry,
    /// Estimated initial allocator capacity (in bytes).
    pub(crate) initial_capacity: usize,
    /// Locale for i18n messages.
    pub(crate) locale: Locale,
    /// Optional set of enabled rule names (if None, all rules are enabled).
    pub(crate) enabled_rules: Option<FxHashSet<String>>,
    /// Rule names disabled by host configuration.
    pub(crate) disabled_rules: FxHashSet<String>,
    /// Rule-level severity overrides from host configuration.
    pub(crate) severity_overrides: FxHashMap<String, Severity>,
    /// Help display level.
    pub(crate) help_level: HelpLevel,
    /// Built-in script rules enabled for this linter.
    pub(crate) script_rules: &'static [&'static str],
    /// Built-in `css/*` rules enabled for this linter.
    pub(crate) css_rules: &'static [&'static str],
    /// Whether native type-aware lint rules may run.
    pub(crate) type_aware_enabled: bool,
    /// Lazily initialized native corsa session for type-aware lint.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) native_corsa: Mutex<Option<CorsaTypeAwareSession>>,
    /// Optional configured Corsa executable for type-aware lint.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) corsa_path: Option<PathBuf>,
}

impl Linter {
    /// Default initial capacity for the arena (64KB).
    pub(crate) const DEFAULT_INITIAL_CAPACITY: usize = 64 * 1024;

    /// Create a new linter with the default ecosystem preset.
    #[inline]
    pub fn new() -> Self {
        let preset = LintPreset::default();
        Self {
            preset: Some(preset),
            registry: RuleRegistry::with_preset(preset),
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
            disabled_rules: FxHashSet::default(),
            severity_overrides: FxHashMap::default(),
            help_level: HelpLevel::default(),
            script_rules: builtin_script_rule_names(preset),
            css_rules: builtin_css_rule_names(preset),
            type_aware_enabled: false,
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            corsa_path: None,
        }
    }

    /// Create a new linter with a named preset.
    #[inline]
    pub fn with_preset(preset: LintPreset) -> Self {
        Self {
            preset: Some(preset),
            registry: RuleRegistry::with_preset(preset),
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
            disabled_rules: FxHashSet::default(),
            severity_overrides: FxHashMap::default(),
            help_level: HelpLevel::default(),
            script_rules: builtin_script_rule_names(preset),
            css_rules: builtin_css_rule_names(preset),
            type_aware_enabled: false,
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            corsa_path: None,
        }
    }

    /// Create a new linter with Vue ecosystem integration rules enabled.
    #[inline]
    pub fn with_ecosystem() -> Self {
        Self {
            preset: None,
            registry: RuleRegistry::with_ecosystem(),
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
            disabled_rules: FxHashSet::default(),
            severity_overrides: FxHashMap::default(),
            help_level: HelpLevel::default(),
            script_rules: ecosystem_builtin_script_rule_names(),
            css_rules: builtin_css_rule_names(LintPreset::Ecosystem),
            type_aware_enabled: false,
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            corsa_path: None,
        }
    }

    /// Create a linter with a custom rule registry.
    #[inline]
    pub fn with_registry(registry: RuleRegistry) -> Self {
        Self {
            preset: None,
            registry,
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
            disabled_rules: FxHashSet::default(),
            severity_overrides: FxHashMap::default(),
            help_level: HelpLevel::default(),
            script_rules: &[],
            css_rules: &[],
            type_aware_enabled: false,
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            corsa_path: None,
        }
    }

    /// Set the initial allocator capacity.
    #[inline]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = capacity;
        self
    }

    /// Set the locale for i18n messages.
    #[inline]
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Set enabled rules (if None, all rules are enabled).
    ///
    /// Pass a list of rule names to enable only those rules.
    /// Rules not in the list will be skipped during linting.
    #[inline]
    pub fn with_enabled_rules(mut self, rules: Option<Vec<String>>) -> Self {
        if rules.is_some() {
            if matches!(self.preset, Some(LintPreset::Incremental)) {
                self.registry = RuleRegistry::with_preset(LintPreset::Opinionated);
            }
            self.registry.register_opt_in_rules();
            self.script_rules = super::script_rules::all_builtin_script_rule_names();
            self.css_rules = super::css_rules::all_builtin_css_rule_names();
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
            names
        });

        if matches!(self.preset, Some(LintPreset::Incremental)) {
            self.registry = RuleRegistry::with_preset(LintPreset::Opinionated);
        }
        if has_type_rule(&rules) {
            self.type_aware_enabled = true;
        }
        self.registry.register_opt_in_rules();
        self.script_rules = super::script_rules::all_builtin_script_rule_names();
        self.css_rules = super::css_rules::all_builtin_css_rule_names();
        enabled_rules.extend(rules);
        self.enabled_rules = Some(enabled_rules);
        self
    }

    /// Disable selected rules while preserving the active preset.
    #[inline]
    pub fn with_disabled_rules(mut self, rules: Vec<String>) -> Self {
        self.disabled_rules = rules.into_iter().collect();
        self
    }

    /// Disable every registered rule that belongs to one of the configured categories.
    #[inline]
    pub fn with_disabled_categories(mut self, categories: Vec<String>) -> Self {
        if categories.is_empty() {
            return self;
        }

        for category in categories {
            let disabled = self
                .registry
                .rules()
                .iter()
                .filter(|rule| {
                    rule_matches_config_category(rule.meta().name, rule.meta().category, &category)
                })
                .map(|rule| String::from(rule.meta().name));
            self.disabled_rules.extend(disabled);
        }
        self
    }

    /// Apply rule-level severity overrides from host configuration.
    #[inline]
    pub fn with_rule_severity_overrides(mut self, rules: Vec<(String, Severity)>) -> Self {
        self.severity_overrides.extend(rules);
        self
    }

    /// Apply category-level severity overrides to every registered matching rule.
    #[inline]
    pub fn with_category_severity_overrides(mut self, categories: Vec<(String, Severity)>) -> Self {
        if categories.is_empty() {
            return self;
        }

        for (category, severity) in categories {
            let overrides = self
                .registry
                .rules()
                .iter()
                .filter(|rule| {
                    rule_matches_config_category(rule.meta().name, rule.meta().category, &category)
                })
                .map(|rule| (String::from(rule.meta().name), severity));
            self.severity_overrides.extend(overrides);
        }
        self
    }

    /// Register an extra rule if the active preset did not already include it.
    #[inline]
    pub fn with_rule(mut self, rule: Box<dyn crate::rule::Rule>) -> Self {
        let rule_name = rule.meta().name;
        if is_type_rule(rule_name) {
            self.type_aware_enabled = true;
        }
        if !self.registry.has_rule(rule_name) {
            self.registry.register(rule);
            self.registry.mark_has_exit_element_rules();
        }
        self
    }

    /// Set the help display level.
    #[inline]
    pub fn with_help_level(mut self, level: HelpLevel) -> Self {
        self.help_level = level;
        self
    }

    /// Allow native type-aware lint rules to run.
    ///
    /// Keeping this separate from rule membership preserves zero-cost defaults:
    /// presets may contain `type/*` rules, but Patina will not parse SFCs for
    /// Corsa-backed checks or start Corsa unless hosts explicitly opt in.
    #[inline]
    pub fn with_type_aware_lint(mut self, enabled: bool) -> Self {
        self.type_aware_enabled = enabled;
        self
    }

    /// Set the Corsa executable used by native type-aware lint rules.
    #[cfg(not(target_arch = "wasm32"))]
    #[inline]
    pub fn with_corsa_path(mut self, path: Option<PathBuf>) -> Self {
        self.corsa_path = path;
        self
    }

    /// Get the current locale.
    #[inline]
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// Check if a rule is enabled.
    #[inline]
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        if self.disabled_rules.contains(rule_name) {
            return false;
        }
        match &self.enabled_rules {
            Some(set) => set.contains(rule_name),
            None => true,
        }
    }

    /// Get the rule registry.
    #[inline]
    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }

    /// Get all registered rules.
    #[inline]
    pub fn rules(&self) -> &[Box<dyn crate::rule::Rule>] {
        self.registry.rules()
    }

    /// Get all registered rule names.
    #[inline]
    pub(crate) fn rule_names(&self) -> &[&'static str] {
        self.registry.rule_names()
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

fn has_type_rule(rules: &[String]) -> bool {
    rules.iter().any(|rule| is_type_rule(rule.as_str()))
}

fn is_type_rule(rule_name: &str) -> bool {
    rule_name.starts_with("type/")
}

fn rule_matches_config_category(
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
