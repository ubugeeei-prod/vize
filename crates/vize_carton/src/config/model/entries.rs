//! Per-entry linter scopes derived from `entries[]`.

use serde::Deserialize;

use super::linter::{LintRuleSeverity, LinterConfig, RawLinterConfig};
use super::linter_rule_options::LintRuleOptions;
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
