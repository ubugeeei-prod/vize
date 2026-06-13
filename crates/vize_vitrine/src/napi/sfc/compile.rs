use napi::{Result, Status};
use napi_derive::napi;
use vize_carton::cstr;

use super::types::{
    SfcCompileOptionsNapi, SfcCompileResultNapi, custom_blocks_to_napi, macro_artifacts_to_napi,
    style_blocks_to_napi,
};
use crate::template_syntax::resolve_template_syntax;

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
        ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
        TemplateCompileOptions,
        compile_sfc_with_template_syntax as sfc_compile_with_template_syntax,
        parse_sfc as sfc_parse,
    };

    let opts = options.unwrap_or_default();
    let filename: vize_carton::CompactString = opts
        .filename
        .unwrap_or_else(|| "anonymous.vue".to_string())
        .into();
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor = match sfc_parse(&source, parse_opts) {
        Ok(d) => d,
        Err(e) => {
            return Ok(SfcCompileResultNapi {
                code: String::new(),
                css: None,
                map: None,
                errors: vec![e.message.into()],
                warnings: vec![],
                template_hash: None,
                style_hash: None,
                script_hash: None,
                has_scoped: false,
                styles: vec![],
                custom_blocks: vec![],
                macro_artifacts: vec![],
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
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())?;
    let standalone = opts.mode.as_deref() == Some("function");
    let external_scope_id: Option<vize_carton::CompactString> = opts
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
        Some(vize_atelier_dom::DomCompilerOptions {
            scope_id,
            source_map,
            ..Default::default()
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
            ..Default::default()
        },
        vapor,
        scope_id: external_scope_id,
    };

    let compile_result =
        sfc_compile_with_template_syntax(&descriptor, compile_opts, template_syntax);

    match compile_result {
        Ok(result) => Ok(SfcCompileResultNapi {
            code: result.code.into(),
            css: result.css.map(Into::into),
            map: result.map.map(|map| map.to_string()),
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
        }),
        Err(e) => Ok(SfcCompileResultNapi {
            code: String::new(),
            css: None,
            map: None,
            errors: vec![e.message.into()],
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
}
