//! Deserialize only Rust-owned fixture fields; JS UTF16 values remain authored.
use super::*;

#[derive(serde::Deserialize)]
pub(in crate::native_selected_setup::tests) struct Pack {
    pub filename: String,
    pub fixtures: Vec<Fixture>,
    #[serde(rename = "rangeControls")]
    pub range_controls: Vec<Fixture>,
}

#[derive(serde::Deserialize)]
pub(in crate::native_selected_setup::tests) struct Fixture {
    pub id: String,
    pub source: String,
    pub dynamic: bool,
    #[serde(rename = "expectedCode")]
    pub expected_code: String,
    #[serde(rename = "nativeMap")]
    pub native_map: Value,
    #[serde(rename = "nativeMapRaw")]
    pub native_map_raw: String,
}

// Unknown runtime-only fields are IgnoredAny, not decoded into Rust UTF8.
// In particular, the unchanged JS values preserve the lone UTF16 surrogates
// produced by Vue renderList's string-index semantics.
pub(in crate::native_selected_setup::tests) fn pack() -> Result<Pack, String> {
    serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_original_for_constant_sfc_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))
}

pub(in crate::native_selected_setup::tests) fn inherited_pack() -> Result<Pack, String> {
    serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_original_for_constant_inherited_sfc_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))
}
