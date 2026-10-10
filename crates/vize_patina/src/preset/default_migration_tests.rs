//! Complete authored preset results for the second Vue 3 migration slice.

use crate::{HelpLevel, LintPreset, LintResult, Linter, RuleRegistry};
use serde_json::{Value, json};
use vize_l0::config::VueVersion;

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/lint-default-migrations/cases.json"
    ))
    .unwrap()
}

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

fn expected(case: &Value, findings: &Value) -> Value {
    json!({
        "filename": case["filename"],
        "error_count": findings.as_array().unwrap().iter().filter(|d| d["severity"] == "error").count(),
        "warning_count": findings.as_array().unwrap().iter().filter(|d| d["severity"] == "warning").count(),
        "diagnostics": findings
    })
}

#[test]
fn presets_preserve_complete_before_and_after_default_findings() {
    let data = corpus();
    let promoted: Vec<vize_l0::String> = data["promotedRules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().into())
        .collect();
    for case in data["cases"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        for preset in LintPreset::ALL {
            let (after, before) = match preset {
                LintPreset::Incremental => (json!([]), json!([])),
                LintPreset::Essential => (case["essentialDiagnostics"].clone(), json!([])),
                _ => (
                    case["diagnostics"].clone(),
                    case["baselineDiagnostics"].clone(),
                ),
            };
            let linter = Linter::with_preset(preset).with_help_level(HelpLevel::None);
            assert_eq!(
                whole(&linter.lint_sfc(source, filename)),
                expected(case, &after),
                "{} / {}",
                case["id"],
                preset.as_str()
            );
            assert_eq!(
                whole(
                    &linter
                        .with_disabled_rules(promoted.clone())
                        .lint_sfc(source, filename)
                ),
                expected(case, &before),
                "baseline {} / {}",
                case["id"],
                preset.as_str()
            );
            for version in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
                let result = Linter::with_preset(preset)
                    .with_help_level(HelpLevel::None)
                    .with_vue_version(Some(version))
                    .lint_sfc(source, filename);
                assert_eq!(
                    whole(&result),
                    expected(case, if version.is_legacy() { &before } else { &after }),
                    "{} / {} / {version:?}",
                    case["id"],
                    preset.as_str()
                );
            }
        }
    }
}

#[test]
fn n8n_explicit_selection_retains_whole_findings_without_default_leakage() {
    let projection: Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/n8n-cli-adoption.json"
    ))
    .unwrap();
    let selected: Vec<vize_l0::String> = projection["linter"]["rules"]
        .as_object()
        .unwrap()
        .keys()
        .map(|name| name.as_str().into())
        .collect();
    assert_eq!(selected.len(), 51);
    assert_eq!(projection["linter"]["preset"], "incremental");
    for case in corpus()["cases"].as_array().unwrap() {
        for preset in [
            LintPreset::Incremental,
            LintPreset::HappyPath,
            LintPreset::Ecosystem,
        ] {
            let result = Linter::with_preset(preset)
                .with_help_level(HelpLevel::None)
                .with_enabled_rules(Some(selected.clone()))
                .lint_sfc(
                    case["source"].as_str().unwrap(),
                    case["filename"].as_str().unwrap(),
                );
            assert_eq!(
                whole(&result),
                expected(case, &case["essentialDiagnostics"]),
                "{} / explicit51 / {}",
                case["id"],
                preset.as_str()
            );
        }
    }
}

#[test]
fn migration_memberships_deduplicate_and_keep_no_fix_contracts() {
    let data = corpus();
    for preset in LintPreset::ALL {
        let mut registry = RuleRegistry::with_preset(preset);
        for name in data["promotedRules"].as_array().unwrap() {
            assert_eq!(
                registry.has_rule(name.as_str().unwrap()),
                preset != LintPreset::Incremental
            );
        }
        registry.register_opt_in_rules();
        for name in data["promotedRules"].as_array().unwrap() {
            let rules: Vec<_> = registry
                .rules()
                .iter()
                .filter(|r| r.meta().name == name.as_str().unwrap())
                .collect();
            assert_eq!(rules.len(), 1);
            assert!(!rules[0].meta().fixable);
            assert_eq!(rules[0].meta().default_severity, crate::Severity::Error);
        }
    }
    let html = "<div v-scope=\"{}\"><button type=\"button\" @keyup.13=\"submit\">Submit</button></div><script src=\"https://unpkg.com/petite-vue\" init></script>";
    let names = data["promotedRules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().into())
        .collect();
    let result = Linter::new()
        .with_enabled_rules(Some(names))
        .lint_standalone_html(html, "index.html");
    assert_eq!(
        whole(&result),
        json!({"filename":"index.html","error_count":0,"warning_count":0,"diagnostics":[]})
    );
}
