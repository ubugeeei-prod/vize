//! Existing linter constructors, shared by opt-in configuration.
#[cfg(not(target_arch = "wasm32"))]
use super::Mutex;
use super::{FxHashMap, FxHashSet, HelpLevel, LintPreset, Linter, Locale, RuleRegistry};
use crate::preset::{
    builtin_css_rule_names, builtin_script_rule_names, ecosystem_builtin_script_rule_names,
};

impl Linter {
    /// Default initial capacity for the arena (64KB).
    pub(crate) const DEFAULT_INITIAL_CAPACITY: usize = 64 * 1024;

    /// Create a new linter with the default happy-path preset.
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
            custom_elements: Default::default(),
            report_unused_disable_directives: false,
            script_rules: builtin_script_rule_names(preset),
            vapor_mode: false,
            requested_vue_version: None,
            requested_vapor_mode: None,
            explicit_no_get_current_instance: false,
            css_rules: builtin_css_rule_names(preset),
            musea_rules: &[],
            musea_design_tokens: None,
            script_rule_overrides: FxHashMap::default(),
            type_aware_enabled: false,
            strict_boolean_options: vize_l0::config::StrictBooleanExpressionsOptions::default(),
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_project_configs: Mutex::new(FxHashMap::default()),
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_project_ownership: Mutex::new(vize_canon::TsconfigOwnershipCache::default()),
            #[cfg(not(target_arch = "wasm32"))]
            reported_type_aware_project_configs: Mutex::new(FxHashSet::default()),
            #[cfg(not(target_arch = "wasm32"))]
            corsa_path: None,
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_tsconfig: None,
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
            custom_elements: Default::default(),
            report_unused_disable_directives: false,
            script_rules: builtin_script_rule_names(preset),
            vapor_mode: false,
            requested_vue_version: None,
            requested_vapor_mode: None,
            explicit_no_get_current_instance: false,
            css_rules: builtin_css_rule_names(preset),
            musea_rules: &[],
            musea_design_tokens: None,
            script_rule_overrides: FxHashMap::default(),
            type_aware_enabled: false,
            strict_boolean_options: vize_l0::config::StrictBooleanExpressionsOptions::default(),
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_project_configs: Mutex::new(FxHashMap::default()),
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_project_ownership: Mutex::new(vize_canon::TsconfigOwnershipCache::default()),
            #[cfg(not(target_arch = "wasm32"))]
            reported_type_aware_project_configs: Mutex::new(FxHashSet::default()),
            #[cfg(not(target_arch = "wasm32"))]
            corsa_path: None,
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_tsconfig: None,
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
            custom_elements: Default::default(),
            report_unused_disable_directives: false,
            script_rules: ecosystem_builtin_script_rule_names(),
            vapor_mode: false,
            requested_vue_version: None,
            requested_vapor_mode: None,
            explicit_no_get_current_instance: false,
            css_rules: builtin_css_rule_names(LintPreset::Ecosystem),
            musea_rules: &[],
            musea_design_tokens: None,
            script_rule_overrides: FxHashMap::default(),
            type_aware_enabled: false,
            strict_boolean_options: vize_l0::config::StrictBooleanExpressionsOptions::default(),
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_project_configs: Mutex::new(FxHashMap::default()),
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_project_ownership: Mutex::new(vize_canon::TsconfigOwnershipCache::default()),
            #[cfg(not(target_arch = "wasm32"))]
            reported_type_aware_project_configs: Mutex::new(FxHashSet::default()),
            #[cfg(not(target_arch = "wasm32"))]
            corsa_path: None,
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_tsconfig: None,
        }
    }

    /// Create a linter with every built-in rule enabled.
    #[inline]
    pub fn with_all() -> Self {
        let mut linter = Self::with_preset(LintPreset::Opinionated);
        linter.preset = None;
        linter.registry = RuleRegistry::with_all();
        linter.registry.register_opt_in_rules();
        linter.script_rules = super::super::script_rules::all_builtin_script_rule_names();
        linter.css_rules = super::super::css_rules::all_builtin_css_rule_names();
        linter.musea_rules = super::super::musea_rules::all_builtin_musea_rule_names();
        linter
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
            custom_elements: Default::default(),
            report_unused_disable_directives: false,
            script_rules: &[],
            vapor_mode: false,
            requested_vue_version: None,
            requested_vapor_mode: None,
            explicit_no_get_current_instance: false,
            css_rules: &[],
            musea_rules: &[],
            musea_design_tokens: None,
            script_rule_overrides: FxHashMap::default(),
            type_aware_enabled: false,
            strict_boolean_options: vize_l0::config::StrictBooleanExpressionsOptions::default(),
            #[cfg(not(target_arch = "wasm32"))]
            native_corsa: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_project_configs: Mutex::new(FxHashMap::default()),
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_project_ownership: Mutex::new(vize_canon::TsconfigOwnershipCache::default()),
            #[cfg(not(target_arch = "wasm32"))]
            reported_type_aware_project_configs: Mutex::new(FxHashSet::default()),
            #[cfg(not(target_arch = "wasm32"))]
            corsa_path: None,
            #[cfg(not(target_arch = "wasm32"))]
            type_aware_tsconfig: None,
        }
    }
}
