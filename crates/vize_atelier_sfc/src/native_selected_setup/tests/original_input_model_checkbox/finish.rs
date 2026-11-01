//! Always persist actual attempts before the strict whole-array verdict.

use serde_json::{Value, json};

pub(super) fn write(mut capture: Value, complete: bool) -> Result<(), String> {
    let mut failures = capture["failures"].as_array().cloned().unwrap_or_default();
    let mut compared = false;
    match serde_json::from_str::<Value>(include_str!(
        "../../../../tests/fixtures/native_original_input_model_checkbox_all_targets.json"
    )) {
        Ok(goldens) => {
            if goldens["schema"] != "vize.native-sfc.original-input-model-checkbox-complete-goldens"
                || goldens["provenance"]["independentlyReviewedBeforeFreeze"] != true
                || goldens["inputHash"] != capture["inputHash"]
                || goldens["inputPack"] != capture["inputPack"]
                || goldens["refusalInputPack"] != capture["refusalInputPack"]
            {
                failures.push(json!("checkbox complete golden source authority differs"));
            } else {
                compared = true;
                for field in ["fixtures", "custody", "attempts", "refusals"] {
                    if capture[field] != goldens["native"][field] {
                        failures.push(json!(format!(
                            "whole current checkbox native {field} differs"
                        )));
                    }
                }
            }
        }
        Err(error) => failures.push(json!(format!("checkbox complete golden parse: {error}"))),
    }
    let matched = complete && compared && failures.is_empty();
    capture["failures"] = json!(failures);
    capture["nativeGoldenComparisonPerformed"] = json!(compared);
    capture["nativeGoldensMatched"] = json!(matched);
    capture["qualification"] = json!(if matched {
        "complete-current-native-oracles-matched"
    } else {
        "failed-current-native-observation"
    });
    // Native arrays alone never establish whole primary/runtime qualification.
    capture["wholePacketQualification"] = json!(false);
    let serialized = serde_json::to_string_pretty(&capture).map_err(|error| format!("{error}"))?;
    let first = capture["failures"]
        .as_array()
        .and_then(|values| values.first())
        .and_then(Value::as_str)
        .map(str::to_owned);
    let persistence = (|| -> Result<(), String> {
        if let Some(path) = std::env::var_os("VIZE_NATIVE_MODEL_CHECKBOX_CAPTURE") {
            let path = std::path::Path::new(&path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| format!("{error}"))?;
            }
            std::fs::write(path, &serialized).map_err(|error| format!("{error}"))?;
        }
        Ok(())
    })();
    if !matched || persistence.is_err() {
        eprintln!("CHECKBOX_MODEL_FAILED_CAPTURE={serialized}");
    }
    if !matched {
        let first = first.unwrap_or_else(|| "incomplete original checkbox observation".into());
        return Err(match persistence {
            Ok(()) => first,
            Err(error) => format!("{first}; additional capture-persistence failure: {error}"),
        });
    }
    persistence
}
