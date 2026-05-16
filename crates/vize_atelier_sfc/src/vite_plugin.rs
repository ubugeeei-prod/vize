//! Native request classification for the Vite plugin.
//!
//! Vite hook orchestration stays in TypeScript because it depends on Vite's
//! async resolver and plugin context. This module owns the deterministic pieces
//! that are easy to keep fast and strict in Rust: query parsing, virtual module
//! normalization, style virtual suffixes, and Vue boundary detection.

mod boundary;
mod css;
mod css_scope;
mod hmr;
mod js_string;
mod query;
mod request;
mod resolver;
mod style;
mod transform;

#[cfg(test)]
mod tests;

pub use css::{CssAliasRule, resolve_css_imports, scope_css_for_pipeline};
pub use hmr::{HmrHashes, detect_hmr_update_type, generate_hmr_code, has_hmr_changes};
pub use request::{
    VitePluginRequest, classify_vite_plugin_request, create_virtual_id, from_virtual_id,
    normalize_fs_id_for_build, normalize_virtual_vue_module_id,
};
pub use resolver::{
    ViteIdParts, create_bare_import_bases, create_bare_import_candidates, is_bare_specifier,
    normalize_require_base, normalize_resolved_vue_path, resolve_alias_request,
    resolve_relative_import, resolve_vue_path, split_id_query,
};
pub use transform::{
    DefineReplacement, DynamicImportAliasRule, apply_define_replacements, is_builtin_define,
    rewrite_dynamic_template_imports, rewrite_static_asset_urls,
    should_apply_define_in_virtual_module, to_browser_import_prefix,
};
