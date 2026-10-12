//! Source-authored whole packets for the existing setup-export rule's presets.

use serde_json::{Value, json};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Severity};

const RULE: &str = "script/no-export-in-script-setup";

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

#[test]
fn explicit_rule_preserves_all_eighteen_runtime_type_and_module_packets() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/setup-export-default-8090/cases.json"
    ))
    .unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 18);
    assert_eq!(
        cases
            .iter()
            .map(|case| case["diagnostics"].as_array().unwrap().len())
            .sum::<usize>(),
        8
    );
    for case in cases {
        let result = Linter::with_preset(LintPreset::Incremental)
            .with_additional_rules(vec![RULE.into()])
            .with_rule_severity_overrides(vec![(RULE.into(), Severity::Error)])
            .with_help_level(HelpLevel::Full)
            .lint_sfc(
                case["source"].as_str().unwrap(),
                case["filename"].as_str().unwrap(),
            );
        assert_eq!(whole(&result), case["apiPacket"], "{}", case["id"]);
    }
}

fn route_linter(name: &str) -> Linter {
    match name {
        "default" => Linter::new(),
        "recommended" => Linter::with_preset(LintPreset::HappyPath),
        "essential" | "config-incremental-cli-essential" => {
            Linter::with_preset(LintPreset::Essential)
        }
        "ecosystem" => Linter::with_preset(LintPreset::Ecosystem),
        "opinionated" | "all" => Linter::with_preset(LintPreset::Opinionated),
        "nuxt" => Linter::with_preset(LintPreset::Nuxt),
        "incremental" | "config-essential-cli-incremental" => {
            Linter::with_preset(LintPreset::Incremental)
        }
        "explicit-off" => Linter::new().with_disabled_rules(vec![RULE.into()]),
        "config-off-cli-essential" => {
            Linter::with_preset(LintPreset::Essential).with_disabled_rules(vec![RULE.into()])
        }
        "n8n-projection" => {
            let projection: Value = serde_json::from_str(include_str!(
                "../../../tests/_fixtures/n8n-cli-adoption.json"
            ))
            .unwrap();
            let rules = projection["linter"]["rules"].as_object().unwrap();
            assert_eq!(rules.len(), 51);
            assert!(!rules.contains_key(RULE));
            Linter::with_preset(LintPreset::Incremental)
                .with_additional_rules(rules.keys().map(|name| name.as_str().into()).collect())
                .with_rule_severity_overrides(
                    rules
                        .keys()
                        .map(|name| (name.as_str().into(), Severity::Error))
                        .collect(),
                )
        }
        unknown => panic!("unknown authored route: {unknown}"),
    }
}

#[test]
fn every_preset_off_and_cli_selection_retains_complete_packets_and_rule_order() {
    let controls: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/setup-export-default-8090/default-route-controls.json"
    ))
    .unwrap();
    let cases = controls["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    let mut packets = 0;
    for case in cases {
        let routes = case["routes"].as_array().unwrap();
        assert_eq!(routes.len(), 13);
        for route in routes {
            let name = route["id"].as_str().unwrap();
            let result = route_linter(name)
                .with_help_level(HelpLevel::Full)
                .lint_sfc(
                    case["source"].as_str().unwrap(),
                    case["filename"].as_str().unwrap(),
                );
            assert_eq!(
                whole(&result),
                route["apiAfterPacket"],
                "{} / {name}",
                case["id"]
            );
            packets += 1;
        }
    }
    assert_eq!(packets, 52);
}
