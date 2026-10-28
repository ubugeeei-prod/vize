//! Apply the already-resolved host policy while appending project diagnostics.

use std::path::{Path, PathBuf};
use vize_l0::{FxHashMap, String, config::LintRuleSeverity, cstr};
use vize_patina::{LintDiagnostic, Severity};

use super::super::entry_rules::ResolvedLinterRuleGroups;

#[derive(Clone, Copy)]
struct Policy<'a> {
    rules: &'a FxHashMap<String, LintRuleSeverity>,
    category: Option<LintRuleSeverity>,
    report_unused_disable_directives: bool,
}

pub(in crate::commands::lint) struct CrossFileRuleSettings<'a> {
    files: FxHashMap<&'a Path, Policy<'a>>,
}

pub(in crate::commands::lint) enum RuleSetting {
    Off,
    Default,
    Severity(Severity),
}

impl<'a> CrossFileRuleSettings<'a> {
    pub(in crate::commands::lint) fn new(
        files: &'a [PathBuf],
        resolved: &'a ResolvedLinterRuleGroups,
        report_unused_disable_directives: bool,
    ) -> Result<Self, String> {
        if files.len() != resolved.file_config_indices.len() {
            return Err("Resolved lint configuration does not cover every input file".into());
        }
        let policies: Vec<_> = resolved
            .configs
            .iter()
            .map(|config| Policy {
                rules: &config.rules,
                report_unused_disable_directives: report_unused_disable_directives
                    || config.report_unused_disable_directives(),
                category: if config
                    .disabled_categories()
                    .iter()
                    .any(|name| name == "suspicious")
                {
                    Some(LintRuleSeverity::Off)
                } else {
                    config
                        .category_severity_overrides()
                        .into_iter()
                        .find_map(|(name, severity)| (name == "suspicious").then_some(severity))
                },
            })
            .collect();
        let files = files
            .iter()
            .zip(&resolved.file_config_indices)
            .map(|(path, index)| {
                policies
                    .get(*index)
                    .map(|policy| (path.as_path(), *policy))
                    .ok_or_else(|| cstr!("Invalid resolved lint configuration index {index}"))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { files })
    }
}

pub(in crate::commands::lint) fn setting(
    settings: Option<&CrossFileRuleSettings<'_>>,
    path: &Path,
    rule: &str,
) -> RuleSetting {
    let Some(policy) = settings.and_then(|settings| settings.files.get(path)) else {
        return RuleSetting::Default;
    };
    let rule = policy.rules.get(rule).copied();
    let group = policy.rules.get("cross-file").copied();
    // Disabled groups/categories, like Patina's disabled categories, stay disabled.
    if [rule, group, policy.category].contains(&Some(LintRuleSeverity::Off)) {
        return RuleSetting::Off;
    }
    match rule.or(group).or(policy.category) {
        Some(LintRuleSeverity::Warn) => RuleSetting::Severity(Severity::Warning),
        Some(LintRuleSeverity::Error) => RuleSetting::Severity(Severity::Error),
        Some(LintRuleSeverity::Off) | None => RuleSetting::Default,
    }
}

pub(in crate::commands::lint) fn configure(
    settings: Option<&CrossFileRuleSettings<'_>>,
    path: &Path,
    rule: &str,
    mut diagnostic: LintDiagnostic,
) -> Option<LintDiagnostic> {
    match setting(settings, path, rule) {
        RuleSetting::Off => None,
        RuleSetting::Default => Some(diagnostic),
        RuleSetting::Severity(severity) => {
            diagnostic.severity = severity;
            Some(diagnostic)
        }
    }
}

pub(in crate::commands::lint) fn report_unused(
    settings: Option<&CrossFileRuleSettings<'_>>,
    path: &Path,
) -> bool {
    settings
        .and_then(|settings| settings.files.get(path))
        .is_some_and(|policy| policy.report_unused_disable_directives)
}
