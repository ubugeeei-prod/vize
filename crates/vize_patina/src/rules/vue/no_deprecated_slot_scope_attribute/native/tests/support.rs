use crate::{HelpLevel, LintResult, Linter, Locale, RuleRegistry, Severity};
use serde_json::{Value, json};

pub const RULE: &str = "vue/no-deprecated-slot-scope-attribute";
pub const MESSAGE: &str =
    "the `slot-scope` attribute was deprecated in Vue 2.6 and removed in Vue 3";
pub const HELP: &str = "Use `v-slot` instead (e.g. `<template v-slot=\"props\">`).";
pub const SOURCE: &str = "<template><div slot-scope=\"props\">Title</div></template>";

pub fn configured(locale: Locale, help: HelpLevel) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(
        super::super::super::NoDeprecatedSlotScopeAttribute,
    ));
    Linter::with_registry(registry)
        .with_locale(locale)
        .with_help_level(help)
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
    starts: &[u32],
    severity: Severity,
    locale: Locale,
    help: HelpLevel,
) -> Value {
    // Exact independent catalogue vectors for the three finite locale controls.
    // Expected output needs no additional host-runtime translator dependency.
    let (message, full_help) = match locale {
        Locale::En => (MESSAGE, HELP),
        Locale::Ja => (
            "`slot-scope` 属性は Vue 2.6 で非推奨となり Vue 3 で削除されました",
            "代わりに `v-slot` を使用してください（例: `<template v-slot=\"props\">`）。",
        ),
        Locale::Zh => (
            "`slot-scope` 属性在 Vue 2.6 中已弃用，并在 Vue 3 中移除",
            "请改用 `v-slot`（例如 `<template v-slot=\"props\">`）。",
        ),
        _ => panic!("unprovided test locale"),
    };
    let diagnostics: Vec<_> = starts
        .iter()
        .map(|start| {
            json!({
                "rule_name": RULE, "severity": severity, "message": message,
                "start": start, "end": start + 10,
                "help": help.process(full_help).map(|help| help.to_string()),
                "labels": [], "fix": null,
            })
        })
        .collect();
    json!({
        "filename": filename, "diagnostics": diagnostics,
        "error_count": if severity == Severity::Error {starts.len()} else {0},
        "warning_count": if severity == Severity::Warning {starts.len()} else {0},
    })
}

pub fn assert_pair(linter: &Linter, source: &str, filename: &str, expected: &Value) {
    let legacy = linter.lint_sfc(source, filename);
    let native = linter.lint_native_sfc(source, filename).unwrap();
    assert_eq!(complete(&legacy), *expected, "legacy {source}");
    assert_eq!(complete(&native), *expected, "native {source}");
    assert_eq!(format!("{legacy:#?}"), format!("{native:#?}"));
    assert!(
        legacy
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.fix.is_none())
    );
    for requery in [
        linter.lint_sfc(source, filename),
        linter.lint_native_sfc(source, filename).unwrap(),
    ] {
        assert_eq!(complete(&requery), *expected);
        assert_eq!(format!("{requery:#?}"), format!("{legacy:#?}"));
    }
}

pub fn corpus_case(id: &str) -> Value {
    let corpus: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/deprecation-report-spans/cases.json"
    )))
    .unwrap();
    assert_eq!(corpus["referencePackage"], "eslint-plugin-vue");
    assert_eq!(corpus["referenceVersion"], "10.9.2");
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 24);
    corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap()
        .clone()
}
