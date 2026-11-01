//! Resolve the same document's lint policy for a formatter request.

use super::{
    LinterConfigPlanWithConfigRuleOptions, ResolvedLinterConfigWithConfigRuleOptions,
    matcher::{LintPlanScope, ProjectIgnoreSet, absolute_path},
};
use std::path::{Path, PathBuf};

/// Scope matching only; rule activation remains the lint engine's authority.
pub struct FormatterLinterPlan {
    plan: LinterConfigPlanWithConfigRuleOptions,
    scopes: Vec<LintPlanScope>,
    ignores: Option<ProjectIgnoreSet>,
    cwd: PathBuf,
}

impl FormatterLinterPlan {
    pub fn new(plan: LinterConfigPlanWithConfigRuleOptions, config_dir: &Path) -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let scopes = plan
            .plan
            .entries
            .iter()
            .map(|entry| {
                LintPlanScope::new(
                    entry.base_path.as_deref(),
                    entry.files.as_deref(),
                    &entry.ignores,
                    config_dir,
                    &cwd,
                )
            })
            .collect();
        let ignores = ProjectIgnoreSet::new(&plan.plan.global_ignores, config_dir);
        Self {
            plan,
            scopes,
            ignores,
            cwd,
        }
    }

    /// Ignored files have no active lint policy to conflict with formatting.
    pub fn resolve(&self, path: &Path) -> Option<ResolvedLinterConfigWithConfigRuleOptions> {
        let path = absolute_path(path, &self.cwd);
        if self
            .ignores
            .as_ref()
            .is_some_and(|ignores| ignores.is_ignored(&path))
        {
            return None;
        }
        let matching = self
            .scopes
            .iter()
            .enumerate()
            .filter_map(|(index, scope)| scope.matches(&path).then_some(index))
            .collect::<Vec<_>>();
        Some(self.plan.resolve_matching_entries(&matching))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigDocument, LintRuleSeverity, VBindStyle};

    #[test]
    fn retains_declaration_order_options_negations_and_ignored_files() {
        let directory = tempfile::tempdir().unwrap();
        let document: ConfigDocument = serde_json::from_str(
            r#"{
              "ignores":["ignored/**", "!ignored/Keep.vue"],
              "linter":{"preset":"incremental","rules":{"vue/v-bind-style":"error"},
                "ruleOptions":{"vue/v-bind-style":"longform"}},
              "entries":[
                {"files":["src/**", "!src/Keep.vue"],
                  "linter":{"rules":{"vue/v-bind-style":"off"}}},
                {"files":["src/active/**"],
                  "linter":{"rules":{"vue/v-bind-style":"warn"}}},
                {"basePath":"packages/ui","files":["**/*.vue"],
                  "linter":{"ruleOptions":{"vue/v-bind-style":"shorthand"}}}
              ]
            }"#,
        )
        .unwrap();
        let plan = FormatterLinterPlan::new(
            document.linter_plan_with_config_rule_options(),
            directory.path(),
        );
        for (file, severity, style) in [
            ("src/Off.vue", LintRuleSeverity::Off, VBindStyle::Longform),
            (
                "src/Keep.vue",
                LintRuleSeverity::Error,
                VBindStyle::Longform,
            ),
            (
                "src/active/On.vue",
                LintRuleSeverity::Warn,
                VBindStyle::Longform,
            ),
            (
                "packages/ui/Card.vue",
                LintRuleSeverity::Error,
                VBindStyle::Shorthand,
            ),
            (
                "outside/Card.vue",
                LintRuleSeverity::Error,
                VBindStyle::Longform,
            ),
        ] {
            let resolved = plan.resolve(&directory.path().join(file)).unwrap();
            assert_eq!(
                resolved.config.rules.get("vue/v-bind-style"),
                Some(&severity)
            );
            assert_eq!(resolved.rule_options.v_bind_style(), Some(style));
        }
        assert!(
            plan.resolve(&directory.path().join("ignored/Drop.vue"))
                .is_none()
        );
        assert!(
            plan.resolve(&directory.path().join("ignored/Keep.vue"))
                .is_some()
        );
    }
}
