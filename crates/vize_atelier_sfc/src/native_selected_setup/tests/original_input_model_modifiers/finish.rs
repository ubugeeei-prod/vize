//! Retain all seven complete native arrays before the strict current verdict.
use serde_json::{Value, json};

pub(super) fn write(mut capture: Value, complete: bool) -> Result<(), String> {
    let mut failures = capture["failures"]
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![json!("missing original failure array")]);
    let mut compared = false;
    let mut matched = false;
    match serde_json::from_str::<Value>(include_str!(
        "../../../../tests/fixtures/native_original_input_model_modifiers_all_targets.json"
    )) {
        Ok(expected) => {
            if expected["schema"]
                != "vize.native-sfc.original-input-model-modifiers-complete-goldens"
                || expected["inputHash"] != capture["inputHash"]
                || expected["inputPack"] != capture["inputPack"]
                || expected["provenance"]["independentlyReviewedBeforeFreeze"] != true
            {
                failures.push(json!(
                    "whole current input/reviewed golden authority mismatch"
                ));
            } else {
                compared = true;
                matched = complete;
                for field in ["fixtures", "custody", "attempts", "refusals"] {
                    if capture[field] != expected["native"][field] {
                        failures.push(json!(format!("whole native {field} mismatch")));
                        matched = false;
                    }
                }
                if capture["currentOutside"] != expected["currentOutside"] {
                    failures.push(json!("whole three compatibility outside arrays mismatch"));
                    matched = false;
                }
            }
        }
        Err(error) => failures.push(json!(format!("reviewed golden parse: {error}"))),
    }
    let qualified = complete && compared && matched && failures.is_empty();
    capture["failures"] = json!(failures);
    capture["nativeGoldenComparisonPerformed"] = json!(compared);
    capture["nativeGoldensMatched"] = json!(matched);
    capture["qualification"] = json!(if qualified {
        "complete-current-native-oracles-matched"
    } else {
        "failed-current-native-observation"
    });
    // Actual official-primary/runtime equality remains a separate Node verdict.
    capture["wholePacketQualification"] = json!(false);
    if let Ok(path) = std::env::var("VIZE_NATIVE_MODEL_MODIFIERS_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&capture).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    println!(
        "{}",
        serde_json::to_string(&capture).map_err(|e| e.to_string())?
    );
    if qualified {
        Ok(())
    } else {
        Err(format!(
            "current whole modifier comparison: {}",
            capture["failures"]
        ))
    }
}
