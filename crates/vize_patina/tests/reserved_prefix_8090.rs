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
fn reserved_names_obey_exact_membership_and_group_policy_with_complete_packets() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-prefix-8090/cases.json"
    ))
    .unwrap();
    let originals: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-prefix-8090/originals.json"
    ))
    .unwrap();
    let companions: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-prefix-8090/default-route-companions.json"
    ))
    .unwrap();
    let api_order: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-prefix-8090/api-order-successors.json"
    ))
    .unwrap();
    let api_cases = api_order["coreCases"].as_array().unwrap();
    assert_eq!(api_cases.len(), 20);
    assert_eq!(api_order["summary"]["coreDiagnosticCount"], 211);
    let projection: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/n8n-cli-adoption.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    let rule = corpus["rule"].as_str().unwrap();
    let projected_rules = projection["linter"]["rules"].as_object().unwrap();
    assert_eq!(cases.len(), 20);
    let companion_cases = companions["cases"].as_array().unwrap();
    assert_eq!(companion_cases.len(), 20);
    assert_eq!(
        companion_cases
            .iter()
            .map(|case| case["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        59
    );
    assert_eq!(corpus["reservedDollarNames"].as_array().unwrap().len(), 24);
    assert_eq!(projected_rules.len(), 51);
    assert!(!projected_rules.contains_key(rule));
    assert_eq!(
        cases
            .iter()
            .map(|case| case["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        152
    );
    assert_eq!(
        cases
            .iter()
            .filter(|case| case["providerParity"] == "unsupported")
            .count(),
        2
    );
    for case in cases {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let companion = companion_cases
            .iter()
            .find(|entry| entry["id"] == case["id"])
            .unwrap();
        assert_eq!(companion["filename"], case["filename"]);
        assert_eq!(companion["sourceSha256"], case["sourceSha256"]);
        let api_case = api_cases
            .iter()
            .find(|entry| entry["id"] == case["id"])
            .unwrap();
        assert_eq!(api_case["filename"], case["filename"]);
        assert_eq!(api_case["sourceSha256"], case["sourceSha256"]);
        let diagnostics = api_case["diagnostics"].as_array().unwrap();
        let off_diagnostics = companion["diagnostics"].as_array().unwrap();
        let projected = projected_rules
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
                off_diagnostics.as_slice(),
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

    let original_cases = originals["cases"].as_array().unwrap();
    assert_eq!(original_cases.len(), 21);
    assert_eq!(
        original_cases
            .iter()
            .map(|case| case["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        1
    );
    for case in original_cases {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let diagnostics = case["diagnostics"].as_array().unwrap();
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![rule.into()]))
            .with_help_level(HelpLevel::Full)
            .lint_sfc(source, filename);
        assert_eq!(
            whole(&result),
            expected(filename, diagnostics),
            "{}",
            case["id"]
        );
    }
}
