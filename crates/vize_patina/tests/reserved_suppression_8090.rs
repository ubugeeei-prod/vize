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
            "labels": d.labels.iter().map(|label| json!({
                "message": label.message, "start": label.start, "end": label.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

fn expected(filename: &str, findings: &[Value]) -> Value {
    json!({
        "filename": filename,
        "error_count": findings.iter().filter(|d| d["severity"] == "error").count(),
        "warning_count": findings.iter().filter(|d| d["severity"] == "warning").count(),
        "diagnostics": findings
    })
}

#[test]
fn registered_aliases_preserve_complete_rule_and_comment_scope_packets() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-suppression-8090/cases.json"
    ))
    .unwrap();
    let projection: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/n8n-cli-adoption.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    let rule = corpus["rule"].as_str().unwrap();
    assert_eq!(cases.len(), 24);
    assert_eq!(
        cases
            .iter()
            .map(|c| c["beforeDiagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        22
    );
    assert_eq!(
        cases
            .iter()
            .map(|c| c["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        12
    );
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let diagnostics = case["diagnostics"].as_array().unwrap();
        let projected = projection["linter"]["rules"]
            .as_object()
            .unwrap()
            .keys()
            .map(|name| name.as_str().into())
            .collect();
        let routes = [
            ("default", Linter::new(), diagnostics.as_slice()),
            (
                "recommended",
                Linter::with_preset(LintPreset::HappyPath),
                diagnostics.as_slice(),
            ),
            (
                "essential",
                Linter::with_preset(LintPreset::Essential),
                diagnostics.as_slice(),
            ),
            (
                "ecosystem",
                Linter::with_preset(LintPreset::Ecosystem),
                diagnostics.as_slice(),
            ),
            (
                "incremental",
                Linter::with_preset(LintPreset::Incremental),
                &[],
            ),
            (
                "explicit-off",
                Linter::new().with_disabled_rules(vec![rule.into()]),
                &[],
            ),
            (
                "n8n-projection",
                Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(projected)),
                &[],
            ),
        ];
        for (route, linter, findings) in routes {
            let result = linter
                .with_help_level(HelpLevel::Full)
                .lint_sfc(source, filename);
            assert_eq!(
                whole(&result),
                expected(filename, findings),
                "{} / {route}",
                case["id"]
            );
        }
    }
    let original: Value = serde_json::from_str(include_str!("../../../tests/_fixtures/differential/lint/reserved-suppression-8090/original-vue-select-successor.json")).unwrap();
    let source = include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-prefix-8090/original-vue-select-src-components-select.vue.fixture"
    );
    let filename = original["filename"].as_str().unwrap();
    assert_eq!(original["beforeDiagnostics"].as_array().unwrap().len(), 1);
    assert_eq!(
        original["publicBeforeDiagnostics"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![rule.into()]))
        .with_help_level(HelpLevel::Full)
        .lint_sfc(source, filename);
    assert_eq!(
        whole(&result),
        expected(filename, original["diagnostics"].as_array().unwrap())
    );
}
