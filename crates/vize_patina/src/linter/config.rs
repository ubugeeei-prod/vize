//! Linter configuration and result types.
//!
//! Defines the `LintResult` output type and the `Linter` struct with its
//! builder-pattern configuration methods.

#[cfg(not(target_arch = "wasm32"))]
use super::corsa_session::CorsaTypeAwareSession;
use crate::{
    diagnostic::{HelpLevel, LintDiagnostic},
    preset::{
        LintPreset, builtin_css_rule_names, builtin_script_rule_names,
        ecosystem_builtin_script_rule_names,
    },
    rule::RuleRegistry,
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use vize_carton::{FxHashSet, String, i18n::Locale};

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
    /// Help display level.
    pub(crate) help_level: HelpLevel,
    /// Built-in script rules enabled for this linter.
    pub(crate) script_rules: &'static [&'static str],
    /// Built-in `css/*` rules enabled for this linter.
    pub(crate) css_rules: &'static [&'static str],
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
            help_level: HelpLevel::default(),
            script_rules: builtin_script_rule_names(preset),
            css_rules: builtin_css_rule_names(preset),
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
            help_level: HelpLevel::default(),
            script_rules: builtin_script_rule_names(preset),
            css_rules: builtin_css_rule_names(preset),
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
            help_level: HelpLevel::default(),
            script_rules: ecosystem_builtin_script_rule_names(),
            css_rules: builtin_css_rule_names(LintPreset::Ecosystem),
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
            help_level: HelpLevel::default(),
            script_rules: &[],
            css_rules: &[],
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

    /// Register an extra rule if the active preset did not already include it.
    #[inline]
    pub fn with_rule(mut self, rule: Box<dyn crate::rule::Rule>) -> Self {
        let rule_name = rule.meta().name;
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
