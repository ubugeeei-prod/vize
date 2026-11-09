//! Whole-run lint settings outside the stable rule configuration.

use serde::Deserialize;

/// Opt-in cross-file analysis and the warning limit for native lint runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
#[non_exhaustive]
pub struct LinterExecutionOptions {
    /// Analyze relationships between input components.
    pub cross_file: bool,
    /// Display the provide/inject tree; also enables cross-file analysis.
    pub cross_file_tree: bool,
    /// Display complexity scores; also enables cross-file analysis.
    pub cross_file_complexity: bool,
    /// Fail when warnings exceed this limit. Absent means unlimited.
    pub max_warnings: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::super::linter::RawLinterConfig;
    use crate::{
        FxHashMap,
        config::{LintRuleSeverity, LinterConfig},
    };

    #[test]
    fn raw_lint_settings_preserve_execution_and_activate_the_editor_rule() {
        let raw: RawLinterConfig = serde_json::from_str(
            r#"{
            "crossFile": true, "crossFileTree": true,
            "crossFileComplexity": true, "strictReactivity": true,
            "maxWarnings": 0
        }"#,
        )
        .unwrap();
        let execution = raw.execution();
        assert_eq!(
            (
                execution.cross_file,
                execution.cross_file_tree,
                execution.cross_file_complexity,
                execution.max_warnings
            ),
            (true, true, true, Some(0))
        );
        let config = LinterConfig::from(raw);
        let expected =
            FxHashMap::from_iter([("type/no-reactivity-loss".into(), LintRuleSeverity::Warn)]);
        assert_eq!(config.rules, expected);
        assert!(config.strict_reactivity_enabled());
        assert!(config.type_aware_lint_enabled());
    }

    #[test]
    fn explicit_rule_disable_wins_over_the_config_switch() {
        let raw: RawLinterConfig = serde_json::from_str(
            r#"{
            "strictReactivity": true, "rules": {"type/no-reactivity-loss": "off"}
        }"#,
        )
        .unwrap();
        let config = LinterConfig::from(raw);
        assert_eq!(
            config.rules,
            FxHashMap::from_iter([("type/no-reactivity-loss".into(), LintRuleSeverity::Off)])
        );
        assert!(!config.strict_reactivity_enabled());
    }

    #[test]
    fn negative_warning_limits_are_rejected() {
        assert!(serde_json::from_str::<RawLinterConfig>(r#"{"maxWarnings": -1}"#).is_err());
    }
}
