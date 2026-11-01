//! Preserve complete current observations before the strict whole-array verdict.

use super::*;

pub(super) fn write(
    pack: &Value,
    captures: Vec<Value>,
    custody: Vec<Value>,
    attempts: Vec<Value>,
    refusals: Vec<Value>,
    mut failures: Vec<String>,
) -> Result<(), String> {
    let emitted = captures.len();
    let native = serde_json::json!({
        "fixtures":captures,"custody":custody,"attempts":attempts,"refusals":refusals,
    });
    let mut compared = false;
    match serde_json::from_str::<Value>(include_str!(
        "../../../../tests/fixtures/native_original_input_model_sfc_vue_3_5_35.json"
    )) {
        Ok(goldens) => {
            if goldens["schema"] != "vize.native-sfc.original-input-model-complete-goldens"
                || goldens["filename"] != pack["filename"]
                || goldens["inputPack"] != *pack
                || goldens["provenance"]["independentlyReviewedBeforeFreeze"] != true
            {
                failures.push(cstr!("complete input Model golden authority mismatch"));
            }
            compared = true;
            for field in ["fixtures", "custody", "attempts", "refusals"] {
                if goldens["native"].get(field) != native.get(field) {
                    failures.push(cstr!("whole ordered input Model {field} golden mismatch"));
                }
            }
        }
        Err(error) => failures.push(cstr!("complete input Model golden parse: {error}")),
    }
    let qualified = failures.is_empty() && compared && emitted == 14;
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_DOM_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-input-model-capture",
            "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
            "qualification":if qualified {"current-complete-native-goldens-matched"} else {"failed-current-complete-native-goldens"},
            "nativeGoldenComparisonPerformed":compared,"sourceRevision":std::env::var("GITHUB_SHA").ok(),
            "fixtures":native["fixtures"],"custody":native["custody"],"attempts":native["attempts"],"refusals":native["refusals"],"failures":failures,
        })).map_err(|error|cstr!("{error}"))?).map_err(|error|cstr!("{error}"))?;
    }
    require!(
        qualified,
        "complete current input Model golden qualification: {failures:?}"
    );
    Ok(())
}
