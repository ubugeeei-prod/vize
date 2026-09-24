//! NAPI bindings for Art status diagnostics.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "N-API values cross the boundary as std `String`s"
)]

use napi_derive::napi;

use super::art::ArtParseOptionsNapi;

/// Return unknown-status warnings for an Art file (`*.art.vue`).
#[napi(js_name = "parseArtStatusWarnings")]
pub fn parse_art_status_warnings(
    source: String,
    options: Option<ArtParseOptionsNapi>,
) -> Vec<String> {
    use vize_musea::{Allocator, parse_art_status_warnings as musea_warnings};

    let allocator = Allocator::new();
    let filename = options
        .and_then(|opts| opts.filename)
        .unwrap_or_else(|| "anonymous.art.vue".to_string());
    musea_warnings(&allocator, &source, &filename)
        .iter()
        .map(|warning| (*warning).to_string())
        .collect()
}
