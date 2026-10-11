use serde_json::{Value, json};
use vize_patina::rules::vue::{SfcElementOrderGroup, SfcElementOrderOptions};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Severity};

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
    let successors: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-suppression-8090/default-route-successors.json"
    ))
    .unwrap();
    let successor_cases = successors["cases"].as_array().unwrap();
    assert_eq!(successor_cases.len(), cases.len());
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
    for (case, successor) in cases.iter().zip(successor_cases) {
        assert_eq!(case["id"], successor["id"]);
        assert_eq!(case["sourceSha256"], successor["sourceSha256"]);
        let source = case["source"].as_str().unwrap();
        let filename = case["filename"].as_str().unwrap();
        let diagnostics = successor["defaultDiagnostics"].as_array().unwrap();
        let essential = successor["essentialDiagnostics"].as_array().unwrap();
        let off = successor["offDiagnostics"].as_array().unwrap();
        let incremental = successor["incrementalDiagnostics"].as_array().unwrap();
        let projected_diagnostics = successor["projectionDiagnostics"].as_array().unwrap();
        let projected = projection["linter"]["rules"]
            .as_object()
            .unwrap()
            .keys()
            .map(|name| name.as_str().into())
            .collect();
        let projected_severities = projection["linter"]["rules"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, value)| {
                assert_eq!(value.as_str(), Some("error"));
                (name.as_str().into(), Severity::Error)
            })
            .collect();
        let projected_order = SfcElementOrderOptions {
            order: projection["linter"]["ruleOptions"]["vue/sfc-element-order"]["order"]
                .as_array()
                .unwrap()
                .iter()
                .map(|selector| SfcElementOrderGroup::new(vec![selector.as_str().unwrap().into()]))
                .collect(),
        };
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
                essential.as_slice(),
            ),
            (
                "ecosystem",
                Linter::with_preset(LintPreset::Ecosystem),
                diagnostics.as_slice(),
            ),
            (
                "incremental",
                Linter::with_preset(LintPreset::Incremental),
                incremental.as_slice(),
            ),
            (
                "explicit-off",
                Linter::new().with_disabled_rules(vec![rule.into()]),
                off.as_slice(),
            ),
            (
                "n8n-projection",
                Linter::with_preset(LintPreset::Incremental)
                    .with_enabled_rules(Some(projected))
                    .with_rule_severity_overrides(projected_severities)
                    .with_sfc_element_order_options(projected_order),
                projected_diagnostics.as_slice(),
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
