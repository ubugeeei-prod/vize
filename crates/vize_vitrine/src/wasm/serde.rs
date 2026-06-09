//! Shared serialization and offset-conversion helpers for the WASM surface.

use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Helper function to serialize values to JsValue with maps as objects
pub(crate) fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

pub(crate) fn to_json_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let json = serde_json::to_string(value).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_sys::JSON::parse(&json)
}

/// Convert a UTF-8 byte offset to a JavaScript string offset (UTF-16 code units).
/// OXC and the SFC parser report byte offsets, while Monaco/JS consumers expect
/// UTF-16 offsets.
pub(crate) fn utf8_byte_to_utf16_offset(content: &str, byte_offset: u32) -> u32 {
    let byte_offset = byte_offset as usize;
    if byte_offset >= content.len() {
        return content.encode_utf16().count() as u32;
    }
    content[..byte_offset].encode_utf16().count() as u32
}
