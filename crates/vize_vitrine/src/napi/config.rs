//! NAPI bindings for public config helpers.
//!
//! The npm package still owns dynamic module loading for `.ts` / `.mjs`
//! configs because it must execute user JavaScript with the caller-provided
//! environment. Once that raw value is available, normalization is pure shape
//! work, so it is handled here in Rust and shared with native callers.

mod js_value;

use napi::bindgen_prelude::{Result, Unknown};
use napi_derive::napi;

/// Normalize a raw `vize.config.*` export into the public resolved shape.
///
/// The implementation works directly with NAPI values instead of serializing
/// through JSON. That keeps JavaScript-only values such as `RegExp` filters
/// intact while still moving the merge, null stripping, and legacy alias
/// handling out of the TypeScript loader.
#[napi(js_name = "normalizeVizeConfig")]
pub fn normalize_vize_config(value: Unknown<'_>) -> Result<Unknown<'_>> {
    js_value::normalize_vize_config(value)
}
