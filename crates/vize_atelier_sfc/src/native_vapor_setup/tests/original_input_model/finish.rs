//! Preserve every complete current array before the strict comparison verdict.

use super::*;

pub(super) fn write(mut capture: Value, complete: bool) -> Result<(), String> {
    let mut failures = Vec::new();
    let mut compared = false;
    let comparison = (|| -> Result<(), String> {
        let golden: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/native_original_input_model_vapor_sfc_vue_3_6_rc9.json"
        ))
        .map_err(|error| cstr!("{error}"))?;
        require!(
            golden["schema"] == "vize.native-vapor.original-input-model-complete-goldens"
                && golden["inputPack"] == capture["inputPack"]
                && golden["inputHash"] == capture["inputHash"]
                && golden["target"] == capture["target"]
                && golden["provenance"]["independentlyReviewedBeforeFreeze"] == true,
            "whole reviewed original Vapor authority"
        );
        compared = true;
        for key in ["fixtures", "custody", "attempts", "refusals"] {
            if capture[key] != golden["native"][key] {
                failures.push(cstr!(
                    "whole current native Vapor {key} differs from reviewed golden"
                ));
            }
        }
        Ok(())
    })();
    if let Err(error) = comparison {
        failures.push(error);
    }
    let original = capture["failures"]
        .as_array_mut()
        .ok_or("original failure records")?;
    original.extend(
        failures
            .into_iter()
            .map(|message| serde_json::json!(message)),
    );
    let qualified = complete && compared && original.is_empty();
    capture["nativeGoldenComparisonPerformed"] = serde_json::json!(compared);
    capture["nativeGoldensMatched"] = serde_json::json!(qualified);
    capture["qualification"] = serde_json::json!(if qualified {
        "current-complete-native-goldens-matched"
    } else {
        "failed-current-native-golden-comparison"
    });
    // This producer owns the four native arrays. Actual primary/runtime whole
    // equality remains separately required by the source-bound Node consumer.
    capture["wholePacketQualification"] = serde_json::json!(false);
    if let Ok(path) = std::env::var("VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&capture).map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    require!(
        qualified,
        "whole native Vapor comparison failures: {}",
        capture["failures"]
    );
    Ok(())
}
