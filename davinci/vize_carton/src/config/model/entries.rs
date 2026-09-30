//! Per-entry linter scopes derived from `entries[]`.

use serde::Deserialize;

use super::linter::{LintRuleSeverity, LinterConfig, RawLinterConfig};
use super::linter_rule_options::{ConfigLintRuleOptions, LintRuleOptions};
use crate::String;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RawConfigEntry {
    pub base_path: Option<String>,
    pub files: Option<Vec<String>>,
    pub ignores: Option<Vec<String>>,
    pub linter: RawLinterConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntryIgnore {
    pub base_path: Option<String>,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntryFiles {
    pub base_path: Option<String>,
    pub files: Vec<String>,
}

/// One declaration-ordered linter override scope from `entries[]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinterConfigEntry {
    pub base_path: Option<String>,
    pub files: Option<Vec<String>>,
    pub ignores: Vec<String>,
    pub rules: crate::FxHashMap<String, LintRuleSeverity>,
}

/// Top-level linter settings plus the scoped rule maps layered over them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinterConfigPlan {
    pub base: LinterConfig,
    pub entries: Vec<LinterConfigEntry>,
    pub global_ignores: Vec<ConfigEntryIgnore>,
    pub rule_options: LintRuleOptions,
}

/// A linter config resolved with the scoped `ruleOptions` that apply to it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedLinterConfig {
    pub config: LinterConfig,
    pub rule_options: LintRuleOptions,
}

/// Linter settings plus entry-local rule options for CLI lint execution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinterConfigPlanWithRuleOptions {
    pub plan: LinterConfigPlan,
    pub entry_rule_options: Vec<LintRuleOptions>,
}

/// A linter config resolved with all config-only `ruleOptions` included.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedLinterConfigWithConfigRuleOptions {
    pub config: LinterConfig,
    pub rule_options: ConfigLintRuleOptions,
}

/// Linter settings plus the full entry-local rule options used by lint hosts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinterConfigPlanWithConfigRuleOptions {
    pub plan: LinterConfigPlan,
    pub rule_options: ConfigLintRuleOptions,
    pub entry_rule_options: Vec<ConfigLintRuleOptions>,
}

impl From<LinterConfigPlan> for LinterConfigPlanWithRuleOptions {
    fn from(plan: LinterConfigPlan) -> Self {
        let entry_rule_options = vec![LintRuleOptions::default(); plan.entries.len()];
        Self {
            plan,
            entry_rule_options,
        }
    }
}

impl From<LinterConfigPlan> for LinterConfigPlanWithConfigRuleOptions {
    fn from(plan: LinterConfigPlan) -> Self {
        let rule_options = ConfigLintRuleOptions::from_stable_options(plan.rule_options.clone());
        let entry_rule_options = vec![ConfigLintRuleOptions::default(); plan.entries.len()];
        Self {
            plan,
            rule_options,
            entry_rule_options,
        }
    }
}

impl LinterConfigPlan {
    /// Merge a set of matching entries in declaration order, with later rules winning.
    pub fn resolve_matching_entries(&self, matching_entries: &[usize]) -> LinterConfig {
        let mut resolved = self.base.clone();
        let mut matches = vec![false; self.entries.len()];
        for index in matching_entries {
            if let Some(value) = matches.get_mut(*index) {
                *value = true;
            }
        }
        for (matches, entry) in matches.into_iter().zip(&self.entries) {
            if matches {
                resolved.rules.extend(entry.rules.clone());
            }
        }
        resolved
    }
}

impl LinterConfigPlanWithRuleOptions {
    /// Merge matching linter rules and rule options in declaration order.
    pub fn resolve_matching_entries(&self, matching_entries: &[usize]) -> ResolvedLinterConfig {
        let mut rule_options = self.plan.rule_options.clone();
        let mut matches = vec![false; self.plan.entries.len()];
        for index in matching_entries {
            if let Some(value) = matches.get_mut(*index) {
                *value = true;
            }
        }
        for (index, matches) in matches.into_iter().enumerate() {
            if matches && let Some(options) = self.entry_rule_options.get(index) {
                rule_options.merge_from(options);
            }
        }
        ResolvedLinterConfig {
            config: self.plan.resolve_matching_entries(matching_entries),
            rule_options,
        }
    }
}

impl LinterConfigPlanWithConfigRuleOptions {
    /// Merge matching linter rules and full rule options in declaration order.
    pub fn resolve_matching_entries(
        &self,
        matching_entries: &[usize],
    ) -> ResolvedLinterConfigWithConfigRuleOptions {
        let mut rule_options = self.rule_options.clone();
        let mut matches = vec![false; self.plan.entries.len()];
        for index in matching_entries {
            if let Some(value) = matches.get_mut(*index) {
                *value = true;
            }
        }
        for (index, matches) in matches.into_iter().enumerate() {
            if matches && let Some(options) = self.entry_rule_options.get(index) {
                rule_options.merge_from(options);
            }
        }
        ResolvedLinterConfigWithConfigRuleOptions {
            config: self.plan.resolve_matching_entries(matching_entries),
            rule_options,
        }
    }
}
