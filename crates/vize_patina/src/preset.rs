//! Built-in lint presets for Patina.

/// Named lint presets exposed across Rust, CLI, and bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LintPreset {
    /// General-purpose defaults that focus on the common Vue happy path.
    #[default]
    HappyPath,
    /// Happy path rules plus stricter conventions and framework assumptions.
    Opinionated,
    /// Error-focused correctness checks.
    Essential,
    /// Opinionated rules adjusted for Nuxt auto-import conventions.
    Nuxt,
}

impl LintPreset {
    pub const ALL: [Self; 4] = [
        Self::HappyPath,
        Self::Opinionated,
        Self::Essential,
        Self::Nuxt,
    ];

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HappyPath => "happy-path",
            Self::Opinionated => "opinionated",
            Self::Essential => "essential",
            Self::Nuxt => "nuxt",
        }
    }

    #[inline]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "happy-path" | "happy_path" | "happy" | "default" | "recommended" => {
                Some(Self::HappyPath)
            }
            "opinionated" | "strict" | "all" => Some(Self::Opinionated),
            "essential" => Some(Self::Essential),
            "nuxt" => Some(Self::Nuxt),
            _ => None,
        }
    }
}

pub(crate) const fn builtin_script_rule_names(preset: LintPreset) -> &'static [&'static str] {
    match preset {
        LintPreset::HappyPath | LintPreset::Essential => &[],
        LintPreset::Opinionated | LintPreset::Nuxt => &[
            "script/no-options-api",
            "script/no-get-current-instance",
            "script/no-next-tick",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::LintPreset;
    use crate::rule::RuleRegistry;

    #[test]
    fn parses_common_aliases() {
        assert_eq!(LintPreset::parse("default"), Some(LintPreset::HappyPath));
        assert_eq!(
            LintPreset::parse("recommended"),
            Some(LintPreset::HappyPath)
        );
        assert_eq!(LintPreset::parse("all"), Some(LintPreset::Opinionated));
        assert_eq!(LintPreset::parse("strict"), Some(LintPreset::Opinionated));
        assert_eq!(LintPreset::parse("nuxt"), Some(LintPreset::Nuxt));
        assert_eq!(LintPreset::parse("unknown"), None);
    }

    #[test]
    fn preset_rule_membership_snapshot() {
        let snapshot = serde_json::json!({
            "happy_path": rule_names(LintPreset::HappyPath),
            "opinionated": rule_names(LintPreset::Opinionated),
            "essential": rule_names(LintPreset::Essential),
            "nuxt": rule_names(LintPreset::Nuxt),
        });

        insta::assert_snapshot!(
            "lint_preset_rule_membership",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    }

    #[test]
    fn happy_path_keeps_opinionated_rules_opt_in() {
        let happy_path = RuleRegistry::with_preset(LintPreset::HappyPath);
        let opinionated = RuleRegistry::with_preset(LintPreset::Opinionated);

        assert!(happy_path.has_rule("vue/attribute-order"));
        assert!(happy_path.has_rule("vue/component-definition-name-casing"));
        assert!(happy_path.has_rule("vue/html-quotes"));
        assert!(happy_path.has_rule("vue/mustache-interpolation-spacing"));
        assert!(happy_path.has_rule("vue/no-lone-template"));
        assert!(happy_path.has_rule("vue/no-multi-spaces"));
        assert!(happy_path.has_rule("vue/no-unused-properties"));
        assert!(happy_path.has_rule("vue/prop-name-casing"));
        assert!(happy_path.has_rule("vue/require-scoped-style"));
        assert!(happy_path.has_rule("vue/sfc-element-order"));
        assert!(happy_path.has_rule("vue/single-style-block"));
        assert!(happy_path.has_rule("vue/v-on-style"));
        assert!(happy_path.has_rule("vue/v-slot-style"));
        assert!(happy_path.has_rule("vapor/no-vue-lifecycle-events"));
        assert!(happy_path.has_rule("type/require-typed-props"));
        assert!(happy_path.has_rule("type/require-typed-emits"));
        assert!(happy_path.has_rule("html/no-empty-palpable-content"));
        assert!(!happy_path.has_rule("vue/multi-word-component-names"));
        assert!(!happy_path.has_rule("a11y/use-list"));
        assert!(opinionated.has_rule("vue/attribute-order"));
        assert!(opinionated.has_rule("vue/component-definition-name-casing"));
        assert!(opinionated.has_rule("vue/html-quotes"));
        assert!(opinionated.has_rule("vue/mustache-interpolation-spacing"));
        assert!(opinionated.has_rule("vue/no-lone-template"));
        assert!(opinionated.has_rule("vue/no-multi-spaces"));
        assert!(opinionated.has_rule("vue/no-unused-properties"));
        assert!(opinionated.has_rule("vue/prop-name-casing"));
        assert!(opinionated.has_rule("vue/require-scoped-style"));
        assert!(opinionated.has_rule("vue/sfc-element-order"));
        assert!(opinionated.has_rule("vue/single-style-block"));
        assert!(opinionated.has_rule("vue/v-on-style"));
        assert!(opinionated.has_rule("vue/v-slot-style"));
        assert!(opinionated.has_rule("vapor/no-vue-lifecycle-events"));
        assert!(opinionated.has_rule("type/require-typed-props"));
        assert!(opinionated.has_rule("type/require-typed-emits"));
        assert!(opinionated.has_rule("html/no-empty-palpable-content"));
        assert!(opinionated.has_rule("vue/multi-word-component-names"));
        assert!(opinionated.has_rule("a11y/use-list"));
        assert!(!super::builtin_script_rule_names(LintPreset::HappyPath)
            .contains(&"script/no-options-api"));
        assert!(super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-options-api"));
        assert!(super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-get-current-instance"));
        assert!(super::builtin_script_rule_names(LintPreset::Opinionated)
            .contains(&"script/no-next-tick"));
    }

    fn rule_names(preset: LintPreset) -> Vec<&'static str> {
        let mut rules: Vec<_> = RuleRegistry::with_preset(preset)
            .rules()
            .iter()
            .map(|rule| rule.meta().name)
            .collect();
        rules.extend_from_slice(super::builtin_script_rule_names(preset));
        rules
    }
}
