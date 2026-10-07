//! Apply the already resolved per-file host configuration to project findings.

use std::path::{Path, PathBuf};
use vize_l0::{FxHashMap, config::LintRuleSeverity};
use vize_patina::{LintResult, Severity};

use super::super::entry_rules::ResolvedLinterRuleGroups;
use crate::config::LinterConfig;

pub(super) struct ProjectRuleConfigs<'a> {
    files: FxHashMap<&'a Path, &'a LinterConfig>,
}

impl<'a> ProjectRuleConfigs<'a> {
    pub(super) fn new(files: &'a [PathBuf], rules: &'a ResolvedLinterRuleGroups) -> Self {
        Self {
            files: files
                .iter()
                .zip(&rules.file_config_indices)
                .filter_map(|(path, index)| {
                    rules
                        .configs
                        .get(*index)
                        .map(|config| (path.as_path(), config))
                })
                .collect(),
        }
    }

    pub(super) fn apply(&self, path: &Path, result: &mut LintResult) {
        let Some(config) = self.files.get(path) else {
            return;
        };
        if config.rules.is_empty() {
            return;
        }
        let disabled = config.disabled_categories();
        let categories = config.category_severity_overrides();
        result.diagnostics.retain_mut(|diagnostic| {
            let rule = diagnostic.rule_name;
            let code = if rule == "cross-file" {
                diagnostic
                    .message
                    .split_once(": ")
                    .map(|(code, _)| code)
                    .unwrap_or(rule)
            } else {
                rule
            };
            let category = if rule == "cross-file" {
                "cross-file"
            } else {
                rule.split('/').next().unwrap_or(rule)
            };
            if disabled
                .iter()
                .any(|name| name == category || (category == "ecosystem" && name == "suspicious"))
            {
                return false;
            }
            let configured = config
                .rules
                .get(code)
                .or_else(|| config.rules.get(rule))
                .copied()
                .or_else(|| {
                    categories
                        .iter()
                        .find(|(name, _)| name == category)
                        .map(|(_, severity)| *severity)
                })
                .or_else(|| {
                    categories
                        .iter()
                        .find(|(name, _)| category == "ecosystem" && name == "suspicious")
                        .map(|(_, severity)| *severity)
                });
            let Some(configured) = configured else {
                return true;
            };
            diagnostic.severity = match configured {
                LintRuleSeverity::Off => return false,
                LintRuleSeverity::Warn => Severity::Warning,
                LintRuleSeverity::Error => Severity::Error,
            };
            if let Some(help) = &mut diagnostic.help {
                for badge in [
                    "**ERROR**",
                    "**WARNING**",
                    "**INFO**",
                    "**HINT**",
                    "ERROR",
                    "WARNING",
                    "INFO",
                    "HINT",
                ] {
                    if let Some(tail) = help.strip_prefix(badge) {
                        let label = match (diagnostic.severity, badge.starts_with('*')) {
                            (Severity::Error, true) => "**ERROR**",
                            (Severity::Warning, true) => "**WARNING**",
                            (Severity::Error, false) => "ERROR",
                            (Severity::Warning, false) => "WARNING",
                        };
                        *help = vize_l0::cstr!("{label}{tail}");
                        break;
                    }
                }
            }
            true
        });
        result.error_count = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        result.warning_count = result.diagnostics.len() - result.error_count;
    }
}
