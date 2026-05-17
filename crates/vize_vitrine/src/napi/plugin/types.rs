use napi_derive::napi;

#[napi(object)]
pub struct CssAliasRuleNapi {
    pub find: String,
    pub replacement: String,
    pub is_regex: bool,
    pub flags: Option<String>,
}

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

#[napi(object)]
pub struct ViteIdPartsNapi {
    pub request: String,
    pub query_suffix: String,
}

#[napi(object)]
pub struct ViteDevMiddlewareRewriteNapi {
    pub cleaned_url: String,
    pub fs_path: String,
}

impl From<DynamicImportAliasRuleNapi> for vize_atelier_sfc::vite_plugin::DynamicImportAliasRule {
    fn from(rule: DynamicImportAliasRuleNapi) -> Self {
        Self {
            from_prefix: rule.from_prefix.into(),
            to_prefix: rule.to_prefix.into(),
        }
    }
}

impl From<CssAliasRuleNapi> for vize_atelier_sfc::vite_plugin::CssAliasRule {
    fn from(rule: CssAliasRuleNapi) -> Self {
        Self {
            find: rule.find.into(),
            replacement: rule.replacement.into(),
            is_regex: rule.is_regex,
            flags: rule.flags.map(Into::into),
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

impl From<vize_atelier_sfc::vite_plugin::ViteIdParts> for ViteIdPartsNapi {
    fn from(parts: vize_atelier_sfc::vite_plugin::ViteIdParts) -> Self {
        Self {
            request: parts.request.into(),
            query_suffix: parts.query_suffix.into(),
        }
    }
}

impl From<vize_atelier_sfc::vite_plugin::ViteDevMiddlewareRewrite>
    for ViteDevMiddlewareRewriteNapi
{
    fn from(rewrite: vize_atelier_sfc::vite_plugin::ViteDevMiddlewareRewrite) -> Self {
        Self {
            cleaned_url: rewrite.cleaned_url.into(),
            fs_path: rewrite.fs_path.into(),
        }
    }
}
