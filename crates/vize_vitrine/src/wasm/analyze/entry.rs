use wasm_bindgen::prelude::*;

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
    let result = super::analyze_sfc_json_with_options(
        source,
        &filename,
        in_tag_comments,
        patterned_template,
    )
    .map_err(|message| JsValue::from_str(&message))?;
    crate::wasm::to_js_value(&result)
}
