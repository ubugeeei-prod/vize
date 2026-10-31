use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use vize_carton::i18n::translator;
use vize_l0::String;
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale, Severity};

pub const RULE: &str = "vue/no-deprecated-slot-attribute";
pub const SOURCE: &str = "<template><div slot=\"header\">Title</div></template>";
pub const MESSAGE: &str = "the `slot` attribute was deprecated in Vue 2.6 and removed in Vue 3";
pub const HELP: &str = "Use `v-slot` instead (e.g. `<template v-slot:header>`).";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    id: String,
    history: String,
    pub source: String,
    pub filename: String,
    entry: String,
    rule: String,
    vue_version: Option<String>,
    vapor: Option<bool>,
    diagnostics: usize,
    fixes: usize,
}

#[derive(Debug)]
pub struct Application {
    diagnostic_index: usize,
    source: String,
    result: LintResult,
}

#[derive(Debug)]
pub struct Observation {
    pub(super) initial: LintResult,
    pub(super) applications: Vec<Application>,
    pub(super) unchanged_requery: Option<LintResult>,
}

pub fn history_case(id: &str) -> Case {
    serde_json::from_str::<Vec<Case>>(include_str!("../fixtures/lint-history/cases.json"))
        .unwrap()
        .into_iter()
        .find(|case| case.id == id)
        .unwrap()
}

pub fn configured(locale: Locale, help: HelpLevel) -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(locale)
        .with_help_level(help)
        .with_vue_version(Some(vize_l0::config::VueVersion::V3))
        .with_vapor_mode(None)
}

pub fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result.diagnostics.iter().map(|diagnostic| {
        let labels: Vec<_> = diagnostic.labels.iter().map(|label| json!({
            "message": label.message.as_str(), "start": label.start, "end": label.end,
        })).collect();
        json!({
            "rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
            "message": diagnostic.message.as_str(), "start": diagnostic.start, "end": diagnostic.end,
            "help": diagnostic.help.as_ref().map(|help| help.as_str()),
            "labels": labels, "fix": diagnostic.fix,
        })
    }).collect();
    json!({
        "filename": result.filename.as_str(), "diagnostics": diagnostics,
        "error_count": result.error_count, "warning_count": result.warning_count,
    })
}

pub fn expected(
    filename: &str,
    start: u32,
    severity: Severity,
    locale: Locale,
    help: HelpLevel,
) -> Value {
    let message = translator().get(locale, "vue/no-deprecated-slot-attribute.message");
    let full_help = translator().get(locale, "vue/no-deprecated-slot-attribute.help");
    if locale == Locale::En {
        assert_eq!(message.as_ref(), MESSAGE);
        assert_eq!(full_help.as_ref(), HELP);
    }
    json!({
        "filename": filename,
        "diagnostics": [{
            "rule_name": RULE, "severity": severity, "message": message.as_ref(),
            "start": start, "end": start + 4,
            "help": help.process(full_help.as_ref()).map(|help| help.to_string()),
            "labels": [], "fix": null,
        }],
        "error_count": usize::from(severity == Severity::Error),
        "warning_count": usize::from(severity == Severity::Warning),
    })
}

pub fn assert_pair(linter: &Linter, source: &str, filename: &str, expected: &Value) {
    let original = linter.lint_sfc(source, filename);
    let native = linter.lint_native_sfc(source, filename).unwrap();
    assert_eq!(complete(&original), *expected, "legacy {source}");
    assert_eq!(complete(&native), *expected, "native {source}");
    assert_eq!(format!("{native:#?}"), format!("{original:#?}"));
    let legacy_requery = linter.lint_sfc(source, filename);
    let native_requery = linter.lint_native_sfc(source, filename).unwrap();
    assert_eq!(
        format!("{native_requery:#?}"),
        format!("{legacy_requery:#?}")
    );
    assert_eq!(format!("{native:#?}"), format!("{native_requery:#?}"));
}

pub fn observe(linter: &Linter, case: &Case, native: bool) -> Observation {
    let query = |source: &str| {
        if native {
            linter.lint_native_sfc(source, &case.filename).unwrap()
        } else {
            linter.lint_sfc(source, &case.filename)
        }
    };
    let initial = query(&case.source);
    assert_eq!(initial.diagnostics.len(), case.diagnostics);
    let applications: Vec<_> = initial
        .diagnostics
        .iter()
        .enumerate()
        .filter_map(|(diagnostic_index, diagnostic)| {
            diagnostic.fix.as_ref().map(|fix| {
                let source = fix.apply(&case.source);
                let result = query(&source);
                Application {
                    diagnostic_index,
                    source,
                    result,
                }
            })
        })
        .collect();
    assert_eq!(applications.len(), case.fixes);
    for application in &applications {
        assert!(application.diagnostic_index < initial.diagnostics.len());
        assert_eq!(
            complete(&application.result),
            complete(&query(&application.source))
        );
    }
    let unchanged_requery = applications.is_empty().then(|| query(&case.source));
    Observation {
        initial,
        applications,
        unchanged_requery,
    }
}
