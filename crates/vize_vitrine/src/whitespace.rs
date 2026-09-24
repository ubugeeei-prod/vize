//! Decode the Vue compiler's public whitespace option at FFI boundaries.
#![expect(
    clippy::disallowed_macros,
    reason = "FFI option validation needs a formatted std String error"
)]
#![expect(
    clippy::disallowed_types,
    reason = "N-API and wasm-bindgen errors use std String"
)]

use vize_atelier_core::WhitespaceStrategy;

pub(crate) fn resolve_whitespace(value: Option<&str>) -> Result<WhitespaceStrategy, String> {
    match value {
        None | Some("condense") => Ok(WhitespaceStrategy::Condense),
        Some("preserve") => Ok(WhitespaceStrategy::Preserve),
        Some(value) => Err(format!(
            "Invalid whitespace `{value}`. Expected `condense` or `preserve`."
        )),
    }
}
