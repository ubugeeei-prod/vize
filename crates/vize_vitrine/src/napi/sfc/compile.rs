use napi::{Result, Status};
use napi_derive::napi;
use vize_carton::cstr;

use super::{
    experimentals::ExperimentalTemplateOptions,
    types::{
        SfcCompileOptionsNapi, SfcCompileResultNapi, custom_blocks_to_napi,
        macro_artifacts_to_napi, style_blocks_to_napi,
    },
};
use crate::{
    artifact_graph::{query_sfc_compile, resolve_vue_version},
    template_syntax::resolve_template_syntax,
};

#[napi(js_name = "compileSfc")]
pub fn compile_sfc(
    source: String,
    options: Option<SfcCompileOptionsNapi>,
) -> Result<SfcCompileResultNapi> {
    compile_sfc_impl(source, options)
        .map_err(|message| napi::Error::new(Status::InvalidArg, message))
}

fn compile_sfc_impl(
    source: String,
    options: Option<SfcCompileOptionsNapi>,
) -> std::result::Result<SfcCompileResultNapi, String> {
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcCompileRequest, SfcCompileSettings,
        SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
    };
    use vize_atlas::Compilation;
    use vize_relief::VueDialectInput;

    let opts = options.unwrap_or_default();
    let filename: vize_carton::CompactString =
        opts.filename.as_deref().unwrap_or("anonymous.vue").into();
    let dialect = resolve_vue_version(opts.vue_version.as_deref())?;
    let vapor = opts.vapor.unwrap_or(false);
    let is_ts = opts.is_ts.unwrap_or(false);
    let source_map = opts.source_map.unwrap_or(false);
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())?;
    let experimentals = ExperimentalTemplateOptions::from_compile(&opts);
    let standalone = opts.mode.as_deref() == Some("function");
    let external_scope_id: Option<vize_carton::CompactString> = opts
        .scope_id
        .as_ref()
        .map(|sid| sid.strip_prefix("data-v-").unwrap_or(sid).into());
    let template_compiler_options = {
        let scope_id = external_scope_id
            .as_ref()
            .map(|scope_id| cstr!("data-v-{scope_id}"));
        Some(vize_atelier_dom::DomCompilerOptions {
            scope_id,
            source_map,
            ..experimentals.dom_options()
        })
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
            scoped: false,
            ssr: opts.ssr.unwrap_or(false),
            is_ts,
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            dialect,
            compiler_options: template_compiler_options,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.clone(),
            scoped: false,
            ..Default::default()
        },
        vapor,
        scope_id: external_scope_id,
    };

    let mut compilation = Compilation::new();
    vize_atelier_sfc::register_atlas_providers(&mut compilation)
        .map_err(|error| error.to_string())?;
    let source_id = compilation
        .add_source(filename.as_str(), source)
        .map_err(|error| error.to_string())?;
    let mut settings = SfcCompileSettings::default();
    settings.insert(
        source_id,
        SfcCompileRequest::new(compile_opts, template_syntax)
            .with_inferred_scoped_from_descriptor(),
    );
    settings
        .install(&mut compilation)
        .map_err(|error| error.to_string())?;
    compilation
        .set_input::<VueDialectInput>(dialect)
        .map_err(|error| error.to_string())?;
    let artifacts = query_sfc_compile(&compilation.snapshot(), source_id)?;
    let descriptor = match artifacts.descriptor_artifact().as_result() {
        Ok(descriptor) => descriptor,
        Err(error) => return Ok(failed_result(error.message.as_str())),
    };
    let template_hash: Option<String> = descriptor.template_hash().map(Into::into);
    let style_hash: Option<String> = descriptor.style_hash().map(Into::into);
    let script_hash: Option<String> = descriptor.script_hash().map(Into::into);
    let styles = style_blocks_to_napi(&descriptor.styles);
    let custom_blocks = custom_blocks_to_napi(&descriptor.custom_blocks);
    let has_scoped = descriptor.styles.iter().any(|style| style.scoped);

    match artifacts.compiled() {
        Ok(result) => Ok(SfcCompileResultNapi {
            code: result.code.to_string(),
            css: result.css.as_ref().map(ToString::to_string),
            map: result.map.as_ref().map(ToString::to_string),
            errors: result
                .errors
                .iter()
                .map(|e| e.message.to_string())
                .collect(),
            warnings: result
                .warnings
                .iter()
                .map(|e| e.message.to_string())
                .collect(),
            template_hash: template_hash.clone(),
            style_hash: style_hash.clone(),
            script_hash: script_hash.clone(),
            has_scoped,
            styles,
            custom_blocks,
            macro_artifacts: macro_artifacts_to_napi(result.macro_artifacts.clone()),
        }),
        Err(error) => Ok(SfcCompileResultNapi {
            code: String::new(),
            css: None,
            map: None,
            errors: vec![error.to_string()],
            warnings: vec![],
            template_hash,
            style_hash,
            script_hash,
            has_scoped,
            styles,
            custom_blocks,
            macro_artifacts: vec![],
        }),
    }
}

fn failed_result(error: &str) -> SfcCompileResultNapi {
    SfcCompileResultNapi {
        code: String::new(),
        css: None,
        map: None,
        errors: vec![error.to_string()],
        warnings: vec![],
        template_hash: None,
        style_hash: None,
        script_hash: None,
        has_scoped: false,
        styles: vec![],
        custom_blocks: vec![],
        macro_artifacts: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sfc_compile_result_surfaces_template_source_map_when_requested() {
        let source = "<template><p>{{ msg }}</p></template>\n";
        let base_options = SfcCompileOptionsNapi {
            filename: Some("template.vue".to_string()),
            ..Default::default()
        };

        let without = compile_sfc_impl(source.to_string(), Some(base_options)).expect("compile");
        assert!(without.errors.is_empty(), "errors: {:?}", without.errors);
        assert!(without.map.is_none(), "no map unless requested");

        let with = compile_sfc_impl(
            source.to_string(),
            Some(SfcCompileOptionsNapi {
                filename: Some("template.vue".to_string()),
                source_map: Some(true),
                ..Default::default()
            }),
        )
        .expect("compile");

        assert!(with.errors.is_empty(), "errors: {:?}", with.errors);
        let map = with.map.expect("a map is surfaced when requested");
        assert!(map.contains("\"version\":3"), "v3 source map: {map}");
        assert!(
            map.contains("template.vue"),
            "map carries the source filename: {map}"
        );
    }

    #[test]
    fn napi_vue_version_reaches_each_supported_compile_plan() {
        for version in ["1", "2", "2.7", "3"] {
            let result = compile_sfc_impl(
                "<template><p>{{ msg }}</p></template>".to_string(),
                Some(SfcCompileOptionsNapi {
                    filename: Some(format!("vue-{version}.vue")),
                    vue_version: Some(version.to_string()),
                    ..Default::default()
                }),
            )
            .unwrap();
            assert!(
                result.errors.is_empty(),
                "vueVersion={version}: {:?}",
                result.errors
            );
            assert!(!result.code.is_empty(), "vueVersion={version}");
        }
    }

    #[test]
    fn napi_vue_version_rejects_ambiguous_values() {
        let result = compile_sfc_impl(
            "<template />".to_string(),
            Some(SfcCompileOptionsNapi {
                vue_version: Some("0".to_string()),
                ..Default::default()
            }),
        );
        let Err(error) = result else {
            panic!("ambiguous vueVersion must fail")
        };
        assert!(error.contains("invalid vueVersion"), "{error}");
    }
}
