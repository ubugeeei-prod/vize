use napi_derive::napi;

#[napi(object)]
pub struct StyleBlockNapi {
    pub content: String,
    pub src: Option<String>,
    pub lang: Option<String>,
    pub scoped: bool,
    pub module: bool,
    pub module_name: Option<String>,
    pub index: u32,
}

#[napi(object)]
pub struct SfcBlockAttributeNapi {
    pub name: String,
    pub value: Option<String>,
}

#[napi(object)]
pub struct CustomBlockNapi {
    pub block_type: String,
    pub content: String,
    pub src: Option<String>,
    pub attrs: Vec<SfcBlockAttributeNapi>,
    pub index: u32,
}

#[napi(object)]
pub struct SfcSrcInfoNapi {
    pub script_src: Option<String>,
    pub template_src: Option<String>,
}

#[napi(object)]
pub struct TemplateAssetUrlNapi {
    pub url: String,
    pub var_name: String,
}

#[napi(object)]
pub struct TemplateAssetTagRuleNapi {
    pub tag: String,
    pub attrs: Vec<String>,
}

#[napi(object)]
pub struct MacroArtifactNapi {
    pub kind: String,
    pub name: String,
    pub source: String,
    pub content: String,
    pub module_code: Option<String>,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
#[derive(Default)]
pub struct SfcParseOptionsNapi {
    pub filename: Option<String>,
}

#[napi(object)]
#[derive(Default)]
pub struct SfcCompileOptionsNapi {
    pub filename: Option<String>,
    pub mode: Option<String>,
    pub source_map: Option<bool>,
    pub ssr: Option<bool>,
    pub vapor: Option<bool>,
    pub custom_renderer: Option<bool>,
    pub template_syntax: Option<String>,
    pub runtime_module_name: Option<String>,
    pub runtime_global_name: Option<String>,
    pub vue_version: Option<String>,
    /// Preserve TypeScript in output when true
    pub is_ts: Option<bool>,
    /// Scope ID for scoped CSS (e.g., "data-v-abc123")
    pub scope_id: Option<String>,
}

#[napi(object)]
pub struct SfcCompileResultNapi {
    pub code: String,
    pub css: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub template_hash: Option<String>,
    pub style_hash: Option<String>,
    pub script_hash: Option<String>,
    pub has_scoped: bool,
    pub styles: Vec<StyleBlockNapi>,
    pub custom_blocks: Vec<CustomBlockNapi>,
    pub macro_artifacts: Vec<MacroArtifactNapi>,
}

#[napi(object)]
#[derive(Default)]
pub struct BatchCompileOptionsNapi {
    pub mode: Option<String>,
    pub ssr: Option<bool>,
    pub vapor: Option<bool>,
    pub custom_renderer: Option<bool>,
    pub template_syntax: Option<String>,
    pub runtime_module_name: Option<String>,
    pub runtime_global_name: Option<String>,
    pub vue_version: Option<String>,
    /// Preserve TypeScript in output when true
    pub is_ts: Option<bool>,
    pub threads: Option<u32>,
}

#[napi(object)]
pub struct BatchCompileResultNapi {
    pub success: u32,
    pub failed: u32,
    pub input_bytes: u32,
    pub output_bytes: u32,
    pub time_ms: f64,
}

#[napi(object)]
pub struct BatchFileInputNapi {
    pub path: String,
    pub source: String,
}

#[napi(object)]
pub struct BatchFileResultNapi {
    pub path: String,
    pub code: String,
    pub css: Option<String>,
    pub scope_id: String,
    pub has_scoped: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub template_hash: Option<String>,
    pub style_hash: Option<String>,
    pub script_hash: Option<String>,
    pub styles: Vec<StyleBlockNapi>,
    pub custom_blocks: Vec<CustomBlockNapi>,
    pub macro_artifacts: Vec<MacroArtifactNapi>,
}

#[napi(object)]
pub struct BatchCompileResultWithFilesNapi {
    pub results: Vec<BatchFileResultNapi>,
    pub success_count: u32,
    pub failed_count: u32,
    pub time_ms: f64,
}

pub(super) fn macro_artifacts_to_napi(
    artifacts: Vec<vize_atelier_sfc::SfcMacroArtifact>,
) -> Vec<MacroArtifactNapi> {
    artifacts
        .into_iter()
        .map(|artifact| MacroArtifactNapi {
            kind: artifact.kind.into(),
            name: artifact.name.into(),
            source: artifact.source.into(),
            content: artifact.content.into(),
            module_code: artifact.module_code.map(Into::into),
            start: artifact.start as u32,
            end: artifact.end as u32,
        })
        .collect()
}

pub(super) fn style_blocks_to_napi(
    styles: &[vize_atelier_sfc::SfcStyleBlock],
) -> Vec<StyleBlockNapi> {
    styles
        .iter()
        .enumerate()
        .map(|(index, style)| {
            let module_attr = style.attrs.get("module");
            let module_name = module_attr.and_then(|value| {
                let value = value.as_ref();
                if value.is_empty() {
                    None
                } else {
                    Some(value.into())
                }
            });

            StyleBlockNapi {
                content: style.content.as_ref().into(),
                src: style.src.as_deref().map(Into::into),
                lang: style.lang.as_deref().map(Into::into),
                scoped: style.scoped,
                module: module_attr.is_some(),
                module_name,
                index: index as u32,
            }
        })
        .collect()
}

pub(super) fn custom_blocks_to_napi(
    blocks: &[vize_atelier_sfc::SfcCustomBlock],
) -> Vec<CustomBlockNapi> {
    blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let mut attrs = block
                .attrs
                .iter()
                .map(|(name, value)| SfcBlockAttributeNapi {
                    name: name.as_ref().into(),
                    value: (!value.is_empty()).then(|| value.as_ref().into()),
                })
                .collect::<Vec<_>>();
            attrs.sort_by(|left, right| left.name.cmp(&right.name));
            CustomBlockNapi {
                block_type: block.block_type.as_ref().into(),
                content: block.content.as_ref().into(),
                src: block.attrs.get("src").map(|value| value.as_ref().into()),
                attrs,
                index: index as u32,
            }
        })
        .collect()
}
