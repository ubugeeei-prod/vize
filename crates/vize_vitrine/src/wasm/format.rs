//! Glyph (Formatter) WASM bindings.

use super::to_js_value;
use wasm_bindgen::prelude::*;

/// Format Vue SFC file
#[wasm_bindgen(js_name = "formatSfc")]
pub fn format_sfc_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    use vize_glyph::format_sfc_with_vue_version;

    let vue_version = parse_vue_version(&options)?;
    let opts = parse_format_options(options);
    match format_sfc_with_vue_version(source, &opts, vue_version) {
        Ok(result) => {
            let output = serde_json::json!({
                "code": result.code,
                "changed": result.changed,
            });
            to_js_value(&output)
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// Format Vue template content
#[wasm_bindgen(js_name = "formatTemplate")]
pub fn format_template_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    use vize_glyph::format_template_with_vue_version;

    let vue_version = parse_vue_version(&options)?;
    let opts = parse_format_options(options);
    match format_template_with_vue_version(source, &opts, vue_version) {
        Ok(result) => {
            let output = serde_json::json!({
                "code": result,
                "changed": result != source,
            });
            to_js_value(&output)
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// Format JavaScript/TypeScript content
#[wasm_bindgen(js_name = "formatScript")]
pub fn format_script_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    use vize_glyph::format_script;

    let opts = parse_format_options(options);
    match format_script(source, &opts) {
        Ok(result) => {
            let output = serde_json::json!({
                "code": result,
                "changed": result != source,
            });
            to_js_value(&output)
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// Parse format options from JsValue
pub(crate) fn parse_format_options(options: JsValue) -> vize_glyph::FormatOptions {
    serde_wasm_bindgen::from_value(options).unwrap_or_default()
}

// Keep Vue selection separate from the unchanged serialized FormatOptions.
fn parse_vue_version(options: &JsValue) -> Result<vize_glyph::VueVersion, JsValue> {
    if !options.is_object() {
        return Ok(vize_glyph::VueVersion::V3);
    }
    let value = js_sys::Reflect::get(options, &JsValue::from_str("vueVersion"))?;
    if value.is_undefined() {
        return Ok(vize_glyph::VueVersion::V3);
    }
    value
        .as_string()
        .and_then(|version| vize_glyph::VueVersion::from_config_str(&version).ok())
        .ok_or_else(|| JsValue::from_str("invalid vueVersion"))
}
