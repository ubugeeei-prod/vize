use napi_derive::napi;

use super::types::{
    CustomBlockNapi, SfcBlockAttributeNapi, SfcSrcInfoNapi, StyleBlockNapi,
    TemplateAssetTagRuleNapi, TemplateAssetUrlNapi,
};

impl From<vize_atelier_sfc::BundlerStyleBlock> for StyleBlockNapi {
    fn from(block: vize_atelier_sfc::BundlerStyleBlock) -> Self {
        Self {
            content: block.content.into(),
            src: block.src.map(Into::into),
            lang: block.lang.map(Into::into),
            scoped: block.scoped,
            module: block.module,
            module_name: block.module_name.map(Into::into),
            index: block.index,
        }
    }
}

impl From<vize_atelier_sfc::SfcBlockAttribute> for SfcBlockAttributeNapi {
    fn from(attr: vize_atelier_sfc::SfcBlockAttribute) -> Self {
        Self {
            name: attr.name.into(),
            value: attr.value.map(Into::into),
        }
    }
}

impl From<vize_atelier_sfc::BundlerCustomBlock> for CustomBlockNapi {
    fn from(block: vize_atelier_sfc::BundlerCustomBlock) -> Self {
        Self {
            block_type: block.block_type.into(),
            content: block.content.into(),
            src: block.src.map(Into::into),
            attrs: block.attrs.into_iter().map(Into::into).collect(),
            index: block.index,
        }
    }
}

impl From<vize_atelier_sfc::SfcSrcInfo> for SfcSrcInfoNapi {
    fn from(info: vize_atelier_sfc::SfcSrcInfo) -> Self {
        Self {
            script_src: info.script_src.map(Into::into),
            template_src: info.template_src.map(Into::into),
        }
    }
}

impl From<vize_atelier_sfc::TemplateAssetUrl> for TemplateAssetUrlNapi {
    fn from(url: vize_atelier_sfc::TemplateAssetUrl) -> Self {
        Self {
            url: url.url.into(),
            var_name: url.var_name.into(),
        }
    }
}

impl From<TemplateAssetTagRuleNapi> for vize_atelier_sfc::TemplateAssetTagRule {
    fn from(rule: TemplateAssetTagRuleNapi) -> Self {
        Self {
            tag: rule.tag.into(),
            attrs: rule.attrs.into_iter().map(Into::into).collect(),
        }
    }
}

#[napi(js_name = "generateSfcScopeId")]
pub fn generate_sfc_scope_id(
    filename: String,
    root: Option<String>,
    is_production: Option<bool>,
    source: Option<String>,
) -> String {
    vize_atelier_sfc::generate_bundler_scope_id(
        &filename,
        root.as_deref(),
        is_production.unwrap_or(false),
        source.as_deref(),
    )
    .into()
}

#[napi(js_name = "extractSfcStyleBlocks")]
pub fn extract_sfc_style_blocks(source: String, filename: Option<String>) -> Vec<StyleBlockNapi> {
    vize_atelier_sfc::extract_style_blocks(&source, filename.as_deref())
        .into_iter()
        .map(Into::into)
        .collect()
}

#[napi(js_name = "extractSfcCustomBlocks")]
pub fn extract_sfc_custom_blocks(source: String, filename: Option<String>) -> Vec<CustomBlockNapi> {
    vize_atelier_sfc::extract_custom_blocks(&source, filename.as_deref())
        .into_iter()
        .map(Into::into)
        .collect()
}

#[napi(js_name = "extractSfcSrcInfo")]
pub fn extract_sfc_src_info(source: String, filename: Option<String>) -> SfcSrcInfoNapi {
    vize_atelier_sfc::extract_src_info(&source, filename.as_deref()).into()
}

#[napi(js_name = "hasSfcScopedStyle")]
pub fn has_sfc_scoped_style(source: String, filename: Option<String>) -> bool {
    vize_atelier_sfc::has_scoped_style(&source, filename.as_deref())
}

#[napi(js_name = "isSfcImportableAssetUrl")]
pub fn is_sfc_importable_asset_url(url: String) -> bool {
    vize_atelier_sfc::is_importable_asset_url(&url)
}

#[napi(js_name = "collectSfcTemplateAssetUrls")]
pub fn collect_sfc_template_asset_urls(
    source: String,
    rules: Option<Vec<TemplateAssetTagRuleNapi>>,
    filename: Option<String>,
) -> Vec<TemplateAssetUrlNapi> {
    let rules = rules.map(|rules| rules.into_iter().map(Into::into).collect::<Vec<_>>());
    vize_atelier_sfc::collect_template_asset_urls(&source, rules.as_deref(), filename.as_deref())
        .into_iter()
        .map(Into::into)
        .collect()
}

#[napi(js_name = "stripSfcScopedCssComments")]
pub fn strip_sfc_scoped_css_comments(css: String) -> String {
    vize_atelier_sfc::strip_css_comments_for_scoped(&css).into()
}

#[napi(js_name = "wrapSfcScopedPreprocessorStyle")]
pub fn wrap_sfc_scoped_preprocessor_style(
    content: String,
    scoped: Option<String>,
    lang: Option<String>,
) -> String {
    vize_atelier_sfc::wrap_scoped_preprocessor_style(&content, scoped.as_deref(), lang.as_deref())
        .into()
}
