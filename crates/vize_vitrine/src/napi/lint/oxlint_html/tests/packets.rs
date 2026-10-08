use serde_json::{Value, json};

use super::super::{Operation, Projection, profile};

pub(super) fn projection(actual: &Projection) -> Value {
    let rules = actual
        .rules
        .iter()
        .map(|rule| {
            json!({
                "name": rule.name.as_str(),
                "severity": rule.severity,
                "active": rule.active,
                "authored_options": rule.authored_options,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "rules": rules,
        "settings": {
            "locale": actual.settings.locale.as_str(),
            "help_level": actual.settings.help_level.as_str(),
            "preset": actual.settings.preset.as_str(),
        },
        "deny_warnings": actual.deny_warnings,
    })
}

pub(super) fn lint_result(actual: &vize_patina::LintResult) -> Value {
    let diagnostics = actual
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels = diagnostic
                .labels
                .iter()
                .map(|label| {
                    json!({
                        "message": label.message.as_str(),
                        "start": label.start,
                        "end": label.end,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "rule_name": diagnostic.rule_name,
                "severity": diagnostic.severity,
                "message": diagnostic.message.as_str(),
                "start": diagnostic.start,
                "end": diagnostic.end,
                "help": diagnostic.help.as_ref().map(|value| value.as_str()),
                "labels": labels,
                "fix": diagnostic.fix,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "error_count": actual.error_count,
        "warning_count": actual.warning_count,
        "diagnostics": diagnostics,
    })
}

pub(super) fn observation(actual: &Result<Operation, profile::Refusal>) -> Value {
    match actual {
        Ok(operation) => {
            let files = operation
                .files
                .iter()
                .map(|file| {
                    json!({
                        "path": file.path,
                        "filename": file.result.filename.as_str(),
                        "result": lint_result(&file.result),
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "completed": {
                    "selection": operation.selection,
                    "projection": projection(&operation.projection),
                    "files": files,
                    "executed_file_count": operation.executed_file_count,
                    "elapsed": {"seconds":operation.elapsed.as_secs(),"nanoseconds":operation.elapsed.subsec_nanos()},
                }
            })
        }
        Err(refusal) => json!({"refused":refusal}),
    }
}
