//! Removed instance slots use the existing canonical template ownership packet.

use serde_json::{Value, json};
use vize_patina::{LintPreset, LintResult, Linter};

const RULE: &str = "script/no-deprecated-dollar-scopedslots-api";

fn whole(result: &LintResult) -> Value {
    json!({
        "filename": result.filename,
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|l| json!({
                "message": l.message, "start": l.start, "end": l.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

#[test]
fn scoped_slots_template_packets_preserve_component_ownership_and_physical_ranges() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/lint-scoped-slots-template/cases.json"
    ))
    .unwrap();
    let original: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/n8n-cli-config/script-rule-research.json"
    ))
    .unwrap();
    let original = original["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == corpus["origin"]["originalCaseId"])
        .unwrap();
    assert_eq!(corpus["cases"][0]["source"], original["source"]);
    assert_eq!(
        corpus["origin"]["originalCliExpectation"],
        original["cliExpectations"][0]
    );
    let linter =
        Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]));
    for case in corpus["cases"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let diagnostics = &case["nativeDiagnostics"];
        assert_eq!(
            whole(&linter.lint_sfc(source, filename)),
            json!({
                "filename": filename, "error_count": diagnostics.as_array().unwrap().len(),
                "warning_count": 0, "diagnostics": diagnostics
            }),
            "{}",
            case["id"]
        );
    }
    let case = &corpus["cases"][0];
    let filename = case["filename"].as_str().unwrap();
    let disabled = linter.with_disabled_rules(vec![RULE.into()]);
    assert_eq!(
        whole(&disabled.lint_sfc(case["source"].as_str().unwrap(), filename)),
        json!({"filename": filename, "error_count": 0, "warning_count": 0, "diagnostics": []})
    );
}

fn reporting_linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_report_unused_disable_directives(true)
}

#[test]
fn used_template_pragmas_share_actual_capability_usage() {
    let linter = reporting_linter();
    for marker in ["eslint", "oxlint"] {
        for rules in [
            "",
            " vue/no-deprecated-dollar-scopedslots-api",
            " script/no-deprecated-dollar-scopedslots-api",
        ] {
            for script in [
                "",
                "<script>export default {}</script>\r\n",
                "<script setup>const value = 1;</script>\r\n",
            ] {
                let source = vize_l0::cstr!(
                    "{script}<template>\r\n<!-- {marker}-disable-next-line{rules} -->\r\n<div>{{{{ $scopedSlots }}}}</div>\r\n</template>\r\n"
                );
                assert_eq!(
                    whole(&linter.lint_sfc(&source, "Used.vue")),
                    json!({
                        "filename":"Used.vue", "error_count":0, "warning_count":0, "diagnostics":[]
                    }),
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn unused_template_capabilities_report_only_the_unused_active_rule_with_whole_ranges() {
    for (rules, body, unused) in [
        (
            "vue/no-deprecated-dollar-scopedslots-api",
            "<div>{{ $slots }}</div>",
            RULE,
        ),
        (
            "vue/no-deprecated-dollar-scopedslots-api, vue/no-v-html",
            "<div>{{ $scopedSlots }}</div>",
            "vue/no-v-html",
        ),
    ] {
        let directive = vize_l0::cstr!("eslint-disable-next-line {rules}");
        let source = vize_l0::cstr!(
            "<script>export default {{}}</script>\r\n<template>\r\n<!-- {directive} -->\r\n{body}\r\n</template>\r\n"
        );
        let start = source.find(directive.as_str()).unwrap();
        let linter =
            reporting_linter().with_enabled_rules(Some(vec![RULE.into(), "vue/no-v-html".into()]));
        assert_eq!(
            whole(&linter.lint_sfc(&source, "Unused.vue")),
            json!({
                "filename":"Unused.vue", "error_count":1, "warning_count":0,
                "diagnostics":[{
                    "rule_name":"vue/comment-directive", "severity":"error",
                    "message":vize_l0::cstr!("Unused eslint-disable directive (no problems were reported from '{unused}')."),
                    "start":start, "end":start+directive.len(), "help":null, "fix":null, "labels":[]
                }]
            }),
            "{source}"
        );
    }
    let source = "<template>\n<!-- eslint-disable-next-line vue/no-deprecated-dollar-scopedslots-api -->\n<div>{{ $scopedSlots }}</div>\n</template>";
    assert_eq!(
        whole(
            &reporting_linter()
                .with_disabled_rules(vec![RULE.into()])
                .lint_sfc(source, "Off.vue")
        ),
        json!({
            "filename":"Off.vue", "error_count":0, "warning_count":0, "diagnostics":[]
        })
    );
}

#[test]
fn template_capability_reporting_preserves_the_complete_configured_warning_packet() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/lint-scoped-slots-template/cases.json"
    ))
    .unwrap();
    let case = &corpus["cases"][0];
    let mut diagnostics = case["nativeDiagnostics"].clone();
    diagnostics[0]["severity"] = json!("warning");
    let linter = reporting_linter()
        .with_rule_severity_overrides(vec![(RULE.into(), vize_patina::Severity::Warning)]);
    assert_eq!(
        whole(&linter.lint_sfc(
            case["source"].as_str().unwrap(),
            case["filename"].as_str().unwrap()
        )),
        json!({
            "filename":case["filename"], "error_count":0, "warning_count":1, "diagnostics":diagnostics
        })
    );
}
