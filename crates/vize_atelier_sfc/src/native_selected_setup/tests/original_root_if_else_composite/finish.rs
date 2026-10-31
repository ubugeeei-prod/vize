//! Complete observation is written before the aggregate verdict.
use super::*;
pub(super) fn write(
    fixture_count: usize,
    captures: Vec<Value>,
    custody: Vec<Value>,
    attempts: Vec<Value>,
    mut failures: Vec<String>,
) -> Result<(), String> {
    let emitted = captures.len();
    let captures = Value::Array(captures);
    let custody = Value::Array(custody);
    let attempts = Value::Array(attempts);
    let mut compared = false;
    match serde_json::from_str::<Value>(include_str!(
        "../../../../tests/fixtures/native_original_root_if_else_composite_sfc_vue_3_5_35.json"
    )) {
        Ok(goldens) => {
            if goldens["schema"]
                != "vize.native-sfc.original-root-if-else-composite-complete-goldens"
                || goldens["filename"] != "OriginalRootIfElseComposite雪🌸.vue"
            {
                failures.push(cstr!("whole composite golden authority mismatch"));
            }
            compared = true;
            for (field, actual) in [
                ("fixtures", &captures),
                ("custody", &custody),
                ("attempts", &attempts),
            ] {
                if goldens["native"].get(field) != Some(actual) {
                    failures.push(cstr!("whole ordered native {field} golden mismatch"));
                }
            }
        }
        Err(error) => failures.push(cstr!("whole composite golden parse: {error}")),
    }
    let qualified = failures.is_empty() && emitted == fixture_count * 2;
    if let Some(path) =
        std::env::var_os("VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_COMPOSITE_SFC_DOM_CAPTURE")
    {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema":"vize.native-sfc.original-root-if-else-composite-capture",
                "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
            "qualification":if qualified { "current-complete-native-goldens-matched" } else { "failed-current-complete-native-goldens" },
            "nativeGoldenComparisonPerformed":compared,
                "sourceRevision":std::env::var("GITHUB_SHA").ok(),
                "fixtures":captures,"custody":custody,"attempts":attempts,"failures":&failures,
            }))
            .map_err(|error| cstr!("{error}"))?,
        )
        .map_err(|error| cstr!("{error}"))?;
    }
    require!(
        failures.is_empty(),
        "original If/Else capture failures: {failures:?}"
    );
    require!(
        emitted == fixture_count * 2,
        "all fourteen mode-major original pairs emitted"
    );
    Ok(())
}
