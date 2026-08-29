use napi::{Result, Status};
use napi_derive::napi;
use vize_atelier_sfc::build_sfc_source_map;
use vize_atelier_sfc::compile_script::typescript::ensure_javascript_output;
use vize_s0::cstr;

use super::types::ModuleShapeNapi;
use super::{
    experimentals::ExperimentalTemplateOptions,
    types::{
        SfcCompileOptionsNapi, SfcCompileResultNapi, custom_blocks_to_napi,
        macro_artifacts_to_napi, style_blocks_to_napi,
    },
};
use crate::template_syntax::resolve_template_syntax;

#[napi(js_name = "compileSfc")]
pub fn compile_sfc(
    source: String,
    options: Option<SfcCompileOptionsNapi>,
) -> Result<SfcCompileResultNapi> {
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
        StyleCompileOptions, TemplateCompileOptions,
        compile_sfc_for_adapter as sfc_compile_for_adapter, parse_sfc as sfc_parse,
    };

    let opts = options.unwrap_or_default();
    let filename: vize_s0::CompactString =
        opts.filename.as_deref().unwrap_or("anonymous.vue").into();
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor = match sfc_parse(&source, parse_opts) {
        Ok(d) => d,
        Err(e) => {
            return Ok(SfcCompileResultNapi {
                code: String::new(),
                map: None,
                css: None,
                errors: vec![e.message.into()],
                warnings: vec![],
                template_hash: None,
                style_hash: None,
                script_hash: None,
                has_scoped: false,
                styles: vec![],
                custom_blocks: vec![],
                macro_artifacts: vec![],
                module_shape: None,
            });
        }
    };

    let template_hash: Option<String> = descriptor.template_hash().map(Into::into);
    let style_hash: Option<String> = descriptor.style_hash().map(Into::into);
    let script_hash: Option<String> = descriptor.script_hash().map(Into::into);
    let styles = style_blocks_to_napi(&descriptor.styles);
    let custom_blocks = custom_blocks_to_napi(&descriptor.custom_blocks);
    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let vapor = opts.vapor.unwrap_or(false);
    let is_ts = opts.is_ts.unwrap_or(false);
    let source_map = opts.source_map.unwrap_or(false);
    let experimentals = ExperimentalTemplateOptions::from_compile(&opts);
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())
        .map_err(|message| napi::Error::new(Status::InvalidArg, message))?;
    let standalone = opts.mode.as_deref() == Some("function");
    let custom_elements = vize_atelier_core::options::CustomElementMatcher::from_patterns(
        crate::types::custom_element_patterns(opts.custom_elements.as_deref()),
    );
    let external_scope_id: Option<vize_s0::CompactString> = opts
        .scope_id
        .as_ref()
        .map(|sid| sid.strip_prefix("data-v-").unwrap_or(sid).into());
    let template_compiler_options = {
        let scope_id = if has_scoped {
            external_scope_id
                .as_ref()
                .map(|scope_id| cstr!("data-v-{scope_id}"))
        } else {
            None
        };
        let mut dom_options = experimentals.dom_options();
        dom_options.scope_id = scope_id;
        if let Some(value) = opts.template_cache_handlers {
            dom_options.cache_handlers = value;
        }
        if let Some(value) = opts.template_comments {
            dom_options.comments = value;
        }
        if let Some(value) = opts.template_hoist_static {
            dom_options.hoist_static = value;
        }
        if let Some(value) = opts.template_prefix_identifiers {
            dom_options.prefix_identifiers = value;
        }
        Some(dom_options)
    };

    let compile_opts = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            inline_template: standalone,
            is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr: opts.ssr.unwrap_or(false),
            is_ts,
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            compiler_options: template_compiler_options,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename,
            scoped: has_scoped,
            trim: opts.style_trim.unwrap_or(false),
            ..Default::default()
        },
        vapor,
        scope_id: external_scope_id,
    };

    let compile_result = sfc_compile_for_adapter(
        &descriptor,
        compile_opts,
        template_syntax,
        custom_elements,
        vize_atelier_core::CodegenOptions::default(),
        if standalone {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
    );

    match compile_result {
        Ok(result) => {
            // The emitter is the last stop before the bundler, so the code
            // crossing this boundary must already be plain JavaScript — the JS
            // plugin no longer re-strips it. `is_ts` callers opted out: they
            // asked for TypeScript in the output and strip it themselves.
            let code: String = if is_ts {
                result.code.into()
            } else {
                ensure_javascript_output(result.code).into()
            };
            // Analyzed from the very bytes that cross the boundary, after every
            // rewriting pass, so the offsets cannot be stale (#3425).
            let module_shape = ModuleShapeNapi::of(&code);
            // Built from those same bytes for the same reason: stripping
            // TypeScript above re-prints the module, so a map produced before
            // that pass would describe code nobody receives (#3399).
            let map = source_map
                .then(|| {
                    let path = opts.filename.as_deref().unwrap_or("anonymous.vue");
                    build_sfc_source_map(&code, &descriptor, path)
                })
                .flatten()
                .map(Into::into);
            Ok(SfcCompileResultNapi {
                code,
                map,
                css: result.css.map(Into::into),
                errors: result
                    .errors
                    .into_iter()
                    .map(|e| e.message.into())
                    .collect(),
                warnings: result
                    .warnings
                    .into_iter()
                    .map(|e| e.message.into())
                    .collect(),
                template_hash: template_hash.clone(),
                style_hash: style_hash.clone(),
                script_hash: script_hash.clone(),
                has_scoped,
                styles,
                custom_blocks,
                macro_artifacts: macro_artifacts_to_napi(result.macro_artifacts),
                module_shape,
            })
        }
        Err(e) => Ok(SfcCompileResultNapi {
            code: String::new(),
            map: None,
            css: None,
            errors: vec![e.message.into()],
            warnings: vec![],
            template_hash,
            style_hash,
            script_hash,
            has_scoped,
            styles,
            custom_blocks,
            macro_artifacts: vec![],
            module_shape: None,
        }),
    }
}
