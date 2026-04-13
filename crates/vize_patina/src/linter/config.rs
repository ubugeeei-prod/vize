//! Linter configuration and result types.
//!
//! Defines the `LintResult` output type and the `Linter` struct with its
//! builder-pattern configuration methods.

#[cfg(not(target_arch = "wasm32"))]
use super::corsa_session::CorsaTypeAwareSession;
use crate::{
    diagnostic::{HelpLevel, LintDiagnostic},
    preset::{builtin_script_rule_names, LintPreset},
    rule::RuleRegistry,
};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use vize_carton::{i18n::Locale, FxHashSet, String};

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
    pub(crate) registry: RuleRegistry,
    /// Estimated initial allocator capacity (in bytes).
    pub(crate) initial_capacity: usize,
    /// Locale for i18n messages.
    pub(crate) locale: Locale,
    /// Optional set of enabled rule names (if None, all rules are enabled).
    pub(crate) enabled_rules: Option<FxHashSet<String>>,
    /// Help display level.
    pub(crate) help_level: HelpLevel,
    /// Built-in script rules enabled for this linter.
    pub(crate) script_rules: &'static [&'static str],
    /// Lazily initialized native corsa session for type-aware lint.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) native_corsa: Mutex<Option<CorsaTypeAwareSession>>,
}

impl Linter {
    /// Default initial capacity for the arena (64KB).
    pub(crate) const DEFAULT_INITIAL_CAPACITY: usize = 64 * 1024;

    /// Create a new linter with the default happy-path preset.
    #[inline]
    pub fn new() -> Self {
        let preset = LintPreset::default();
        Self {
            registry: RuleRegistry::with_preset(preset),
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
            help_level: HelpLevel::default(),
            script_rules: builtin_script_rule_names(preset),
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
        }
    }

    /// Create a new linter with a named preset.
    #[inline]
    pub fn with_preset(preset: LintPreset) -> Self {
        Self {
            registry: RuleRegistry::with_preset(preset),
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
            help_level: HelpLevel::default(),
            script_rules: builtin_script_rule_names(preset),
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
        }
    }

    /// Create a linter with a custom rule registry.
    #[inline]
    pub fn with_registry(registry: RuleRegistry) -> Self {
        Self {
            registry,
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
            help_level: HelpLevel::default(),
            script_rules: &[],
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
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
        self.enabled_rules = rules.map(|r| r.into_iter().collect());
        self
    }

    /// Set the help display level.
    #[inline]
    pub fn with_help_level(mut self, level: HelpLevel) -> Self {
        self.help_level = level;
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
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}
