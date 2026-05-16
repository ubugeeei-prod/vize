//! N-API bindings for native Vite plugin request classification.
//!
//! The actual classification model lives in `vize_atelier_sfc`; vitrine only
//! converts that Rust shape into the JavaScript-facing N-API object.

#![allow(clippy::disallowed_types)]

mod request;

pub use request::VitePluginRequestNapi;

#[napi_derive::napi(js_name = "classifyVitePluginRequest")]
pub fn classify_vite_plugin_request(id: String) -> VitePluginRequestNapi {
    vize_atelier_sfc::vite_plugin::classify_vite_plugin_request(&id).into()
}
