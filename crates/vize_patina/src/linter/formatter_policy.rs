//! Metadata-only activation for the formatter's bind-style interaction.

use super::Linter;
use crate::LintPreset;
use vize_l0::{
    String,
    config::{ConfigLintRuleOptions, LinterConfig, VBindStyle},
    cstr,
};

impl Linter {
    /// Validate only the formatter's interaction with an active bind-style rule.
    pub fn validate_directive_normalization(
        config: &LinterConfig,
        options: &ConfigLintRuleOptions,
        normalize: bool,
    ) -> Result<(), String> {
        if normalize && Self::has_active_longform_bind(config, options)? {
            return Err(String::from(
                "normalizeDirectiveShorthands=true conflicts with active vue/v-bind-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand binding style",
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
        if !config.enabled || options.v_bind_style() != Some(VBindStyle::Longform) {
            return Ok(false);
        }
        let linter = match config.preset.as_deref() {
            None => Self::new(),
            Some("all") => Self::with_all(),
            Some(name) => Self::with_preset(
                LintPreset::parse(name).ok_or_else(|| cstr!("Unknown lint preset '{name}'"))?,
            ),
        }
        .with_additional_rules(config.enabled_rules())
        .with_disabled_rules(config.disabled_rules())
        .with_disabled_categories(config.disabled_categories());
        Ok(linter.registry().has_rule("vue/v-bind-style")
            && linter.is_rule_enabled("vue/v-bind-style"))
    }
}

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
    fn option_does_not_enable_a_rule_and_effective_disabled_policy_wins() {
        let contract: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tests/_fixtures/differential/linter/formatter-bind-style-policy-6098/cases.json"
        ))
        .unwrap();
        assert!(active(contract["config"]["linter"].clone()).unwrap());
        for linter in [
            serde_json::json!({"preset":"incremental","ruleOptions":{"vue/v-bind-style":"longform"}}),
            serde_json::json!({"enabled":false,"preset":"opinionated","ruleOptions":{"vue/v-bind-style":"longform"}}),
            serde_json::json!({"preset":"opinionated","rules":{"vue/v-bind-style":"off"},"ruleOptions":{"vue/v-bind-style":"longform"}}),
            serde_json::json!({"preset":"opinionated","categories":{"style":"off"},"ruleOptions":{"vue/v-bind-style":"longform"}}),
            serde_json::json!({"preset":"incremental","rules":{"vue/v-bind-style":"error"}}),
            serde_json::json!({"preset":"incremental","rules":{"vue/v-bind-style":"error"},"ruleOptions":{"vue/v-bind-style":"shorthand"}}),
        ] {
            assert!(!active(linter).unwrap());
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
}
