//! Whole authored packets for the two default warning policies.

use crate::{LintPreset, LintResult, Linter, rule::RuleRegistry};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn cases() -> Value {
    serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/lint-preset-default-warnings/cases.json"
    ))
    .unwrap()
}

fn whole(result: &LintResult) -> Value {
    json!({
        "filename": result.filename,
        "error_count": result.error_count, "warning_count": result.warning_count,
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
fn all_six_presets_emit_the_complete_authored_warning_packets() {
    let corpus = cases();
    for case in corpus["cases"].as_array().unwrap() {
        let filename = case["filename"].as_str().unwrap();
        for preset in LintPreset::ALL {
            let enabled = case["enabledPresets"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name == preset.as_str());
            let baseline = if case["baselinePresets"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name == preset.as_str())
            {
                case["baselineDiagnostics"].as_array().unwrap().clone()
            } else {
                vec![]
            };
            // SFC diagnostics precede the script warnings appended by lint_sfc.
            let mut diagnostics = baseline.clone();
            if enabled {
                diagnostics.push(case["diagnostic"].clone());
            }
            let expected = json!({
                "filename": filename, "error_count": 0,
                "warning_count": diagnostics.len(), "diagnostics": diagnostics
            });
            let linter = Linter::with_preset(preset);
            assert_eq!(
                whole(&linter.lint_sfc(case["source"].as_str().unwrap(), filename)),
                expected,
                "{} / {}",
                case["id"],
                preset.as_str()
            );
            assert_eq!(
                whole(&linter.lint_sfc(case["repair"].as_str().unwrap(), filename)),
                json!({"filename": filename, "error_count": 0, "warning_count": 0, "diagnostics": []}),
                "{} / {} / repaired",
                case["id"],
                preset.as_str()
            );
            let disabled = linter.with_disabled_rules(vec![case["rule"].as_str().unwrap().into()]);
            assert_eq!(
                whole(&disabled.lint_sfc(case["source"].as_str().unwrap(), filename)),
                json!({"filename": filename, "error_count": 0, "warning_count": baseline.len(), "diagnostics": baseline}),
                "{} / {} / explicit off",
                case["id"],
                preset.as_str()
            );
        }
        for alias in [
            "default",
            "recommended",
            "general-recommended",
            "happy-path",
        ] {
            assert_eq!(
                whole(
                    &Linter::with_preset(LintPreset::parse(alias).unwrap())
                        .lint_sfc(case["source"].as_str().unwrap(), filename)
                ),
                whole(&Linter::new().lint_sfc(case["source"].as_str().unwrap(), filename)),
                "{} / {alias}",
                case["id"]
            );
        }
    }
}

#[test]
fn preset_registration_and_script_metadata_keep_each_rule_once() {
    for preset in LintPreset::ALL {
        let mut registry = RuleRegistry::with_preset(preset);
        let expected = !matches!(preset, LintPreset::Essential | LintPreset::Incremental);
        assert_eq!(registry.has_rule("vue/no-use-v-else-with-v-for"), expected);
        for _ in 0..2 {
            registry.register_opt_in_rules();
            let names = registry.rule_names();
            assert_eq!(
                names.iter().copied().collect::<BTreeSet<_>>().len(),
                names.len()
            );
            assert_eq!(
                names
                    .iter()
                    .filter(|n| **n == "vue/no-use-v-else-with-v-for")
                    .count(),
                1
            );
        }
        assert_eq!(
            super::builtin_script_rule_names(preset)
                .iter()
                .filter(|n| **n == "script/no-deep-destructure-in-props")
                .count(),
            usize::from(matches!(preset, LintPreset::Opinionated | LintPreset::Nuxt))
        );
    }
    assert_eq!(
        RuleRegistry::with_preset(LintPreset::Incremental)
            .rules()
            .len(),
        0
    );
    assert!(super::builtin_script_rule_names(LintPreset::Incremental).is_empty());
}
