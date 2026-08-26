use super::LintResult;
use crate::diagnostic::Severity;
use vize_s0::{FxHashMap, String};

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
