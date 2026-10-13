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
fn bare_instance_spellings_are_valid_with_explicit_prefix_policy_successors() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/bare-reserved-8090/cases.json"
    ))
    .unwrap();
    let successors: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-prefix-8090/bare-parent-successors.json"
    ))
    .unwrap();
    let successor_cases = successors["cases"].as_array().unwrap();
    let route_successors: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-prefix-8090/bare-parent-route-successors.json"
    ))
    .unwrap();
    let route_cases = route_successors["cases"].as_array().unwrap();
    let api_order: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/reserved-prefix-8090/api-order-successors.json"
    ))
    .unwrap();
    let api_cases = api_order["bareCases"].as_array().unwrap();
    assert_eq!(api_cases.len(), 16);
    assert_eq!(route_cases.len(), 16);
    assert_eq!(
        route_cases
            .iter()
            .map(|case| case["combinedApiDiagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        11
    );
    let projection: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/n8n-cli-adoption.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    let rule = corpus["rule"].as_str().unwrap();
    let projected_rules = projection["linter"]["rules"].as_object().unwrap();
    assert_eq!(cases.len(), 16);
    assert_eq!(successor_cases.len(), 16);
    assert_eq!(corpus["removedBareNames"].as_array().unwrap().len(), 14);
    assert_eq!(projected_rules.len(), 51);
    assert!(!projected_rules.contains_key(rule));
    assert_eq!(
        cases
            .iter()
            .map(|case| case["beforeDiagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        137
    );
    assert_eq!(
        cases
            .iter()
            .map(|case| case["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        17
    );
    assert_eq!(
        successor_cases
            .iter()
            .map(|case| case["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        9
    );

    for case in cases {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let successor = successor_cases
            .iter()
            .find(|successor| successor["id"] == case["id"])
            .unwrap();
        assert_eq!(successor["filename"], case["filename"]);
        assert_eq!(successor["sourceSha256"], case["sourceSha256"]);
        let route_successor = route_cases
            .iter()
            .find(|entry| entry["id"] == case["id"])
            .unwrap();
        assert_eq!(route_successor["sourceSha256"], case["sourceSha256"]);
        assert_eq!(
            route_successor["targetDiagnostics"],
            successor["diagnostics"]
        );
        let api_case = api_cases
            .iter()
            .find(|entry| entry["id"] == case["id"])
            .unwrap();
        assert_eq!(api_case["filename"], case["filename"]);
        assert_eq!(api_case["sourceSha256"], case["sourceSha256"]);
        let diagnostics = api_case["diagnostics"].as_array().unwrap();
        let off_diagnostics = route_successor["diagnostics"].as_array().unwrap();
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
                json!({
                    "filename": filename,
                    "error_count": findings.iter().filter(|d| d["severity"] == "error").count(),
                    "warning_count": findings.iter().filter(|d| d["severity"] == "warning").count(),
                    "diagnostics": findings
                }),
                "{} / {route}",
                case["id"]
            );
        }
    }

    let original: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/bare-reserved-8090/original-wave-ui-accordion.json"
    ))
    .unwrap();
    let filename = original["filename"].as_str().unwrap();
    let source = include_str!(
        "../../../tests/_fixtures/differential/lint/bare-reserved-8090/original-wave-ui-accordion.vue.fixture"
    );
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![rule.into()]))
        .with_help_level(HelpLevel::Full)
        .lint_sfc(source, filename);
    assert_eq!(
        whole(&result),
        json!({
            "filename": filename, "error_count": 0, "warning_count": 0,
            "diagnostics": original["diagnostics"]
        }),
        "pinned original WaveUI file"
    );
}
