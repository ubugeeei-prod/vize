//! Authored complete default-preset results; no runtime snapshot recapture.

use crate::{HelpLevel, LintPreset, LintResult, Linter};
use serde_json::{Value, json};

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/lint-default-correctness/cases.json"
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

#[test]
fn default_and_ecosystem_preserve_complete_authored_correctness_results() {
    let data = corpus();
    for case in data["cases"].as_array().unwrap() {
        if case["kind"] == "html" {
            continue;
        }
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let expected = json!({
            "filename": filename,
            "error_count": case["diagnostics"].as_array().unwrap().iter().filter(|d| d["severity"] == "error").count(),
            "warning_count": case["diagnostics"].as_array().unwrap().iter().filter(|d| d["severity"] == "warning").count(),
            "diagnostics": case["diagnostics"]
        });
        for preset in [LintPreset::HappyPath, LintPreset::Ecosystem] {
            let result = Linter::with_preset(preset)
                .with_help_level(HelpLevel::None)
                .lint_sfc(source, filename);
            assert_eq!(
                whole(&result),
                expected,
                "{} / {}",
                case["id"],
                preset.as_str()
            );
        }
    }
}

#[test]
fn promoted_rules_can_be_disabled_while_retaining_baseline_and_empty_incremental() {
    let data = corpus();
    let promoted: Vec<vize_l0::String> = data["promotedRules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().into())
        .collect();
    for case in &data["cases"].as_array().unwrap()[..6] {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        for (linter, expected_diagnostics) in [
            (
                Linter::new().with_disabled_rules(promoted.clone()),
                case["baselineDiagnostics"].clone(),
            ),
            (Linter::with_preset(LintPreset::Incremental), json!([])),
        ] {
            assert_eq!(
                whole(&linter.lint_sfc(source, filename)),
                json!({
                    "filename": filename, "error_count": 0,
                    "warning_count": expected_diagnostics.as_array().unwrap().len(),
                    "diagnostics": expected_diagnostics
                }),
                "{}",
                case["id"]
            );
        }
    }
    let petite = data["cases"].as_array().unwrap().last().unwrap();
    let result = Linter::new()
        .with_enabled_rules(Some(promoted))
        .lint_standalone_html(
            petite["source"].as_str().unwrap(),
            petite["filename"].as_str().unwrap(),
        );
    assert_eq!(
        whole(&result),
        json!({
            "filename": petite["filename"], "error_count": 0, "warning_count": 0, "diagnostics": []
        })
    );
}

#[test]
fn ecosystem_inherits_every_default_script_rule_and_presets_register_once() {
    let happy = super::builtin_script_rule_names(LintPreset::HappyPath);
    let eco = super::builtin_script_rule_names(LintPreset::Ecosystem);
    for name in happy {
        assert!(eco.contains(name), "ecosystem lost {name}");
    }
    let data = corpus();
    for preset in LintPreset::ALL {
        let mut registry = crate::rule::RuleRegistry::with_preset(preset);
        for name in data["promotedRules"].as_array().unwrap() {
            assert_eq!(
                registry.has_rule(name.as_str().unwrap()),
                preset != LintPreset::Incremental
            );
        }
        registry.register_opt_in_rules();
        for name in data["promotedRules"].as_array().unwrap() {
            assert_eq!(
                registry
                    .rule_names()
                    .iter()
                    .filter(|n| **n == name.as_str().unwrap())
                    .count(),
                1
            );
        }
    }
}
