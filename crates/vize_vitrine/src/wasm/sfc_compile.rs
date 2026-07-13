//! SFC WASM boundary backed by Atlas descriptor and compile products.

use vize_atelier_sfc::compile_script::typescript::transform_typescript_to_js;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileRequest, SfcCompileSettings,
    SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
};
use vize_atlas::Compilation;
use vize_relief::VueDialectInput;
use wasm_bindgen::prelude::*;

use super::compiler::Compiler;
use super::experimentals::experimental_dom_options;
use super::options::parse_compiler_options;
use super::serde::to_json_js_value;
use super::sfc_types::{
    SfcScriptResult, SfcWasmResult, descriptor_to_wasm, macro_artifact_to_wasm,
};
use crate::{
    artifact_graph::{query_sfc_compile, resolve_vue_version},
    template_syntax::resolve_template_syntax,
};

#[wasm_bindgen]
impl Compiler {
    #[wasm_bindgen(js_name = "compileSfc")]
    pub fn compile_sfc(&self, source: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let opts = parse_compiler_options(&options).options;
        let filename: vize_carton::CompactString = opts
            .filename
            .clone()
            .unwrap_or_else(|| "anonymous.vue".to_string())
            .into();
        let dialect = resolve_vue_version(opts.vue_version.as_deref())
            .map_err(|message| JsValue::from_str(&message))?;
        let vapor = opts.output_mode.as_deref() == Some("vapor");
        let output_is_ts = opts.script_ext.as_deref() == Some("preserve");
        let external_scope_id = opts
            .scope_id
            .as_deref()
            .map(|scope_id| scope_id.strip_prefix("data-v-").unwrap_or(scope_id).into());
        let mut template_compiler_options = experimental_dom_options(&opts);
        template_compiler_options.source_map = opts.source_map.unwrap_or(false);
        let compile_options = SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some(filename.clone()),
                inline_template: opts.mode.as_deref() == Some("function"),
                is_ts: output_is_ts,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename.clone()),
                scoped: false,
                ssr: opts.ssr.unwrap_or(false),
                is_ts: output_is_ts,
                custom_renderer: opts.custom_renderer.unwrap_or(false),
                dialect,
                compiler_options: Some(template_compiler_options),
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
        let syntax = resolve_template_syntax(opts.template_syntax.as_deref())
            .map_err(|message| JsValue::from_str(&message))?;
        let mut compilation = Compilation::new();
        crate::artifact_graph::register_sfc_compile_providers(&mut compilation)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let source_id = compilation
            .add_source(filename.as_str(), source)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let mut settings = SfcCompileSettings::default();
        settings.insert(
            source_id,
            SfcCompileRequest::new(compile_options, syntax)
                .with_runtime_names(
                    opts.runtime_module_name.as_deref().unwrap_or("vue"),
                    opts.runtime_global_name.as_deref().unwrap_or("Vue"),
                )
                .with_inferred_scoped_from_descriptor(),
        );
        settings
            .install(&mut compilation)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        compilation
            .set_input::<VueDialectInput>(dialect)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let artifacts = query_sfc_compile(&compilation.snapshot(), source_id)
            .map_err(|message| JsValue::from_str(&message))?;
        let descriptor = artifacts
            .descriptor_artifact()
            .as_result()
            .map_err(|error| JsValue::from_str(&error.message))?;
        let result = artifacts.compiled().map_err(JsValue::from_str)?;
        let source_is_ts = descriptor
            .script_setup
            .as_ref()
            .or(descriptor.script.as_ref())
            .and_then(|script| script.lang.as_deref())
            .is_some_and(|lang| matches!(lang, "ts" | "tsx"));
        let mut template_result = artifacts
            .render()
            .map_err(JsValue::from_str)?
            .map(|render| crate::CompileResult {
                code: render.code().to_string(),
                preamble: String::new(),
                ast: serde_json::json!({}),
                map: result.map.clone(),
                helpers: vec![],
                templates: render
                    .templates()
                    .map(|templates| templates.iter().map(ToString::to_string).collect()),
            });
        if source_is_ts
            && !output_is_ts
            && let Some(template) = template_result.as_mut()
        {
            template.code = transform_typescript_to_js(&template.code).to_string();
        }
        let script_code = if source_is_ts && !output_is_ts {
            transform_typescript_to_js(&result.code).to_string()
        } else {
            result.code.to_string()
        };
        let binding_metadata = result
            .bindings
            .as_ref()
            .and_then(|bindings| serde_json::to_value(bindings).ok())
            .and_then(|bindings| bindings.get("bindings").cloned());
        to_json_js_value(&SfcWasmResult {
            descriptor: descriptor_to_wasm(descriptor),
            template: template_result,
            script: SfcScriptResult {
                code: script_code,
                bindings: result
                    .bindings
                    .clone()
                    .map(|bindings| serde_json::to_value(bindings).unwrap_or_default()),
            },
            css: result.css.as_ref().map(ToString::to_string),
            errors: result
                .errors
                .iter()
                .map(|error| error.message.to_string())
                .collect(),
            warnings: result
                .warnings
                .iter()
                .map(|warning| warning.message.to_string())
                .collect(),
            binding_metadata,
            macro_artifacts: result
                .macro_artifacts
                .iter()
                .map(macro_artifact_to_wasm)
                .collect(),
        })
    }
}
