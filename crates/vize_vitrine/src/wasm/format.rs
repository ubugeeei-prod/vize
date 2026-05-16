//! Glyph (Formatter) WASM bindings.

use super::to_js_value;
use wasm_bindgen::prelude::*;

/// Format Vue SFC file
#[wasm_bindgen(js_name = "formatSfc")]
pub fn format_sfc_wasm(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    use vize_glyph::format_sfc;

    let opts = parse_format_options(options);
    match format_sfc(source, &opts) {
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
    use vize_glyph::format_template;

    let opts = parse_format_options(options);
    match format_template(source, &opts) {
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
