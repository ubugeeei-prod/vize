use super::LintResult;
use crate::diagnostic::Severity;
use vize_l0::{FxHashMap, String};

pub(crate) fn append_with_rule_overrides(
    result: &mut LintResult,
    mut diagnostics: Vec<crate::diagnostic::LintDiagnostic>,
    overrides: &FxHashMap<String, Severity>,
) {
    if diagnostics.is_empty() {
        return;
    }

    for diagnostic in &mut diagnostics {
        if let Some(severity) = overrides.get(diagnostic.rule_name) {
            diagnostic.severity = *severity;
        }
    }
    result.diagnostics.extend(diagnostics);
    recount(result);
}

pub(crate) fn apply_severity_overrides(
    result: &mut LintResult,
    overrides: &FxHashMap<String, Severity>,
) {
    if overrides.is_empty() || result.diagnostics.is_empty() {
        return;
    }

    let mut changed = false;
    for diagnostic in &mut result.diagnostics {
        if diagnostic.rule_name.starts_with("type/")
            && let Some(severity) = overrides.get(diagnostic.rule_name)
            && diagnostic.severity != *severity
        {
            diagnostic.severity = *severity;
            changed = true;
        }
    }
    if changed {
        recount(result);
    }
}

fn recount(result: &mut LintResult) {
    result.error_count = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    result.warning_count = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Warning)
        .count();
}
