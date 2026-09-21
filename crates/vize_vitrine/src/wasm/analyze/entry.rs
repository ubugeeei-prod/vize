use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// `performance.now()`: milliseconds with microsecond-scale resolution
    /// (5 µs in a cross-origin-isolated page such as the playground).
    #[wasm_bindgen(js_namespace = performance, js_name = now)]
    fn performance_now() -> f64;
}

/// The browser's monotonic clock in nanoseconds, for the ladder timings.
fn host_clock_ns() -> u64 {
    // Non-negative and far below u64::MAX nanoseconds for any real session.
    (performance_now() * 1_000_000.0) as u64
}

/// Analyze Vue SFC for semantic information (scopes, bindings, etc.).
#[wasm_bindgen(js_name = "analyzeSfc")]
pub fn analyze_sfc_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let filename = js_sys::Reflect::get(&options, &JsValue::from_str("filename"))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "anonymous.vue".to_string());
    let in_tag_comments =
        js_sys::Reflect::get(&options, &JsValue::from_str("experimentalInTagComments"))
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
    let patterned_template = js_sys::Reflect::get(
        &options,
        &JsValue::from_str("experimentalPatternedTemplate"),
    )
    .ok()
    .and_then(|value| value.as_bool())
    .unwrap_or(false);
    let result = super::analyze_sfc_json_with_clock(
        source,
        &filename,
        in_tag_comments,
        patterned_template,
        &host_clock_ns,
    )
    .map_err(|message| JsValue::from_str(&message))?;
    crate::wasm::to_js_value(&result)
}
