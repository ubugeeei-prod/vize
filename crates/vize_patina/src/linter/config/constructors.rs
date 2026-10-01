//! Existing linter constructors, shared by opt-in configuration.
use super::*;
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
            script_rules: builtin_script_rule_names(preset),
            vapor_mode: false,
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
            vapor_mode: false,
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
            vapor_mode: false,
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
            vapor_mode: false,
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
            corsa_path: None,
        }
    }
}
