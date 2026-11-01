//! Whole independently reviewed native arrays, retained before verdict.

use serde_json::{Value, json};

pub(super) fn write(mut capture: Value, complete: bool) -> Result<(), String> {
    let mut failures = capture["failures"]
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![json!("missing actual producer failure array")]);
    let expected = serde_json::from_str::<Value>(include_str!(
        "../fixtures/native_original_input_model_trim_all_targets.json"
    ));
    let mut compared = false;
    let mut matched = false;
    match expected {
        Ok(expected) => {
            if expected["schema"] != "vize.native-sfc.original-input-model-trim-complete-goldens"
                || expected["inputHash"] != capture["inputHash"]
                || expected["inputPack"] != capture["inputPack"]
                || expected["provenance"]["independentlyReviewedBeforeFreeze"] != true
            {
                failures.push(json!(
                    "whole current input/independent golden authority mismatch"
                ));
            } else {
                let current = serde_json::from_str::<Value>(include_str!(
                    "../fixtures/native_original_input_model_current_outside.json"
                ))
                .map_err(|error| format!("{error}"));
                match current {
                    Ok(current)
                        if current["schema"] == "vize.native-sfc.input-model-current-outside"
                            && current["inputPack"]["history"]["trimInputsSha256"]
                                == capture["inputHash"]
                            && current["provenance"]["independentlyReviewedBeforeFreeze"]
                                == true
                            && current["arrays"]["trim"]
                                .as_array()
                                .is_some_and(|rows| rows.len() == 36) =>
                    {
                        compared = true;
                        matched = complete;
                        for field in ["fixtures", "custody", "attempts", "refusals"] {
                            let expected_field = if field == "refusals" {
                                &current["arrays"]["trim"]
                            } else {
                                &expected["native"][field]
                            };
                            if &capture[field] != expected_field {
                                failures.push(json!(format!(
                                    "whole native {field} differs from reviewed origin"
                                )));
                                matched = false;
                            }
                        }
                    }
                    _ => failures.push(json!("whole reviewed current outside authority mismatch")),
                }
            }
        }
        Err(error) => failures.push(json!(format!("reviewed golden parse: {error}"))),
    }
    let qualified = complete && compared && matched && failures.is_empty();
    capture["failures"] = json!(failures);
    capture["nativeGoldenComparisonPerformed"] = json!(compared);
    capture["nativeGoldensMatched"] = json!(matched);
    capture["wholePacketQualification"] = json!(false);
    capture["qualification"] = json!(if qualified {
        "complete-current-native-oracles-matched"
    } else {
        "failed-current-native-observation"
    });
    if let Ok(path) = std::env::var("VIZE_NATIVE_MODEL_TRIM_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_string_pretty(&capture).map_err(|error| format!("{error}"))?,
        )
        .map_err(|error| format!("{error}"))?;
    }
    println!(
        "{}",
        serde_json::to_string(&capture).map_err(|error| format!("{error}"))?
    );
    if qualified {
        Ok(())
    } else {
        Err(format!(
            "current trim native comparison failed: {}",
            capture["failures"]
        ))
    }
}
