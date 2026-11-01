//! Metadata-only activation for the formatter's directive-style interactions.

use super::Linter;
use crate::LintPreset;
use crate::rules::vue::{
    VBindStyleOption, VOnStyleOption, VSlotStyle as SlotRule, VSlotStyleOption,
};
use vize_l0::{
    String,
    config::{
        ConfigLintRuleOptions, LinterConfig, VBindStyle, VOnStyle, VSlotNamedStyle, VSlotStyle,
        VSlotStyleValue, VSlotUniformStyle,
    },
    cstr,
};

#[derive(Clone, Copy)]
enum StyleRule {
    Bind,
    On,
    Slot,
}

impl Linter {
    /// Validate the formatter's interaction with active directive style rules.
    pub fn validate_directive_normalization(
        config: &LinterConfig,
        options: &ConfigLintRuleOptions,
        normalize: bool,
    ) -> Result<(), String> {
        Self::validate_normalization(config, options, normalize, false)
    }

    /// Match the editor's existing constructor, including its category handling.
    pub fn validate_editor_directive_normalization(
        config: &LinterConfig,
        options: &ConfigLintRuleOptions,
        normalize: bool,
    ) -> Result<(), String> {
        Self::validate_normalization(config, options, normalize, true)
    }

    fn validate_normalization(
        config: &LinterConfig,
        options: &ConfigLintRuleOptions,
        normalize: bool,
        editor: bool,
    ) -> Result<(), String> {
        if normalize && Self::active_configured_bind(config, options, editor)? {
            return Err(String::from(
                "normalizeDirectiveShorthands=true conflicts with active vue/v-bind-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand binding style",
            ));
        }
        if normalize && Self::active_configured_style(config, options, editor, StyleRule::On)? {
            return Err(String::from(
                "normalizeDirectiveShorthands=true conflicts with active vue/v-on-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand event style",
            ));
        }
        if normalize && Self::active_configured_style(config, options, editor, StyleRule::Slot)? {
            return Err(String::from(
                "normalizeDirectiveShorthands=true conflicts with an active vue/v-slot-style longform policy; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand/v-slot for the affected slot positions",
            ));
        }
        Ok(())
    }

    /// Whether a formatter would undo an active longform binding policy.
    ///
    /// Uses the actual preset, rule and category builders. No lint traversal,
    /// source parsing, type-aware query or backend initialization is performed.
    pub fn has_active_longform_bind(
        config: &LinterConfig,
        options: &ConfigLintRuleOptions,
    ) -> Result<bool, String> {
        Self::active_configured_bind(config, options, false)
    }

    fn active_configured_bind(
        config: &LinterConfig,
        options: &ConfigLintRuleOptions,
        editor: bool,
    ) -> Result<bool, String> {
        Self::active_configured_style(config, options, editor, StyleRule::Bind)
    }

    fn active_configured_style(
        config: &LinterConfig,
        options: &ConfigLintRuleOptions,
        editor: bool,
        rule: StyleRule,
    ) -> Result<bool, String> {
        let (longform, name) = match rule {
            StyleRule::Bind => (
                options.v_bind_style() == Some(VBindStyle::Longform),
                "vue/v-bind-style",
            ),
            StyleRule::On => (
                options.v_on_style() == Some(VOnStyle::Longform),
                "vue/v-on-style",
            ),
            StyleRule::Slot => (
                match options.v_slot_style() {
                    Some(VSlotStyle::Uniform(style)) => style == VSlotUniformStyle::Longform,
                    Some(VSlotStyle::ByPosition(style)) => {
                        style.at_component == VSlotStyleValue::Longform
                            || style.default_slot == VSlotStyleValue::Longform
                            || style.named == VSlotNamedStyle::Longform
                    }
                    None => false,
                },
                "vue/v-slot-style",
            ),
        };
        if !config.enabled || !longform {
            return Ok(false);
        }
        let disabled = config.disabled_rules();
        let mut additional = config.enabled_rules();
        for (name, configured) in [
            (
                "script/no-restricted-members",
                !options.restricted_members().is_empty(),
            ),
            (
                "musea/prefer-design-tokens",
                !options.musea_design_tokens().is_empty(),
            ),
        ] {
            if configured
                && !disabled.iter().any(|rule| rule == name)
                && !additional.iter().any(|rule| rule == name)
            {
                additional.push(name.into());
            }
        }
        let mut linter = match config.preset.as_deref() {
            None => Self::new(),
            Some("all") if !editor => Self::with_all(),
            Some(name) => Self::with_preset(
                LintPreset::parse(name).ok_or_else(|| cstr!("Unknown lint preset '{name}'"))?,
            ),
        }
        .with_additional_rules(additional)
        .with_disabled_rules(disabled);
        // The existing CLI applies categories before configurable rules. The
        // editor constructor currently does not apply disabled categories.
        if !editor {
            linter = linter.with_disabled_categories(config.disabled_categories());
        }
        // Bind/On can register a missing rule; Slot only replaces a selected
        // instance. Their actual registry and enabled/disabled sets decide.
        let linter = match rule {
            StyleRule::Bind => linter.with_v_bind_style(VBindStyleOption::Longform),
            StyleRule::On => linter.with_v_on_style(VOnStyleOption::Longform),
            StyleRule::Slot => {
                linter.with_v_slot_style(SlotRule::uniform(VSlotStyleOption::Longform))
            }
        };
        Ok(linter.registry().has_rule(name) && linter.is_rule_enabled(name))
    }
}

#[cfg(test)]
mod slot_style_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::config::ConfigDocument;

    fn active(linter: serde_json::Value) -> Result<bool, String> {
        let document: ConfigDocument =
            serde_json::from_value(serde_json::json!({ "linter": linter })).unwrap();
        let resolved = document
            .linter_plan_with_config_rule_options()
            .resolve_matching_entries(&[]);
        Linter::has_active_longform_bind(&resolved.config, &resolved.rule_options)
    }

    #[test]
    fn mirrors_actual_configurable_rule_registration_and_disabled_selection() {
        let contract: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tests/_fixtures/differential/linter/formatter-bind-style-policy-6098/cases.json"
        ))
        .unwrap();
        assert!(active(contract["config"]["linter"].clone()).unwrap());
        for linter in [
            serde_json::json!({"preset":"incremental","rules":{"vue/no-v-html":"error"},"ruleOptions":{"vue/v-bind-style":"longform"}}),
            serde_json::json!({"enabled":false,"preset":"opinionated","ruleOptions":{"vue/v-bind-style":"longform"}}),
            serde_json::json!({"preset":"opinionated","rules":{"vue/v-bind-style":"off"},"ruleOptions":{"vue/v-bind-style":"longform"}}),
            serde_json::json!({"preset":"opinionated","categories":{"style":"off"},"ruleOptions":{"vue/v-bind-style":"longform"}}),
            serde_json::json!({"preset":"incremental","rules":{"vue/v-bind-style":"error"}}),
            serde_json::json!({"preset":"incremental","rules":{"vue/v-bind-style":"error"},"ruleOptions":{"vue/v-bind-style":"shorthand"}}),
        ] {
            assert!(!active(linter).unwrap());
        }
        for categories in [serde_json::json!({}), serde_json::json!({"style":"off"})] {
            assert!(active(serde_json::json!({"preset":"incremental","categories":categories,"ruleOptions":{"vue/v-bind-style":"longform"}})).unwrap());
        }
        for severity in ["warn", "error"] {
            assert!(active(serde_json::json!({"preset":"incremental","rules":{"vue/v-bind-style":severity},"ruleOptions":{"vue/v-bind-style":"longform"}})).unwrap());
        }
        assert!(
            active(
                serde_json::json!({"preset":"all","ruleOptions":{"vue/v-bind-style":"longform"}})
            )
            .unwrap()
        );
        assert!(active(serde_json::json!({"preset":"opinionated","ruleOptions":{"vue/v-bind-style":"longform"}})).unwrap());
        let defaults =
            active(serde_json::json!({"ruleOptions":{"vue/v-bind-style":"longform"}})).unwrap();
        let ecosystem = active(
            serde_json::json!({"preset":"ecosystem","ruleOptions":{"vue/v-bind-style":"longform"}}),
        )
        .unwrap();
        assert!(defaults);
        assert_eq!(defaults, ecosystem);
        assert!(active(serde_json::json!({"preset":"unknown","ruleOptions":{"vue/v-bind-style":"longform"}})).is_err());
    }

    #[test]
    fn event_conflict_preserves_bind_priority_and_explicit_false() {
        let document: ConfigDocument = serde_json::from_value(serde_json::json!({
            "linter":{"preset":"incremental","rules":{"vue/v-on-style":"error"},
                "ruleOptions":{"vue/v-on-style":"longform"}}
        }))
        .unwrap();
        let resolved = document
            .linter_plan_with_config_rule_options()
            .resolve_matching_entries(&[]);
        let reason = "normalizeDirectiveShorthands=true conflicts with active vue/v-on-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand event style";
        assert_eq!(
            Linter::validate_directive_normalization(
                &resolved.config,
                &resolved.rule_options,
                true
            ),
            Err(String::from(reason))
        );
        assert_eq!(
            Linter::validate_editor_directive_normalization(
                &resolved.config,
                &resolved.rule_options,
                true
            ),
            Err(String::from(reason))
        );
        assert_eq!(
            Linter::validate_directive_normalization(
                &resolved.config,
                &resolved.rule_options,
                false
            ),
            Ok(())
        );
        let both: ConfigDocument = serde_json::from_value(serde_json::json!({
            "linter":{"preset":"incremental","rules":{"vue/v-bind-style":"error","vue/v-on-style":"error"},
                "ruleOptions":{"vue/v-bind-style":"longform","vue/v-on-style":"longform"}}
        })).unwrap();
        let resolved = both
            .linter_plan_with_config_rule_options()
            .resolve_matching_entries(&[]);
        let reason = "normalizeDirectiveShorthands=true conflicts with active vue/v-bind-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand binding style";
        assert_eq!(
            Linter::validate_directive_normalization(
                &resolved.config,
                &resolved.rule_options,
                true
            ),
            Err(String::from(reason))
        );
        assert_eq!(
            Linter::validate_editor_directive_normalization(
                &resolved.config,
                &resolved.rule_options,
                true
            ),
            Err(String::from(reason))
        );
    }
}
