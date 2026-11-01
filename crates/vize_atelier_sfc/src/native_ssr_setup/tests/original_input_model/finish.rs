//! Whole current comparison retains the actual arrays before any verdict.

use super::*;

pub(super) fn write(
    pack: &Value,
    captures: Vec<Value>,
    owners: Vec<Value>,
    attempts: Vec<Value>,
    refusals: Vec<Value>,
    mut failures: Vec<String>,
) -> Result<(), String> {
    let complete = failures.is_empty()
        && captures.len() == 7
        && owners.len() == 7
        && attempts.len() == 7
        && refusals.len() == 12;
    let actual = serde_json::json!({"fixtures":captures,"custody":owners,
        "attempts":attempts,"refusals":refusals});
    let mut compared = false;
    let comparison = (|| -> Result<(), String> {
        let golden: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/native_original_input_model_ssr_sfc_vue_3_5_35.json"
        ))
        .map_err(|error| cstr!("{error}"))?;
        require!(
            golden["schema"] == "vize.native-sfc.original-input-model-ssr-complete-goldens"
                && golden["inputPack"] == *pack
                && golden["filename"] == pack["filename"]
                && golden["provenance"]["independentlyReviewedBeforeFreeze"] == true,
            "whole reviewed original SSR authority"
        );
        compared = true;
        for field in ["fixtures", "custody", "attempts", "refusals"] {
            if actual[field] != golden["native"][field] {
                failures.push(cstr!(
                    "whole current native SSR {field} differs from reviewed golden"
                ));
            }
        }
        Ok(())
    })();
    if let Err(error) = comparison {
        failures.push(error);
    }
    let qualified = complete && compared && failures.is_empty();
    let capture = serde_json::json!({
        "schema":"vize.native-sfc.original-input-model-ssr-capture",
        "adapter":"vize_atelier_sfc::compile_native_setup_ssr_sfc",
        "sourceRevision":std::env::var("GITHUB_SHA").ok(),"inputPack":pack,
        "qualification":if qualified {"current-complete-native-goldens-matched"}
            else {"failed-current-native-golden-comparison"},
        "nativeGoldenComparisonPerformed":compared,"nativeGoldensMatched":qualified,
        "failures":failures,"fixtures":actual["fixtures"],"custody":actual["custody"],
        "attempts":actual["attempts"],"refusals":actual["refusals"],
    });
    if let Ok(path) = std::env::var("VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_SSR_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&capture).map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    require!(
        qualified,
        "whole current SSR Model comparison failures: {}",
        capture["failures"]
    );
    Ok(())
}
