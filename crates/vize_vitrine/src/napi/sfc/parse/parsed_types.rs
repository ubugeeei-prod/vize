//! Shared declaration schema; the existing JSON serializer remains unchanged.

use napi_derive::napi;
use serde::Serialize;
use serde_json::Value;

/// An effective parsed attribute. Boolean and empty values remain empty strings.
#[napi(object)]
#[derive(Serialize)]
pub struct SfcParsedAttributeNapi {
    pub name: String,
    pub value: String,
}

/// All offsets and columns use UTF-8 bytes in the original source.
#[napi(object)]
pub struct SfcParsedLocationNapi {
    pub start: f64,
    pub end: f64,
    pub tag_start: f64,
    pub tag_end: f64,
    pub start_line: f64,
    pub start_column: f64,
    pub end_line: f64,
    pub end_column: f64,
}

#[napi(object)]
pub struct SfcParsedTemplateNapi {
    pub content: String,
    pub loc: SfcParsedLocationNapi,
    #[napi(ts_type = "string | null")]
    pub lang: Value,
    pub attrs: Option<Vec<SfcParsedAttributeNapi>>,
}

#[napi(object)]
pub struct SfcParsedScriptNapi {
    pub content: String,
    pub loc: SfcParsedLocationNapi,
    #[napi(ts_type = "string | null")]
    pub lang: Value,
    pub setup: bool,
    pub attrs: Option<Vec<SfcParsedAttributeNapi>>,
}

#[napi(object)]
pub struct SfcParsedStyleNapi {
    pub content: String,
    pub loc: SfcParsedLocationNapi,
    #[napi(ts_type = "string | null")]
    pub lang: Value,
    pub scoped: bool,
    #[napi(ts_type = "string | null")]
    pub module: Value,
    pub attrs: Option<Vec<SfcParsedAttributeNapi>>,
}

#[napi(object)]
pub struct SfcParsedCustomBlockNapi {
    #[napi(js_name = "type")]
    pub block_type: String,
    pub content: String,
    pub loc: SfcParsedLocationNapi,
    pub attrs: Option<Vec<SfcParsedAttributeNapi>>,
}

#[napi(object)]
pub struct SfcParseResultNapi {
    pub filename: String,
    pub source: String,
    #[napi(ts_type = "SfcParsedTemplateNapi | null")]
    pub template: Value,
    #[napi(ts_type = "SfcParsedScriptNapi | null")]
    pub script: Value,
    #[napi(ts_type = "SfcParsedScriptNapi | null")]
    pub script_setup: Value,
    pub styles: Vec<SfcParsedStyleNapi>,
    pub custom_blocks: Vec<SfcParsedCustomBlockNapi>,
}
