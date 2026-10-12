//! Public CSS and independently enabled markerless-list fallback observations.
#![cfg(test)]

use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale};

const CORPUS: &str =
    include_str!("../../../tests/_fixtures/differential/css-engine-boundary-3295/cases.json");

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| json!({"message": label.message, "start": label.start, "end": label.end}))
                .collect();
            json!({"rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message, "start": diagnostic.start, "end": diagnostic.end,
                "help": diagnostic.help, "labels": labels, "fix": diagnostic.fix})
        })
        .collect();
    json!({"filename": result.filename, "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics})
}

fn exercise(rule: &str, expected_cases: usize, name: &str) {
    let corpus: Value = serde_json::from_str(CORPUS).expect("finite complete CSS boundary corpus");
    assert_eq!(corpus["issue"], 3295);
    assert_eq!(corpus["relatedIssue"], 4961);
    assert_eq!(corpus["linterCases"].as_array().unwrap().len(), 15);
    for (case, css) in corpus["originals"].as_array().unwrap().iter().zip([
        "a{opacity:abs(-50%)}",
        "a{color:hsl(0 abs(-50%) 0%)}",
        "a{text-size-adjust:calc(5)}",
    ]) {
        assert_eq!(case["source"], css, "whole unchanged original CSS");
    }
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![rule.into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".into());
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile)
        .join("css-engine-boundary-3295-patina");
    fs::create_dir_all(&directory).unwrap();
    let mut observations = Vec::new();
    let mut failures = Vec::new();
    let mut cases = 0;
    for case in corpus["linterCases"].as_array().unwrap() {
        if case["options"]["enabledRules"] != json!([rule]) {
            continue;
        }
        cases += 1;
        assert_eq!(case["options"]["preset"], "incremental");
        assert_eq!(case["options"]["locale"], "en");
        assert_eq!(case["options"]["helpLevel"], "full");
        let source = case["source"].as_str().unwrap();
        let filename = case["options"]["filename"].as_str().unwrap();
        for pass in 1..=3 {
            let result = linter.lint_sfc(source, filename);
            let actual = complete(&result);
            let mut observation = json!({"case": case["id"], "pass": pass,
                "source": source, "options": case["options"],
                "expected": case["expected"], "actual": actual});
            if actual != case["expected"] {
                failures.push(observation.clone());
            }
            fs::write(
                directory.join(name),
                serde_json::to_vec_pretty(
                    &json!({"completed": observations, "current": observation}),
                )
                .unwrap(),
            )
            .expect("retain whole initial result before inspecting an optional fix");
            if let Some(expected) = case["fixedExpected"].as_str() {
                if let Some(fix) = result
                    .diagnostics
                    .first()
                    .and_then(|finding| finding.fix.as_ref())
                {
                    let fixed = fix.apply(source);
                    let fixed_actual = complete(&linter.lint_sfc(&fixed, filename));
                    observation["fixedActual"] = json!({"source": fixed, "result": fixed_actual});
                    if fixed.as_str() != expected || fixed_actual != case["fixedResultExpected"] {
                        failures.push(observation.clone());
                    }
                } else {
                    observation["fixedActual"] = json!({"missingFix": true});
                    failures.push(observation.clone());
                }
            }
            observations.push(observation);
            fs::write(
                directory.join(name),
                serde_json::to_vec_pretty(&observations).unwrap(),
            )
            .expect("retain every whole actual result before reporting mismatch");
        }
    }
    assert_eq!(cases, expected_cases);
    assert_eq!(observations.len(), expected_cases * 3);
    assert_eq!(
        failures,
        Vec::<Value>::new(),
        "whole results at {directory:?}"
    );
}

#[test]
fn css_engine_defects_use_existing_invalid_css_fallback_and_keep_healthy_findings() {
    exercise("css/no-important", 8, "whole-css-results.json");
}

#[test]
fn markerless_list_engine_defects_use_existing_fallback_without_activating_css_rules() {
    exercise("a11y/no-redundant-roles", 7, "whole-role-results.json");
}
