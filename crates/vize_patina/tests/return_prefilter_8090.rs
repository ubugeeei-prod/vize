use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter};

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
fn default_sfc_return_checks_reach_methods_and_arrow_trivia() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/return-prefilter-8090/cases.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    let rules = corpus["rules"].as_array().unwrap();
    let disabled = rules
        .iter()
        .map(|rule| rule.as_str().unwrap().into())
        .collect::<Vec<_>>();
    for case in cases {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let diagnostics = case["diagnostics"].as_array().unwrap();
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
                    "error_count": diagnostics.iter().filter(|d| d["severity"] == "error").count(),
                    "warning_count": diagnostics.iter().filter(|d| d["severity"] == "warning").count(),
                    "diagnostics": diagnostics
                }),
                "{} / {}",
                case["id"],
                preset.as_str()
            );
        }
        for linter in [
            Linter::new().with_disabled_rules(disabled.clone()),
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
