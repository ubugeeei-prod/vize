use crate::{HelpLevel, LintPreset, LintResult, Linter};
use serde_json::{Value, json};

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
fn typed_prop_defaults_reach_default_sfc_dispatch_with_complete_controls() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/differential/lint/typed-prop-default-prefilter-8564/cases.json"
    ))
    .unwrap();
    let rule = corpus["rule"].as_str().unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    assert_eq!(
        cases
            .iter()
            .filter(|case| !case["diagnostics"].as_array().unwrap().is_empty())
            .map(|case| case["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "original-vaul",
            "with-defaults",
            "destructure-alias",
            "unicode-crlf"
        ],
        "fixture must retain the four authored positive cases"
    );
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        for preset in [
            LintPreset::HappyPath,
            LintPreset::Essential,
            LintPreset::Ecosystem,
        ] {
            let result = Linter::with_preset(preset)
                .with_help_level(HelpLevel::None)
                .lint_sfc(source, filename);
            assert_eq!(
                whole(&result),
                json!({
                    "filename": filename,
                    "error_count": case["diagnostics"].as_array().unwrap().len(),
                    "warning_count": 0,
                    "diagnostics": case["diagnostics"]
                }),
                "{} / {}",
                case["id"],
                preset.as_str()
            );
        }
        for linter in [
            Linter::new().with_disabled_rules(vec![rule.into()]),
            Linter::with_preset(LintPreset::Incremental),
        ] {
            assert_eq!(
                whole(&linter.lint_sfc(source, filename)),
                json!({
                    "filename": filename, "error_count": 0, "warning_count": 0,
                    "diagnostics": []
                }),
                "{}",
                case["id"]
            );
        }
    }
}
