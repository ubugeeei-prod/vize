//! N-API bindings for native Vite plugin request classification.
//!
//! The actual classification model lives in `vize_atelier_sfc`; vitrine only
//! converts that Rust shape into the JavaScript-facing N-API object.

#![allow(clippy::disallowed_types)]

mod precompile;
mod request;
mod types;

use napi_derive::napi;
pub use precompile::*;
pub use request::VitePluginRequestNapi;
pub use types::*;

#[napi(js_name = "classifyVitePluginRequest")]
pub fn classify_vite_plugin_request(id: String) -> VitePluginRequestNapi {
    vize_atelier_sfc::vite_plugin::classify_vite_plugin_request(&id).into()
}

#[napi(js_name = "normalizeViteCssModuleFilename")]
pub fn normalize_vite_css_module_filename(filename: String) -> String {
    vize_atelier_sfc::vite_plugin::normalize_css_module_filename(&filename).into()
}

#[napi(js_name = "normalizeViteDevMiddlewareUrl")]
pub fn normalize_vite_dev_middleware_url(req_url: String) -> Option<ViteDevMiddlewareRewriteNapi> {
    vize_atelier_sfc::vite_plugin::normalize_dev_middleware_url(&req_url).map(Into::into)
}

#[napi(js_name = "scopeViteCssForPipeline")]
pub fn scope_vite_css_for_pipeline(css: String, scope_id: String) -> String {
    vize_atelier_sfc::vite_plugin::scope_css_for_pipeline(&css, &scope_id).into()
}

#[napi(js_name = "transformViteCssVarsForPipeline")]
pub fn transform_vite_css_vars_for_pipeline(css: String, scope_id: String) -> String {
    vize_atelier_sfc::vite_plugin::transform_css_vars_for_pipeline(&css, &scope_id).into()
}

#[napi(js_name = "resolveViteCssImports")]
pub fn resolve_vite_css_imports(
    css: String,
    importer: String,
    alias_rules: Vec<CssAliasRuleNapi>,
    is_dev: Option<bool>,
    dev_url_base: Option<String>,
) -> String {
    let alias_rules = alias_rules.into_iter().map(Into::into).collect::<Vec<_>>();
    vize_atelier_sfc::vite_plugin::resolve_css_imports(
        &css,
        &importer,
        &alias_rules,
        is_dev.unwrap_or(false),
        dev_url_base.as_deref(),
    )
    .into()
}

#[napi(js_name = "splitViteIdQuery")]
pub fn split_vite_id_query(id: String) -> ViteIdPartsNapi {
    vize_atelier_sfc::vite_plugin::split_id_query(&id).into()
}

#[napi(js_name = "isViteBareSpecifier")]
pub fn is_vite_bare_specifier(id: String) -> bool {
    vize_atelier_sfc::vite_plugin::is_bare_specifier(&id)
}

#[napi(js_name = "normalizeViteRequireBase")]
pub fn normalize_vite_require_base(importer: Option<String>) -> Option<String> {
    vize_atelier_sfc::vite_plugin::normalize_require_base(importer.as_deref()).map(Into::into)
}

#[napi(js_name = "resolveViteAliasRequest")]
pub fn resolve_vite_alias_request(
    id: String,
    alias_rules: Vec<CssAliasRuleNapi>,
) -> Option<String> {
    let alias_rules = alias_rules.into_iter().map(Into::into).collect::<Vec<_>>();
    vize_atelier_sfc::vite_plugin::resolve_alias_request(&id, &alias_rules).map(Into::into)
}

#[napi(js_name = "normalizeViteResolvedVuePath")]
pub fn normalize_vite_resolved_vue_path(id: String) -> Option<String> {
    vize_atelier_sfc::vite_plugin::normalize_resolved_vue_path(&id).map(Into::into)
}

#[napi(js_name = "resolveViteVuePath")]
pub fn resolve_vite_vue_path(root: String, id: String, importer: Option<String>) -> String {
    vize_atelier_sfc::vite_plugin::resolve_vue_path(&root, &id, importer.as_deref()).into()
}

#[napi(js_name = "createViteBareImportBases")]
pub fn create_vite_bare_import_bases(root: String, importer: Option<String>) -> Vec<String> {
    vize_atelier_sfc::vite_plugin::create_bare_import_bases(&root, importer.as_deref())
        .into_iter()
        .map(Into::into)
        .collect()
}

#[napi(js_name = "createViteBareImportCandidates")]
pub fn create_vite_bare_import_candidates(
    id: String,
    alias_rules: Vec<CssAliasRuleNapi>,
    resolved_id: Option<String>,
) -> Vec<String> {
    let alias_rules = alias_rules.into_iter().map(Into::into).collect::<Vec<_>>();
    vize_atelier_sfc::vite_plugin::create_bare_import_candidates(
        &id,
        &alias_rules,
        resolved_id.as_deref(),
    )
    .into_iter()
    .map(Into::into)
    .collect()
}

#[napi(js_name = "resolveViteRelativeImport")]
pub fn resolve_vite_relative_import(id: String, importer: String) -> Option<String> {
    vize_atelier_sfc::vite_plugin::resolve_relative_import(&id, &importer).map(Into::into)
}

#[napi(js_name = "createViteVirtualId")]
pub fn create_vite_virtual_id(real_path: String, ssr: Option<bool>) -> String {
    vize_atelier_sfc::vite_plugin::create_virtual_id(&real_path, ssr.unwrap_or(false)).into()
}

#[napi(js_name = "fromViteVirtualId")]
pub fn from_vite_virtual_id(virtual_id: String) -> String {
    vize_atelier_sfc::vite_plugin::from_virtual_id(&virtual_id).into()
}

#[napi(js_name = "normalizeViteVirtualVueModuleId")]
pub fn normalize_vite_virtual_vue_module_id(id: String) -> String {
    vize_atelier_sfc::vite_plugin::normalize_virtual_vue_module_id(&id).into()
}

#[napi(js_name = "normalizeViteFsIdForBuild")]
pub fn normalize_vite_fs_id_for_build(id: String) -> String {
    vize_atelier_sfc::vite_plugin::normalize_fs_id_for_build(&id).into()
}

#[napi(js_name = "toViteBrowserImportPrefix")]
pub fn to_vite_browser_import_prefix(replacement: String) -> String {
    vize_atelier_sfc::vite_plugin::to_browser_import_prefix(&replacement).into()
}

#[napi(js_name = "rewriteViteStaticAssetUrls")]
pub fn rewrite_vite_static_asset_urls(
    code: String,
    alias_rules: Vec<DynamicImportAliasRuleNapi>,
) -> String {
    let alias_rules = alias_rules.into_iter().map(Into::into).collect::<Vec<_>>();
    vize_atelier_sfc::vite_plugin::rewrite_static_asset_urls(&code, &alias_rules).into()
}

#[napi(js_name = "rewriteViteDynamicTemplateImports")]
pub fn rewrite_vite_dynamic_template_imports(
    code: String,
    alias_rules: Vec<DynamicImportAliasRuleNapi>,
) -> String {
    let alias_rules = alias_rules.into_iter().map(Into::into).collect::<Vec<_>>();
    vize_atelier_sfc::vite_plugin::rewrite_dynamic_template_imports(&code, &alias_rules).into()
}

#[napi(js_name = "rewriteViteImportMetaGlobBase")]
pub fn rewrite_vite_import_meta_glob_base(code: String, importer: String, root: String) -> String {
    vize_atelier_sfc::vite_plugin::rewrite_import_meta_glob_base(&code, &importer, &root).into()
}

#[napi(js_name = "isBuiltinViteDefine")]
pub fn is_builtin_vite_define(key: String) -> bool {
    vize_atelier_sfc::vite_plugin::is_builtin_define(&key)
}

#[napi(js_name = "shouldApplyViteDefineInVirtualModule")]
pub fn should_apply_vite_define_in_virtual_module(key: String) -> bool {
    vize_atelier_sfc::vite_plugin::should_apply_define_in_virtual_module(&key)
}

#[napi(js_name = "applyViteDefineReplacements")]
pub fn apply_vite_define_replacements(code: String, defines: Vec<DefineReplacementNapi>) -> String {
    let defines = defines.into_iter().map(Into::into).collect::<Vec<_>>();
    vize_atelier_sfc::vite_plugin::apply_define_replacements(&code, &defines).into()
}

#[napi(js_name = "hasViteHmrChanges")]
pub fn has_vite_hmr_changes(prev: Option<HmrHashesNapi>, next: HmrHashesNapi) -> bool {
    let prev = prev.map(Into::into);
    let next = next.into();
    vize_atelier_sfc::vite_plugin::has_hmr_changes(prev.as_ref(), &next)
}

#[napi(js_name = "detectViteHmrUpdateType")]
pub fn detect_vite_hmr_update_type(prev: Option<HmrHashesNapi>, next: HmrHashesNapi) -> String {
    let prev = prev.map(Into::into);
    let next = next.into();
    vize_atelier_sfc::vite_plugin::detect_hmr_update_type(prev.as_ref(), &next).into()
}

#[napi(js_name = "generateViteHmrCode")]
pub fn generate_vite_hmr_code(scope_id: String, update_type: String) -> String {
    vize_atelier_sfc::vite_plugin::generate_hmr_code(&scope_id, &update_type).into()
}
