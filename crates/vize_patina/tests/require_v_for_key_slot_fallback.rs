use serde_json::{Value, json};
use vize_patina::rules::vue::RequireVForKey;
use vize_patina::{HelpLevel, Linter, RuleRegistry};

use crate::v_for_key_capture;
use v_for_key_capture::{whole_packet, write_capture};

// The original 21 before packets remain intact in CASES.
const CASES: &str = include_str!("fixtures/slot-fallback-key/cases.json");
const AFTER: &str = include_str!("fixtures/v-for-key-ownership/slot-fallback-after.json");

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
    write_capture("v-for-slot-current-packets.json", &packets);
    if let Ok(path) = std::env::var("VIZE_SLOT_KEY_CAPTURE_PATH") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }

    let after: Value = serde_json::from_str(AFTER).unwrap();
    let after = after.as_array().unwrap();
    assert_eq!(
        after.len(),
        cases.len(),
        "all original cases retain a current oracle"
    );
    for ((case, recorded), current) in cases.iter().zip(&packets).zip(after) {
        assert_eq!(case["id"], current["caseId"]);
        if case["id"] != "empty-slot-policy-limit" {
            assert_eq!(current["packet"], case["expectedNativePacket"]);
        }
        let expected = current["packet"]["error_count"].as_u64().unwrap();
        assert_eq!(
            recorded["packet"]["error_count"], expected,
            "{}",
            case["id"]
        );
        assert_eq!(recorded["packet"]["warning_count"], 0, "{}", case["id"]);
        assert_eq!(recorded["packet"], current["packet"], "{}", case["id"]);
    }
}
