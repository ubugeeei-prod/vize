//! Batch resolution of declaration-ordered `entries[].linter.rules`.

use std::path::{Path, PathBuf};
use vize_carton::{
    FxHashMap, String,
    config::{LintRuleOptions, LintRuleSeverity, LinterConfigPlan, LinterFeatureFlags},
};
use vize_patina::{HelpLevel, LintPreset, Linter, Severity};

use super::LintArgs;
#[cfg(test)]
use crate::lint_plan::matcher::GlobSequence;
use crate::lint_plan::matcher::{LintPlanScope, absolute_path};

pub(super) struct ResolvedLinterRuleGroups {
    pub(super) configs: Vec<crate::config::LinterConfig>,
    pub(super) file_config_indices: Vec<usize>,
}

impl ResolvedLinterRuleGroups {
    pub(super) fn build_linters(
        &self,
        preset: LintPreset,
        help_level: HelpLevel,
        args: &LintArgs,
        features: LinterFeatureFlags,
        rule_options: &LintRuleOptions,
        configured_corsa_path: Option<PathBuf>,
    ) -> Vec<Linter> {
        self.configs
            .iter()
            .map(|config| {
                let type_aware =
                    args.type_aware || args.strict_reactivity || config.type_aware_lint_enabled();
                let mut linter = Linter::with_preset(preset)
                    .with_additional_rules(config.enabled_rules())
                    .with_disabled_rules(config.disabled_rules())
                    .with_disabled_categories(config.disabled_categories())
                    .with_category_severity_overrides(severity_overrides(
                        config.category_severity_overrides(),
                    ))
                    .with_rule_severity_overrides(severity_overrides(
                        config.rule_severity_overrides(),
                    ))
                    .with_help_level(help_level)
                    .with_type_aware_lint(type_aware)
                    .with_vue_version(features.vue_version)
                    .with_vapor_mode(features.vapor)
                    .with_restricted_globals(rule_options.restricted_globals())
                    .with_restricted_members(rule_options.restricted_members());
                #[cfg(not(target_arch = "wasm32"))]
                {
                    linter = linter.with_corsa_path(configured_corsa_path.clone());
                }
                #[cfg(not(target_arch = "wasm32"))]
                if args.strict_reactivity {
                    linter = linter.with_rule(Box::new(
                        vize_patina::rules::type_aware::NoReactivityLoss::new(),
                    ));
                }
                linter
            })
            .collect()
    }
}

fn severity_overrides(entries: Vec<(String, LintRuleSeverity)>) -> Vec<(String, Severity)> {
    entries
        .into_iter()
        .filter_map(|(name, severity)| match severity {
            LintRuleSeverity::Off => None,
            LintRuleSeverity::Warn => Some((name, Severity::Warning)),
            LintRuleSeverity::Error => Some((name, Severity::Error)),
        })
        .collect()
}

pub(super) struct LinterRuleResolver {
    plan: LinterConfigPlan,
    scopes: Vec<LintPlanScope>,
}

impl LinterRuleResolver {
    pub(super) fn new(plan: LinterConfigPlan, config_dir: &Path, cwd: &Path) -> Self {
        let config_dir = absolute_path(config_dir, cwd);
        let scopes = plan
            .entries
            .iter()
            .map(|entry| {
                LintPlanScope::new(
                    entry.base_path.as_deref(),
                    entry.files.as_deref(),
                    &entry.ignores,
                    &config_dir,
                    cwd,
                )
            })
            .collect();
        Self { plan, scopes }
    }

    /// Resolve each distinct matching-entry signature exactly once for the batch.
    pub(super) fn resolve_files(&self, files: &[PathBuf], cwd: &Path) -> ResolvedLinterRuleGroups {
        if self.scopes.is_empty() {
            return ResolvedLinterRuleGroups {
                configs: vec![self.plan.base.clone()],
                file_config_indices: vec![0; files.len()],
            };
        }
        let mut signatures = FxHashMap::<Vec<usize>, usize>::default();
        let mut configs = Vec::new();
        let mut file_config_indices = Vec::with_capacity(files.len());
        for file in files {
            let signature = self.matching_entries(file, cwd);
            let index = match signatures.get(&signature) {
                Some(index) => *index,
                None => {
                    let index = configs.len();
                    configs.push(self.plan.resolve_matching_entries(&signature));
                    signatures.insert(signature, index);
                    index
                }
            };
            file_config_indices.push(index);
        }
        ResolvedLinterRuleGroups {
            configs,
            file_config_indices,
        }
    }

    fn matching_entries(&self, file: &Path, cwd: &Path) -> Vec<usize> {
        let file = absolute_path(file, cwd);
        self.scopes
            .iter()
            .enumerate()
            .filter_map(|(index, scope)| scope.matches(&file).then_some(index))
            .collect()
    }
}

#[cfg(test)]
#[path = "entry_rules_tests.rs"]
mod tests;
