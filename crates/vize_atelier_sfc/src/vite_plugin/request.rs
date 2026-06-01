use vize_carton::String;

use super::{
    boundary::boundary_kind,
    query::{query_has_key, query_value_is, split_request},
    style::classify_style,
};

const VIZE_SSR_PREFIX: &str = "\0vize-ssr:";

/// Native classification of a Vite plugin module request.
///
/// The TypeScript plugin calls this once per hook and then uses the resulting
/// facts instead of repeatedly reparsing Vite's stringly module IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VitePluginRequest {
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

/// Classifies a Vite module ID into normalized native facts.
pub fn classify_vite_plugin_request(id: &str) -> VitePluginRequest {
    let split = split_request(id);
    let normalized_vue_path = normalize_vue_path(split.path);
    let has_macro_query = query_value_is(split.query, "macro", "true");
    let has_define_page_query = query_has_key(split.query, "definePage");
    let style = classify_style(split.query);
    let is_vize_virtual = is_vize_virtual_vue_module_id(id);
    let is_vize_ssr_virtual = id.starts_with(VIZE_SSR_PREFIX);

    VitePluginRequest {
        path: String::from(split.path),
        query_suffix: String::from(split.query_suffix),
        normalized_vue_path: String::from(normalized_vue_path),
        stripped_virtual_path: stripped_virtual_query_path(id),
        is_vize_virtual,
        is_vize_ssr_virtual,
        vize_virtual_path: is_vize_virtual.then(|| vize_virtual_path(id)),
        normalized_fs_id: normalized_fs_id(id),
        has_macro_query,
        has_define_page_query,
        is_macro_virtual_id: id.starts_with('\0') && (has_macro_query || has_define_page_query),
        is_vue_sfc_path: normalized_vue_path.ends_with(".vue"),
        is_vue_style_query: style.is_vue_style_query,
        style_lang: style.lang,
        style_index: style.index,
        style_scoped: style.scoped,
        has_style_module: style.has_module,
        style_virtual_suffix: style.virtual_suffix,
        boundary_kind: boundary_kind(normalized_vue_path).map(String::from),
    }
}

/// Create the Vize virtual module ID for a real Vue SFC path.
pub fn create_virtual_id(real_path: &str, ssr: bool) -> String {
    let suffix = ".ts";
    let mut virtual_id = String::with_capacity(
        real_path.len() + suffix.len() + if ssr { VIZE_SSR_PREFIX.len() } else { 1 },
    );
    if ssr {
        virtual_id.push_str(VIZE_SSR_PREFIX);
    } else {
        virtual_id.push('\0');
    }
    virtual_id.push_str(real_path);
    virtual_id.push_str(suffix);
    virtual_id
}

/// Extract the real Vue path from a Vize virtual module ID.
pub fn from_virtual_id(virtual_id: &str) -> String {
    let request = classify_vite_plugin_request(virtual_id);
    if let Some(path) = request.vize_virtual_path {
        return path;
    }
    let normalized = normalize_virtual_vue_module_id(virtual_id);
    let split = super::query::split_request(&normalized);
    String::from(split.path)
}

/// Normalize Vize virtual Vue IDs to their real Vue path plus query suffix.
pub fn normalize_virtual_vue_module_id(id: &str) -> String {
    let request = classify_vite_plugin_request(id);
    if let Some(path) = request.vize_virtual_path {
        let mut normalized = String::with_capacity(path.len() + request.query_suffix.len());
        normalized.push_str(path.as_str());
        normalized.push_str(request.query_suffix.as_str());
        return normalized;
    }

    let mut normalized =
        String::with_capacity(request.normalized_vue_path.len() + request.query_suffix.len());
    normalized.push_str(request.normalized_vue_path.as_str());
    normalized.push_str(request.query_suffix.as_str());
    normalized
}

/// Normalize Vite `/@fs` IDs for build output.
pub fn normalize_fs_id_for_build(id: &str) -> String {
    classify_vite_plugin_request(id)
        .normalized_fs_id
        .unwrap_or_else(|| String::from(id))
}

fn normalize_vue_path(path: &str) -> &str {
    path.strip_suffix(".ts")
        .filter(|normalized| normalized.ends_with(".vue"))
        .unwrap_or(path)
}

fn stripped_virtual_query_path(id: &str) -> Option<String> {
    id.strip_prefix('\0').map(|without_prefix| {
        let split = split_request(without_prefix);
        String::from(normalize_vue_path(split.path))
    })
}

fn is_vize_virtual_vue_module_id(id: &str) -> bool {
    id.starts_with('\0') && split_request(id).path.ends_with(".vue.ts")
}

fn vize_virtual_path(id: &str) -> String {
    let prefix_len = if id.starts_with(VIZE_SSR_PREFIX) {
        VIZE_SSR_PREFIX.len()
    } else {
        1
    };
    let without_prefix = &id[prefix_len..];
    let split = split_request(without_prefix);
    String::from(normalize_vue_path(split.path))
}

fn normalized_fs_id(id: &str) -> Option<String> {
    let split = split_request(id);
    let path = split.path.strip_prefix("/@fs")?;
    let mut normalized = String::with_capacity(path.len() + split.query_suffix.len());
    normalized.push_str(path);
    normalized.push_str(split.query_suffix);
    Some(normalized)
}
