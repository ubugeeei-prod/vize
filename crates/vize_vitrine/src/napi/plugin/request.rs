#![allow(clippy::disallowed_macros)]

use napi_derive::napi;
use vize_atelier_sfc::vite_plugin::VitePluginRequest;

#[napi(object)]
pub struct VitePluginRequestNapi {
    /// Path segment before the query string.
    pub path: String,
    /// Query suffix including the leading `?`, or an empty string.
    pub query_suffix: String,
    /// Path normalized for macro virtual modules (`.vue.ts` -> `.vue`).
    pub normalized_vue_path: String,
    /// For `\0...` virtual macro IDs, the real path without the virtual prefix.
    pub stripped_virtual_path: Option<String>,
    /// Whether this ID is a Vize-compiled virtual Vue module.
    pub is_vize_virtual: bool,
    /// Whether this ID is a Vize SSR virtual Vue module.
    pub is_vize_ssr_virtual: bool,
    /// Real `.vue` path extracted from a Vize virtual Vue module ID.
    pub vize_virtual_path: Option<String>,
    /// Build-safe ID with Vite's `/@fs` prefix removed when present.
    pub normalized_fs_id: Option<String>,
    /// Whether the query contains `macro=true`.
    pub has_macro_query: bool,
    /// Whether the query contains `definePage`.
    pub has_define_page_query: bool,
    /// Whether this is a `\0` virtual ID carrying a macro query.
    pub is_macro_virtual_id: bool,
    /// Whether the request points at a Vue SFC after macro normalization.
    pub is_vue_sfc_path: bool,
    /// Whether the request is a Vite Vue style virtual query.
    pub is_vue_style_query: bool,
    /// Style block language, defaulting to `css` for style virtual queries.
    pub style_lang: Option<String>,
    /// Style block index for style virtual queries.
    pub style_index: Option<u32>,
    /// Scoped attribute value for style virtual queries.
    pub style_scoped: Option<String>,
    /// Whether the style query carries a CSS modules marker.
    pub has_style_module: bool,
    /// Extension suffix Vite should see for the style pipeline.
    pub style_virtual_suffix: Option<String>,
    /// Vue boundary file kind: `client`, `server`, or undefined.
    pub boundary_kind: Option<String>,
}

impl From<VitePluginRequest> for VitePluginRequestNapi {
    fn from(request: VitePluginRequest) -> Self {
        Self {
            path: request.path.into(),
            query_suffix: request.query_suffix.into(),
            normalized_vue_path: request.normalized_vue_path.into(),
            stripped_virtual_path: request.stripped_virtual_path.map(Into::into),
            is_vize_virtual: request.is_vize_virtual,
            is_vize_ssr_virtual: request.is_vize_ssr_virtual,
            vize_virtual_path: request.vize_virtual_path.map(Into::into),
            normalized_fs_id: request.normalized_fs_id.map(Into::into),
            has_macro_query: request.has_macro_query,
            has_define_page_query: request.has_define_page_query,
            is_macro_virtual_id: request.is_macro_virtual_id,
            is_vue_sfc_path: request.is_vue_sfc_path,
            is_vue_style_query: request.is_vue_style_query,
            style_lang: request.style_lang.map(Into::into),
            style_index: request.style_index,
            style_scoped: request.style_scoped.map(Into::into),
            has_style_module: request.has_style_module,
            style_virtual_suffix: request.style_virtual_suffix.map(Into::into),
            boundary_kind: request.boundary_kind.map(Into::into),
        }
    }
}
