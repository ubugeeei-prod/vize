//! #12999: the remaining typescript-eslint `no-unused-vars` corpus differences.

use crate::diagnostic::Severity;
use crate::{LintPreset, LintResult, Linter};
use serde_json::{Value, json};
use vize_l0::config::ConfigLintRuleOptions;

const RULE: &str = "vue/no-unused-script-vars";
const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unused-script-vars-12999/corpus.json"
));

fn source(code: &str) -> String {
    format!("<script setup lang=\"ts\">\n{code}\n</script>\n\n<template><div /></template>\n")
}

fn options(value: Value) -> ConfigLintRuleOptions {
    serde_json::from_value(json!({ "vue/no-unused-script-vars": value })).unwrap()
}

fn lint(source: &str, options: &ConfigLintRuleOptions) -> LintResult {
    Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_config_rule_options(options)
        .lint_sfc(source, "Reported.vue")
}

fn packet(result: &LintResult) -> Value {
    json!({
        "filename": result.filename,
        "errorCount": result.error_count,
        "warningCount": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|diagnostic| json!({
            "rule": diagnostic.rule_name,
            "severity": diagnostic.severity,
            "message": diagnostic.message.as_str(),
            "start": diagnostic.start,
            "end": diagnostic.end,
            "help": diagnostic.help.as_deref(),
            "labels": diagnostic.labels.iter().map(|label| json!({
                "message": label.message.as_str(),
                "start": label.start,
                "end": label.end,
            })).collect::<Vec<_>>(),
            "fix": diagnostic.fix.as_ref(),
        })).collect::<Vec<_>>(),
    })
}

fn expected(source: &str, name: &str, marker: &str, message: &str) -> Value {
    let marker_start = source.find(marker).unwrap();
    let name_start = marker.rfind(name).unwrap();
    let start = marker_start + name_start;
    json!({
        "rule": RULE,
        "severity": Severity::Warning,
        "message": message,
        "start": start,
        "end": start + name.len(),
        "help": null,
        "labels": [],
        "fix": null,
    })
}

fn expected_packet(diagnostics: Vec<Value>) -> Value {
    json!({
        "filename": "Reported.vue",
        "errorCount": 0,
        "warningCount": diagnostics.len(),
        "diagnostics": diagnostics,
    })
}

#[test]
fn all_eleven_reported_global_comment_cases_have_complete_packets() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    assert_eq!(corpus["schema"], "vize.no-unused-script-vars.corpus");
    assert_eq!(corpus["typescriptEslint"], "8.72.2");
    let options = options(corpus["options"].clone());
    let cases = corpus["globalCases"].as_array().unwrap();
    assert_eq!(cases.len(), 11);
    for case in cases {
        let code = case["code"].as_str().unwrap();
        let source = source(code);
        let diagnostics = case["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|diagnostic| {
                let name = diagnostic["name"].as_str().unwrap();
                let marker = diagnostic["marker"].as_str().unwrap();
                expected(
                    &source,
                    name,
                    marker,
                    &format!(
                        "'{name}' is defined but never used. Allowed unused vars must match /^_/u"
                    ),
                )
            })
            .collect();
        assert_eq!(
            packet(&lint(&source, &options)),
            expected_packet(diagnostics),
            "{code}"
        );
    }
}

#[test]
fn setter_and_stored_nested_callback_are_not_reported() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let source = source(corpus["quietCase"]["code"].as_str().unwrap());
    let options = options(corpus["options"].clone());
    assert_eq!(packet(&lint(&source, &options)), expected_packet(vec![]));
}

#[test]
fn ordinary_method_parameter_and_direct_iife_self_read_still_report() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let source = source(corpus["oppositeCase"]["code"].as_str().unwrap());
    let options = options(corpus["options"].clone());
    assert_eq!(
        packet(&lint(&source, &options)),
        expected_packet(vec![
            expected(
                &source,
                "a",
                "foo(a)",
                "'a' is defined but never used. Allowed unused args must match /^_/u",
            ),
            expected(
                &source,
                "cb",
                "  cb = (function",
                "'cb' is assigned a value but never used. Allowed unused args must match /^_/u",
            ),
        ])
    );
}

#[test]
fn global_directive_boundaries_options_and_values_preserve_negative_controls() {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let fixture = source(corpus["boundaryCase"]["code"].as_str().unwrap());
    let defaults = options(json!({
        "vars": "all",
        "varsIgnorePattern": "^_"
    }));
    assert_eq!(packet(&lint(&fixture, &defaults)), expected_packet(vec![]));

    let local = options(json!({ "vars": "local", "varsIgnorePattern": "" }));
    let unused_global = source("/*global outside */");
    assert_eq!(
        packet(&lint(&unused_global, &local)),
        expected_packet(vec![])
    );

    let used_ignored = source("/*global ignored */\nignored;");
    let report_used = options(json!({
        "vars": "all",
        "varsIgnorePattern": "^ignored$",
        "reportUsedIgnorePattern": true
    }));
    assert_eq!(
        packet(&lint(&used_ignored, &report_used)),
        expected_packet(vec![expected(
            &used_ignored,
            "ignored",
            "/*global ignored */",
            "'ignored' is marked as ignored but is used. Used vars must not match /^ignored$/u",
        )])
    );
}
