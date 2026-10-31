//! Complete observation is written before the aggregate verdict.
use super::*;
pub(super) fn write(
    fixture_count: usize,
    captures: Vec<Value>,
    custody: Vec<Value>,
    attempts: Vec<Value>,
    failures: Vec<String>,
) -> Result<(), String> {
    let emitted = captures.len();
    let captures = Value::Array(captures);
    let custody = Value::Array(custody);
    let attempts = Value::Array(attempts);
    let qualified = failures.is_empty() && emitted == fixture_count * 2;
    if let Some(path) =
        std::env::var_os("VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_COMPOSITE_SFC_DOM_CAPTURE")
    {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema":"vize.native-sfc.original-root-if-else-composite-capture",
                "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
            "qualification":if qualified { "observed-unqualified-current-oracles-pending" } else { "failed-current-source-observation" },
            "nativeGoldenComparisonPerformed":false,
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
