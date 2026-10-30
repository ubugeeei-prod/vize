//! Per-entry linter scopes derived from `entries[]`.

use serde::{Deserialize, Deserializer};

use super::linter_rule_options::{ConfigLintRuleOptions, LintRuleOptions};
use super::{
    compiler::{RawCompilerConfig, RawTemplateSyntaxConfig},
    linter::{LintRuleSeverity, LinterConfig, RawLinterConfig},
};
use crate::String;

#[derive(Debug, Clone, Default)]
pub(crate) struct RawConfigEntry {
    pub base_path: Option<String>,
    pub files: Option<Vec<String>>,
    pub ignores: Option<Vec<String>>,
    pub compiler: RawCompilerConfig,
    pub linter: RawLinterConfig,
    settings_authored: bool,
}

impl RawConfigEntry {
    /// An ignores-only entry affects discovery; an override only scopes its settings.
    pub(crate) fn is_global_ignore(&self) -> bool {
        self.files.is_none() && !self.settings_authored
    }
}

impl<'de> Deserialize<'de> for RawConfigEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Default, Deserialize)]
        #[serde(default, rename_all = "camelCase")]
        struct Entry {
            base_path: Option<String>,
            files: Option<Vec<String>>,
            ignores: Option<Vec<String>>,
            #[serde(deserialize_with = "present_entry_setting")]
            compiler: Option<RawCompilerConfig>,
            #[serde(deserialize_with = "present_entry_setting")]
            linter: Option<RawLinterConfig>,
        }
        let entry = Entry::deserialize(deserializer)?;
        let settings_authored = entry.compiler.is_some() || entry.linter.is_some();
        Ok(Self {
            base_path: entry.base_path,
            files: entry.files,
            ignores: entry.ignores,
            compiler: entry.compiler.unwrap_or_default(),
            linter: entry.linter.unwrap_or_default(),
            settings_authored,
        })
    }
}

/// Retain presence even for `{}` while preserving the existing rejection of `null`.
fn present_entry_setting<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
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

/// One declaration-ordered compiler override scope from `entries[]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerConfigEntry {
    pub base_path: Option<String>,
    pub files: Option<Vec<String>>,
    pub ignores: Vec<String>,
    pub template_syntax: Option<&'static str>,
    pub whitespace: Option<String>,
    pub vapor: Option<bool>,
    pub custom_elements: Vec<String>,
}

impl CompilerConfigEntry {
    pub(crate) fn from_raw(entry: &RawConfigEntry) -> Self {
        Self {
            base_path: entry.base_path.clone(),
            files: entry.files.clone(),
            ignores: entry.ignores.clone().unwrap_or_default(),
            template_syntax: entry
                .compiler
                .template_syntax
                .map(RawTemplateSyntaxConfig::as_str),
            whitespace: entry.compiler.whitespace.as_deref().map(String::from),
            vapor: entry.compiler.vapor,
            custom_elements: entry.compiler.custom_elements.clone(),
        }
    }

    pub(crate) fn has_overrides(&self) -> bool {
        self.template_syntax.is_some()
            || self.whitespace.is_some()
            || self.vapor.is_some()
            || !self.custom_elements.is_empty()
    }
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
