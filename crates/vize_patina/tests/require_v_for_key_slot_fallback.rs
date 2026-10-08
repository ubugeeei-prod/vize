use serde_json::{Value, json};
use vize_patina::rules::vue::RequireVForKey;
use vize_patina::{HelpLevel, LintDiagnostic, LintResult, Linter, RuleRegistry};

const CASES: &str = include_str!("fixtures/slot-fallback-key/cases.json");

fn diagnostic_packet(diagnostic: &LintDiagnostic) -> Value {
    let LintDiagnostic {
        rule_name,
        severity,
        message,
        start,
        end,
        help,
        labels,
        fix,
    } = diagnostic;
    json!({
        "rule_name": rule_name,
        "severity": severity,
        "message": message,
        "start": start,
        "end": end,
        "help": help,
        "labels": labels.iter().map(|label| json!({
            "message": label.message,
            "start": label.start,
            "end": label.end,
        })).collect::<Vec<_>>(),
        "fix": fix,
    })
}

fn whole_packet(result: &LintResult) -> Value {
    let LintResult {
        filename,
        diagnostics,
        error_count,
        warning_count,
    } = result;
    json!({
        "filename": filename,
        "diagnostics": diagnostics.iter().map(diagnostic_packet).collect::<Vec<_>>(),
        "error_count": error_count,
        "warning_count": warning_count,
    })
}

#[test]
fn slot_fallback_key_owned_source_api_packets() {
    let fixture: Value = serde_json::from_str(CASES).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(RequireVForKey));
    let linter = Linter::with_registry(registry).with_help_level(HelpLevel::None);
    let packets = cases
        .iter()
        .map(|case| {
            let result = linter.lint_sfc(
                case["source"].as_str().unwrap(),
                case["filename"].as_str().unwrap(),
            );
            json!({"caseId": case["id"], "packet": whole_packet(&result)})
        })
        .collect::<Vec<_>>();

    // Retain every public API field before assertions, including failing cases.
    if let Ok(path) = std::env::var("VIZE_SLOT_KEY_CAPTURE_PATH") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }

    for (case, recorded) in cases.iter().zip(&packets) {
        let expected = case["expectedNativeDiagnostics"].as_u64().unwrap();
        assert_eq!(
            recorded["packet"]["error_count"], expected,
            "{}",
            case["id"]
        );
        assert_eq!(recorded["packet"]["warning_count"], 0, "{}", case["id"]);
        assert_eq!(
            recorded["packet"], case["expectedNativePacket"],
            "{}",
            case["id"]
        );
    }
}
