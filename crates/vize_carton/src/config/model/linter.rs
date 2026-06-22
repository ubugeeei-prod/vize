//! Linter config model.

use serde::{Deserialize, Serialize};

use crate::{FxHashMap, String};

const CATEGORY_LINT_MARKER_PREFIX: &str = "__vize_internal/category/";
const TYPE_AWARE_LINT_MARKER: &str = "__vize_internal/type-aware-lint";

/// Per-rule lint severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintRuleSeverity {
    Off,
    Warn,
    Error,
}

/// Linter settings shared by CLI and IDE linting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LinterConfig {
    pub enabled: bool,
    pub preset: Option<String>,
    pub rules: FxHashMap<String, LintRuleSeverity>,
}

/// Raw linter config with unstable config-only switches.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RawLinterConfig {
    #[serde(flatten)]
    config: LinterConfig,
    type_aware: bool,
    categories: FxHashMap<String, LintRuleSeverity>,
}

impl LinterConfig {
    /// Returns true when the config matches the built-in defaults.
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }

    /// Whether the host wants the strict-reactivity rule enabled. The rule
    /// shows up when `type/no-reactivity-loss` is explicitly configured to
    /// `warn` or `error` (rather than `off`). CLI users opt in via
    /// `--strict-reactivity`; LSP users opt in via the same rule entry.
    pub fn strict_reactivity_enabled(&self) -> bool {
        self.rules
            .get("type/no-reactivity-loss")
            .map(|severity| !matches!(severity, LintRuleSeverity::Off))
            .unwrap_or(false)
    }

    /// Whether native type-aware lint should run.
    ///
    /// The config-only `typeAware` switch enables preset-provided type-aware
    /// rules. An explicitly enabled `type/*` rule also counts as an opt-in.
    pub fn type_aware_lint_enabled(&self) -> bool {
        self.rules.contains_key(TYPE_AWARE_LINT_MARKER)
            || self.rules.iter().any(|(rule, severity)| {
                is_public_rule(rule)
                    && rule.starts_with("type/")
                    && !matches!(severity, LintRuleSeverity::Off)
            })
    }

    fn enable_type_aware_lint(&mut self) {
        self.rules
            .insert(TYPE_AWARE_LINT_MARKER.into(), LintRuleSeverity::Warn);
    }

    /// Rule names explicitly disabled by config.
    pub fn disabled_rules(&self) -> Vec<String> {
        let mut rules = self
            .rules
            .iter()
            .filter(|(rule, severity)| {
                is_public_rule(rule) && matches!(severity, LintRuleSeverity::Off)
            })
            .map(|(rule, _)| rule.clone())
            .collect::<Vec<_>>();
        rules.sort();
        rules
    }

    /// Rule-level severities explicitly configured as `warn` or `error`.
    pub fn rule_severity_overrides(&self) -> Vec<(String, LintRuleSeverity)> {
        let mut rules = self
            .rules
            .iter()
            .filter(|(rule, severity)| {
                is_public_rule(rule) && !matches!(severity, LintRuleSeverity::Off)
            })
            .map(|(rule, severity)| (rule.clone(), *severity))
            .collect::<Vec<_>>();
        rules.sort_by(|(left, _), (right, _)| left.cmp(right));
        rules
    }

    /// Rule names explicitly enabled by config.
    pub fn enabled_rules(&self) -> Vec<String> {
        let mut rules = self
            .rules
            .iter()
            .filter(|(rule, severity)| {
                is_public_rule(rule) && !matches!(severity, LintRuleSeverity::Off)
            })
            .map(|(rule, _)| rule.clone())
            .collect::<Vec<_>>();
        rules.sort();
        rules
    }

    /// Rule categories explicitly disabled by config.
    pub fn disabled_categories(&self) -> Vec<String> {
        let mut categories = self
            .rules
            .iter()
            .filter_map(|(rule, severity)| {
                let category = category_marker_name(rule)?;
                matches!(severity, LintRuleSeverity::Off).then(|| String::from(category))
            })
            .collect::<Vec<_>>();
        categories.sort();
        categories
    }

    /// Category-level severities explicitly configured as `warn` or `error`.
    pub fn category_severity_overrides(&self) -> Vec<(String, LintRuleSeverity)> {
        let mut categories = self
            .rules
            .iter()
            .filter_map(|(rule, severity)| {
                let category = category_marker_name(rule)?;
                (!matches!(severity, LintRuleSeverity::Off))
                    .then(|| (String::from(category), *severity))
            })
            .collect::<Vec<_>>();
        categories.sort_by(|(left, _), (right, _)| left.cmp(right));
        categories
    }
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preset: None,
            rules: FxHashMap::default(),
        }
    }
}

impl From<RawLinterConfig> for LinterConfig {
    fn from(raw: RawLinterConfig) -> Self {
        let mut config = raw.config;
        if raw.type_aware {
            config.enable_type_aware_lint();
        }
        for (category, severity) in raw.categories {
            config.rules.insert(
                crate::cstr!("{CATEGORY_LINT_MARKER_PREFIX}{category}"),
                severity,
            );
        }
        config
    }
}

fn is_public_rule(rule: &str) -> bool {
    rule != TYPE_AWARE_LINT_MARKER && category_marker_name(rule).is_none()
}

fn category_marker_name(rule: &str) -> Option<&str> {
    rule.strip_prefix(CATEGORY_LINT_MARKER_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::{LintRuleSeverity, LinterConfig, RawLinterConfig};

    #[test]
    fn type_aware_lint_defaults_to_disabled() {
        assert!(!LinterConfig::default().type_aware_lint_enabled());
    }

    #[test]
    fn type_aware_lint_can_be_enabled_as_a_group() {
        let mut config = LinterConfig::default();
        config.enable_type_aware_lint();

        assert!(config.type_aware_lint_enabled());
        assert!(config.enabled_rules().is_empty());
    }

    #[test]
    fn raw_type_aware_deserializes_as_group_opt_in() {
        let raw = serde_json::from_str::<RawLinterConfig>(r#"{ "typeAware": true }"#).unwrap();
        let config = LinterConfig::from(raw);

        assert!(config.type_aware_lint_enabled());
        assert!(config.enabled_rules().is_empty());
    }

    #[test]
    fn enabled_type_rule_counts_as_type_aware_opt_in() {
        let mut config = LinterConfig::default();
        config.rules.insert(
            "type/no-unsafe-template-binding".into(),
            LintRuleSeverity::Warn,
        );

        assert!(config.type_aware_lint_enabled());
    }

    #[test]
    fn disabled_type_rule_does_not_opt_in() {
        let mut config = LinterConfig::default();
        config
            .rules
            .insert("type/no-reactivity-loss".into(), LintRuleSeverity::Off);

        assert!(!config.type_aware_lint_enabled());
    }

    #[test]
    fn rule_severity_overrides_include_warn_and_error_rules() {
        let mut config = LinterConfig::default();
        config
            .rules
            .insert("html/id-duplication".into(), LintRuleSeverity::Warn);
        config
            .rules
            .insert("vue/permitted-contents".into(), LintRuleSeverity::Error);
        config
            .rules
            .insert("vue/require-scoped-style".into(), LintRuleSeverity::Off);

        assert_eq!(
            config.rule_severity_overrides(),
            [
                ("html/id-duplication".into(), LintRuleSeverity::Warn),
                ("vue/permitted-contents".into(), LintRuleSeverity::Error),
            ]
        );
        assert_eq!(config.disabled_rules(), ["vue/require-scoped-style"]);
    }

    #[test]
    fn category_overrides_split_disabled_from_severity_overrides() {
        let raw = serde_json::from_str::<RawLinterConfig>(
            r#"{ "categories": { "style": "off", "a11y": "warn" } }"#,
        )
        .unwrap();
        let config = LinterConfig::from(raw);

        assert_eq!(config.disabled_categories(), ["style"]);
        assert_eq!(
            config.category_severity_overrides(),
            [("a11y".into(), LintRuleSeverity::Warn)]
        );
    }
}
