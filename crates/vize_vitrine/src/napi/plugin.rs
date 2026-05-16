//! N-API bindings for native Vite plugin request classification.
//!
//! The actual classification model lives in `vize_atelier_sfc`; vitrine only
//! converts that Rust shape into the JavaScript-facing N-API object.

#![allow(clippy::disallowed_types)]

mod request;

use napi_derive::napi;
pub use request::VitePluginRequestNapi;

#[napi(object)]
pub struct DynamicImportAliasRuleNapi {
    pub from_prefix: String,
    pub to_prefix: String,
}

#[napi(object)]
pub struct DefineReplacementNapi {
    pub key: String,
    pub value: String,
}

#[napi(object)]
pub struct HmrHashesNapi {
    pub script_hash: Option<String>,
    pub template_hash: Option<String>,
    pub style_hash: Option<String>,
}

impl From<DynamicImportAliasRuleNapi> for vize_atelier_sfc::vite_plugin::DynamicImportAliasRule {
    fn from(rule: DynamicImportAliasRuleNapi) -> Self {
        Self {
            from_prefix: rule.from_prefix.into(),
            to_prefix: rule.to_prefix.into(),
        }
    }
}

impl From<DefineReplacementNapi> for vize_atelier_sfc::vite_plugin::DefineReplacement {
    fn from(define: DefineReplacementNapi) -> Self {
        Self {
            key: define.key.into(),
            value: define.value.into(),
        }
    }
}

impl From<HmrHashesNapi> for vize_atelier_sfc::vite_plugin::HmrHashes {
    fn from(hashes: HmrHashesNapi) -> Self {
        Self {
            script_hash: hashes.script_hash.map(Into::into),
            template_hash: hashes.template_hash.map(Into::into),
            style_hash: hashes.style_hash.map(Into::into),
        }
    }
}

#[napi(js_name = "classifyVitePluginRequest")]
pub fn classify_vite_plugin_request(id: String) -> VitePluginRequestNapi {
    vize_atelier_sfc::vite_plugin::classify_vite_plugin_request(&id).into()
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
