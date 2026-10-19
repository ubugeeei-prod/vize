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
            "labels": d.labels.iter().map(|l| json!({
                "message": l.message, "start": l.start, "end": l.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

fn expected(filename: &str, diagnostics: &Value) -> Value {
    let findings = diagnostics.as_array().unwrap();
    json!({
        "filename": filename,
        "error_count": findings.iter().filter(|d| d["severity"] == "error").count(),
        "warning_count": findings.iter().filter(|d| d["severity"] == "warning").count(),
        "diagnostics": diagnostics
    })
}

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/public-vue-imports-8090/cases.json"
    ))
    .unwrap()
}

fn query(linter: &Linter, case: &Value, source: &str) -> LintResult {
    let filename = case["filename"].as_str().unwrap();
    if case["entry"] == "script" {
        linter.lint_script(source, filename)
    } else {
        linter.lint_sfc(source, filename)
    }
}

#[test]
fn public_import_identity_and_safe_fixes_preserve_complete_legacy_packets() {
    let corpus = corpus();
    let rule = corpus["rule"].as_str().unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 29);
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![rule.into()]))
        .with_help_level(HelpLevel::Full);
    for case in cases {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let result = query(&linter, case, source);
        assert_eq!(
            whole(&result),
            expected(filename, &case["diagnostics"]),
            "{}",
            case["id"]
        );
        let fixes = result
            .diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.fix.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(
            fixes.len(),
            case["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|d| !d["fix"].is_null())
                .count(),
            "{}",
            case["id"]
        );
        for fix in fixes {
            // Each offered fix is applied to the complete original carrier.
            assert_eq!(
                fix.apply(source).as_str(),
                case["fixedSource"].as_str().unwrap()
            );
        }
        let fixed = case["fixedSource"].as_str().unwrap();
        assert_eq!(
            whole(&query(&linter, case, fixed)),
            expected(filename, &case["fixedDiagnostics"]),
            "{} / fixed",
            case["id"]
        );
    }
}

#[test]
fn minimal_import_routes_retain_full_preset_config_and_order_diagnostics() {
    let corpus = corpus();
    let controls: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/public-vue-imports-8090/default-route-controls.json"
    ))
    .unwrap();
    // Later preset changes name the route whose vector a route now shares.
    let successor: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/public-vue-imports-8090/default-route-controls-successor.json"
    ))
    .unwrap();
    let projection: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/n8n-cli-adoption.json"
    ))
    .unwrap();
    let rule = corpus["rule"].as_str().unwrap();
    let projected = projection["linter"]["rules"].as_object().unwrap();
    assert_eq!(projected.len(), 51);
    assert!(!projected.contains_key(rule));
    let controls = controls["cases"].as_array().unwrap();
    assert_eq!(controls.len(), 4);
    for control in controls {
        let case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == control["id"])
            .unwrap();
        let order = SfcElementOrderOptions {
            order: projection["linter"]["ruleOptions"]["vue/sfc-element-order"]["order"]
                .as_array()
                .unwrap()
                .iter()
                .map(|selector| SfcElementOrderGroup::new(vec![selector.as_str().unwrap().into()]))
                .collect(),
        };
        let routes = [
            ("default", Linter::new()),
            ("recommended", Linter::with_preset(LintPreset::HappyPath)),
            ("essential", Linter::with_preset(LintPreset::Essential)),
            ("ecosystem", Linter::with_preset(LintPreset::Ecosystem)),
            ("incremental", Linter::with_preset(LintPreset::Incremental)),
            (
                "explicit-off",
                Linter::new().with_disabled_rules(vec![rule.into()]),
            ),
            (
                "n8n-projection",
                Linter::with_preset(LintPreset::Incremental)
                    .with_enabled_rules(Some(
                        projected.keys().map(|name| name.as_str().into()).collect(),
                    ))
                    .with_rule_severity_overrides(
                        projected
                            .keys()
                            .map(|name| (name.as_str().into(), Severity::Error))
                            .collect(),
                    )
                    .with_sfc_element_order_options(order),
            ),
        ];
        assert_eq!(control["routes"].as_array().unwrap().len(), routes.len());
        for (name, linter) in routes {
            let authored = successor["routes"][name].as_str().unwrap_or(name);
            let route = control["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["id"] == authored)
                .unwrap();
            assert_eq!(
                whole(&query(
                    &linter.with_help_level(HelpLevel::Full),
                    case,
                    case["source"].as_str().unwrap()
                )),
                expected(case["filename"].as_str().unwrap(), &route["diagnostics"]),
                "{} / {name}",
                case["id"]
            );
        }
    }
}
