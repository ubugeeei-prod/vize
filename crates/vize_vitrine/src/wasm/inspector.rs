//! Compiler inspector WASM bindings.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use super::to_js_value;
use vize_curator::inspector::{InspectorSourceFile, build_diff, build_graph};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "buildInspectorGraph")]
pub fn build_inspector_graph(files: JsValue) -> Result<JsValue, JsValue> {
    let files: Vec<InspectorSourceFile> = serde_wasm_bindgen::from_value(files)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    to_js_value(&build_graph(&files))
}

#[wasm_bindgen(js_name = "buildInspectorDiff")]
pub fn build_inspector_diff(left: &str, right: &str) -> Result<JsValue, JsValue> {
    to_js_value(&build_diff(left, right))
}
