use vize_carton::String;

use super::query::{parse_u32, query_has_key, query_value};

/// Classification result for Vite's Vue style virtual query.
pub struct StyleRequest {
    /// Whether the request is a Vue style virtual module.
    pub is_vue_style_query: bool,
    /// Style language passed to downstream CSS plugins.
    pub lang: Option<String>,
    /// Original SFC style block index.
    pub index: Option<u32>,
    /// Scoped CSS id carried by Vue's virtual query.
    pub scoped: Option<String>,
    /// Whether the style block is a CSS modules request.
    pub has_module: bool,
    /// File extension suffix that lets Vite pick the right style pipeline.
    pub virtual_suffix: Option<String>,
}

/// Classifies Vue style virtual query metadata without allocating on misses.
pub fn classify_style(query: &str) -> StyleRequest {
    let is_vue_style_query = query.contains("vue&type=style") || query.contains("vue=&type=style");
    if !is_vue_style_query {
        return StyleRequest {
            is_vue_style_query,
            lang: None,
            index: None,
            scoped: None,
            has_module: false,
            virtual_suffix: None,
        };
    }

    let lang = query_value(query, "lang")
        .filter(|value| !value.is_empty())
        .unwrap_or("css");
    let has_module = query_has_key(query, "module");

    StyleRequest {
        is_vue_style_query,
        lang: Some(String::from(lang)),
        index: query_value(query, "index").and_then(parse_u32),
        scoped: query_value(query, "scoped")
            .filter(|value| !value.is_empty())
            .map(String::from),
        has_module,
        virtual_suffix: Some(build_virtual_suffix(lang, has_module)),
    }
}

fn build_virtual_suffix(lang: &str, has_module: bool) -> String {
    let mut suffix = String::with_capacity(lang.len() + if has_module { 9 } else { 1 });
    suffix.push('.');
    if has_module {
        suffix.push_str("module.");
    }
    suffix.push_str(lang);
    suffix
}
