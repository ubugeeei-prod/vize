use napi::Result;
use napi_derive::napi;

#[napi(object)]
#[derive(Default)]
pub struct CssCompileOptionsNapi {
    pub filename: Option<String>,
    pub scoped: Option<bool>,
    pub scope_id: Option<String>,
    pub source_map: Option<bool>,
    pub minify: Option<bool>,
    pub css_modules: Option<bool>,
    pub custom_media: Option<bool>,
    pub targets: Option<CssTargetsNapi>,
}

#[napi(object)]
#[derive(Default)]
pub struct CssTargetsNapi {
    pub chrome: Option<u32>,
    pub firefox: Option<u32>,
    pub safari: Option<u32>,
    pub edge: Option<u32>,
    pub ios: Option<u32>,
    pub android: Option<u32>,
}

#[napi(object)]
pub struct CssCompileResultNapi {
    pub code: String,
    pub map: Option<String>,
    pub css_vars: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[napi(js_name = "compileCss")]
pub fn compile_css_napi(
    source: String,
    options: Option<CssCompileOptionsNapi>,
) -> Result<CssCompileResultNapi> {
    use vize_atelier_sfc::{CssCompileOptions, CssTargets, compile_css};

    let opts = options.unwrap_or_default();
    let targets = opts.targets.map(|targets| CssTargets {
        chrome: targets.chrome,
        firefox: targets.firefox,
        safari: targets.safari,
        edge: targets.edge,
        ios: targets.ios,
        android: targets.android,
    });

    let result = compile_css(
        &source,
        &CssCompileOptions {
            filename: opts.filename.map(Into::into),
            scoped: opts.scoped.unwrap_or(false),
            scope_id: opts.scope_id.map(Into::into),
            source_map: opts.source_map.unwrap_or(false),
            minify: opts.minify.unwrap_or(false),
            css_modules: opts.css_modules.unwrap_or(false),
            custom_media: opts.custom_media.unwrap_or(false),
            targets,
        },
    );

    Ok(CssCompileResultNapi {
        code: result.code.into(),
        map: result.map.map(Into::into),
        css_vars: result.css_vars.into_iter().map(Into::into).collect(),
        errors: result.errors.into_iter().map(Into::into).collect(),
        warnings: result.warnings.into_iter().map(Into::into).collect(),
    })
}
