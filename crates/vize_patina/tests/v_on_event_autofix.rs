//! Independent whole-output and argument-edit contracts from eslint-plugin-vue10.9.2.

use serde_json::{Value, json};
use std::path::PathBuf;
use vize_l0::config::ConfigLintRuleOptions;
use vize_patina::{LintPreset, LintResult, Linter, Severity};

const RULE: &str = "vue/v-on-event-hyphenation";

fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/v-on-event-autofix-6097")
}

fn utf16_byte(source: &str, position: u64) -> u32 {
    let mut offset = 0;
    for (byte, character) in source.char_indices() {
        if offset == position {
            return byte as u32;
        }
        offset += character.len_utf16() as u64;
        assert!(
            offset <= position,
            "reference must not split a surrogate pair"
        );
    }
    assert_eq!(offset, position);
    source.len() as u32
}

fn location(source: &str, line: u64, column: u64) -> u32 {
    let start: usize = source
        .split_inclusive('\n')
        .take(line as usize - 1)
        .map(str::len)
        .sum();
    start as u32 + utf16_byte(&source[start..], column - 1)
}

fn policy(options: &Value) -> ConfigLintRuleOptions {
    let options = options.as_array().unwrap();
    if options.is_empty() {
        return ConfigLintRuleOptions::default();
    }
    let value = if let Some(payload) = options.get(1) {
        let mut payload = payload.as_object().unwrap().clone();
        payload.insert("style".into(), options[0].clone());
        Value::Object(payload)
    } else {
        options[0].clone()
    };
    serde_json::from_value(json!({ RULE: value })).unwrap()
}

fn compare(source: &str, result: &LintResult, findings: &Value, id: &str) {
    let findings = findings.as_array().unwrap();
    assert_eq!(result.error_count, 0, "{id}");
    assert_eq!(result.warning_count, findings.len(), "{id}");
    assert_eq!(result.diagnostics.len(), findings.len(), "{id}");
    for (diagnostic, finding) in result.diagnostics.iter().zip(findings) {
        assert_eq!(diagnostic.rule_name, RULE, "{id}");
        assert_eq!(diagnostic.severity, Severity::Warning, "{id}");
        assert_eq!(
            diagnostic.start,
            location(
                source,
                finding["line"].as_u64().unwrap(),
                finding["column"].as_u64().unwrap()
            ),
            "{id}"
        );
        assert_eq!(
            diagnostic.end,
            location(
                source,
                finding["endLine"].as_u64().unwrap(),
                finding["endColumn"].as_u64().unwrap()
            ),
            "{id}"
        );
        match finding.get("fix") {
            None => assert!(diagnostic.fix.is_none(), "{id}"),
            Some(fix) => {
                let actual = diagnostic.fix.as_ref().unwrap();
                assert_eq!(actual.edits.len(), 1, "{id}");
                let edit = &actual.edits[0];
                assert_eq!(
                    edit.start,
                    utf16_byte(source, fix["range"][0].as_u64().unwrap()),
                    "{id}"
                );
                assert_eq!(
                    edit.end,
                    utf16_byte(source, fix["range"][1].as_u64().unwrap()),
                    "{id}"
                );
                assert_eq!(
                    edit.new_text.as_str(),
                    fix["text"].as_str().unwrap(),
                    "{id}"
                );
            }
        }
    }
}

fn apply(source: &str, result: &LintResult) -> String {
    let mut edits: Vec<_> = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.fix.as_ref())
        .flat_map(|fix| fix.edits.iter())
        .collect();
    edits.sort_by_key(|edit| std::cmp::Reverse((edit.start, edit.end)));
    let mut output = source.to_owned();
    for edit in edits {
        output.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    output
}

#[test]
fn all_twenty_two_whole_reference_outputs_and_requeries_preserve_static_argument_spans() {
    let root = directory();
    let corpus: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("cases.json")).unwrap()).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 22);
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let source = std::fs::read_to_string(root.join(case["file"].as_str().unwrap())).unwrap();
        let reference: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(case["reference"]["file"].as_str().unwrap()))
                .unwrap(),
        )
        .unwrap();
        let filename = case["file"].as_str().unwrap().trim_end_matches(".txt");
        let linter = Linter::with_preset(LintPreset::Opinionated)
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_config_rule_options(&policy(&case["options"]));
        let mut result = linter.lint_sfc(&source, filename);
        compare(&source, &result, &reference["messages"], id);
        let mut output = source.clone();
        for _ in 0..10 {
            let next = apply(&output, &result);
            if next == output {
                break;
            }
            output = next;
            result = linter.lint_sfc(&output, filename);
        }
        assert_eq!(
            output,
            reference["fixResult"]["output"].as_str().unwrap(),
            "{id}"
        );
        compare(&output, &result, &reference["requery"], id);
        let again = linter.lint_sfc(&output, filename);
        compare(
            &output,
            &again,
            &reference["secondFixResult"]["messages"],
            id,
        );
        assert_eq!(apply(&output, &again), output, "{id}");
        // The Unicode identity-edit frontier has repeated ESLint `fixed:true`.
        // Whole bytes and remaining diagnostics above are the contract here.
    }
}

#[test]
fn opt_in_does_not_enable_a_disabled_rule_and_legacy_replacement_resets_fixes() {
    let source = r#"<template><MyWidget @saveItem="save" /></template>"#;
    let options: ConfigLintRuleOptions =
        serde_json::from_value(json!({RULE: {"autofix": true}})).unwrap();
    let disabled = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![]))
        .with_config_rule_options(&options)
        .lint_sfc(source, "App.vue");
    assert!(disabled.diagnostics.is_empty());
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_config_rule_options(&options)
        .with_v_on_event_hyphenation_options(
            vize_patina::rules::VOnEventHyphenationStyle::Always,
            vec![],
        );
    assert!(
        linter.lint_sfc(source, "App.vue").diagnostics[0]
            .fix
            .is_none()
    );
}
