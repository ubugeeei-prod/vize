//! Whole public #7989 input and independently authored CSS declaration controls.

use serde_json::{Value, json};
use std::{fs, path::Path, time::SystemTime};
use vize_l0::cstr;
use vize_patina::{LintPreset, LintResult, Linter, Severity};

const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/linter/css-value-tokens-7989/cases.json");
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/css-value-tokens-7989/MyNotes.vue.txt"
);

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

#[test]
fn entire_original_and_css_controls_preserve_complete_service_vectors() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 17);
    assert_eq!(cases[0]["source"], ORIGINAL);
    let rules = [
        "script/prefer-ref-over-reactive",
        "script/prefer-use-id",
        "css/no-v-bind-performance",
        "css/no-important",
    ];
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(rules.iter().map(|rule| (*rule).into()).collect()))
        .with_rule_severity_overrides(
            rules
                .iter()
                .map(|rule| ((*rule).into(), Severity::Warning))
                .collect(),
        );
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".into());
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let capture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile)
        .join("css-value-tokens-7989-api")
        .join(cstr!("{}-{time}", std::process::id()).as_str());
    fs::create_dir_all(&capture).unwrap();
    let mut observed = Vec::new();
    let mut failures = Vec::new();
    for case in cases {
        let source = case["source"].as_str().unwrap();
        assert_eq!(Some(source.len() as u64), case["bytes"].as_u64());
        let actual = complete(&linter.lint_sfc(source, case["filename"].as_str().unwrap()));
        observed.push(json!({"case": case["id"], "source": source,
            "expected": case["service"], "actual": actual}));
        fs::write(
            capture.join("whole-observations.json"),
            serde_json::to_vec_pretty(&observed).unwrap(),
        )
        .unwrap();
        if actual != case["service"] {
            failures
                .push(json!({"case": case["id"], "actual": actual, "expected": case["service"]}));
        }
    }
    assert_eq!(observed.len(), 17);
    assert_eq!(
        failures,
        Vec::<Value>::new(),
        "whole service vectors; raw at {capture:?}"
    );
}
