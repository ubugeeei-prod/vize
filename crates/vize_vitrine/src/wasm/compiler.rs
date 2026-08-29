//! The `Compiler` WASM class, its free-function aliases, and the internal
//! template/SFC compilation pipeline.

mod free_fns;
pub(in crate::wasm) mod pipeline;

pub use free_fns::*;

use vize_s0::Allocator;
use wasm_bindgen::prelude::*;

use crate::{CompilerOptions, template_syntax::resolve_template_syntax};
use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher};
use vize_atelier_core::parser::parse_with_options_custom_elements_and_template_syntax;
use vize_atelier_sfc::compile_script::typescript::transform_typescript_to_js;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    StyleCompileOptions, TemplateCompileOptions,
    compile_sfc_for_adapter as sfc_compile_for_adapter, parse_sfc,
};

use super::ast::build_ast_json;
use super::experimentals::{compiler_parser_options, experimental_dom_options};
use super::options::{parse_compiler_options, parse_css_options};
use super::serde::{to_js_value, to_json_js_value};
use super::sfc_types::{
    SfcScriptResult, SfcWasmResult, descriptor_to_wasm, macro_artifact_to_wasm,
};
use pipeline::compile_internal;

fn compiler_codegen_options(opts: &CompilerOptions, default_filename: &str) -> CodegenOptions {
    let mut codegen_options = CodegenOptions {
        filename: opts.filename.as_deref().unwrap_or(default_filename).into(),
        ..CodegenOptions::default()
    };
    if let Some(runtime_module_name) = opts.runtime_module_name.as_deref() {
        codegen_options.runtime_module_name = runtime_module_name.into();
    }
    if let Some(runtime_global_name) = opts.runtime_global_name.as_deref() {
        codegen_options.runtime_global_name = runtime_global_name.into();
    }
    codegen_options
}

/// WASM Compiler instance
#[wasm_bindgen]
pub struct Compiler;

#[wasm_bindgen]
impl Compiler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Compiler
    }

    /// Compile template to VDom render function
    #[wasm_bindgen]
    pub fn compile(&self, template: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);

        match compile_internal(template, &parsed.options, false, parsed.binding_metadata) {
            Ok(result) => to_json_js_value(&result),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Compile template to Vapor mode
    #[wasm_bindgen(js_name = "compileVapor")]
    pub fn compile_vapor(&self, template: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);

        match compile_internal(template, &parsed.options, true, None) {
            Ok(result) => to_json_js_value(&result),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Parse template to AST
    #[wasm_bindgen]
    pub fn parse(&self, template: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);
        let allocator = Allocator::new();
        let template_syntax = resolve_template_syntax(parsed.options.template_syntax.as_deref())
            .map_err(|message| JsValue::from_str(&message))?;

        let (root, errors) = parse_with_options_custom_elements_and_template_syntax(
            &allocator,
            template,
            compiler_parser_options(&parsed.options),
            CustomElementMatcher::from_patterns(crate::types::custom_element_patterns(
                parsed.options.custom_elements.as_deref(),
            )),
            template_syntax,
        );

        if !errors.is_empty() {
            return Err(crate::parse_errors::message(&errors, template).into());
        }

        let ast = build_ast_json(&root);
        to_js_value(&ast)
    }

    /// Parse SFC (.vue file)
    #[wasm_bindgen(js_name = "parseSfc")]
    pub fn parse_sfc_method(&self, source: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let filename: vize_s0::CompactString =
            js_sys::Reflect::get(&options, &JsValue::from_str("filename"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "anonymous.vue".to_string())
                .into();

        let opts = SfcParseOptions {
            filename,
            ..Default::default()
        };

        match parse_sfc(source, opts) {
            Ok(descriptor) => to_json_js_value(&descriptor_to_wasm(&descriptor)),
            Err(e) => Err(JsValue::from_str(&e.message)),
        }
    }

    /// Parse CSS into a serialized LightningCSS AST.
    #[wasm_bindgen(js_name = "parseCssAst")]
    pub fn parse_css_ast_method(&self, css: &str, options: JsValue) -> Result<JsValue, JsValue> {
        use vize_atelier_sfc::parse_css_ast;
        let opts = parse_css_options(options);
        let result = parse_css_ast(css, &opts);
        to_js_value(&result)
    }

    /// Print CSS from a serialized LightningCSS AST.
    #[wasm_bindgen(js_name = "printCssAst")]
    pub fn print_css_ast_method(&self, ast: JsValue, options: JsValue) -> Result<JsValue, JsValue> {
        use vize_atelier_sfc::print_css_ast;
        let ast = serde_wasm_bindgen::from_value(ast)
            .map_err(|e| JsValue::from_str(&format!("Invalid CSS AST: {e}")))?;
        let opts = parse_css_options(options);
        let result = print_css_ast(ast, &opts);
        to_js_value(&result)
    }

    /// Compile CSS with LightningCSS
    #[wasm_bindgen(js_name = "compileCss")]
    pub fn compile_css_method(&self, css: &str, options: JsValue) -> Result<JsValue, JsValue> {
        use vize_atelier_sfc::compile_css;
        let opts = parse_css_options(options);
        let result = compile_css(css, &opts);
        to_js_value(&result)
    }

    /// Compile SFC template block
    #[wasm_bindgen(js_name = "compileSfc")]
    pub fn compile_sfc(&self, source: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);
        let opts = parsed.options;

        let filename: vize_s0::CompactString = opts
            .filename
            .clone()
            .unwrap_or_else(|| "anonymous.vue".to_string())
            .into();

        let parse_opts = SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        };

        let descriptor = match parse_sfc(source, parse_opts) {
            Ok(d) => d,
            Err(e) => return Err(JsValue::from_str(&e.message)),
        };

        let has_vapor_attr = descriptor
            .script_setup
            .as_ref()
            .map(|s| s.attrs.contains_key("vapor"))
            .unwrap_or(false)
            || descriptor
                .script
                .as_ref()
                .map(|s| s.attrs.contains_key("vapor"))
                .unwrap_or(false);
        let use_vapor = has_vapor_attr || opts.output_mode.as_deref() == Some("vapor");

        let source_is_ts = descriptor
            .script_setup
            .as_ref()
            .and_then(|s| s.lang.as_ref())
            .map(|l| l == "ts" || l == "tsx")
            .unwrap_or(false)
            || descriptor
                .script
                .as_ref()
                .and_then(|s| s.lang.as_ref())
                .map(|l| l == "ts" || l == "tsx")
                .unwrap_or(false);

        let output_is_ts = opts
            .script_ext
            .as_deref()
            .map(|ext| ext == "preserve")
            .unwrap_or(false);

        let mut opts = opts;
        // Keep the template result's source-map identity aligned with the SFC
        // descriptor even when the caller relies on the facade default.
        if opts.filename.is_none() {
            opts.filename = Some(filename.to_string());
        }
        if source_is_ts {
            opts.is_ts = Some(true);
        }

        let mut template_result = if let Some(template) = &descriptor.template {
            match compile_internal(&template.content, &opts, use_vapor, None) {
                Ok(r) => Some(r),
                Err(e) => return Err(JsValue::from_str(&e)),
            }
        } else {
            None
        };

        let standalone = opts.mode.as_deref() == Some("function");
        let codegen_options = compiler_codegen_options(&opts, filename.as_str());
        let sfc_opts = SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            },
            script: ScriptCompileOptions {
                id: Some(filename.clone()),
                inline_template: standalone,
                is_ts: output_is_ts,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                id: Some(filename.clone()),
                scoped: descriptor.styles.iter().any(|s| s.scoped),
                ssr: opts.ssr.unwrap_or(false),
                is_ts: output_is_ts,
                custom_renderer: opts.custom_renderer.unwrap_or(false),
                compiler_options: Some(experimental_dom_options(&opts)),
                ..Default::default()
            },
            style: StyleCompileOptions {
                id: filename,
                scoped: descriptor.styles.iter().any(|s| s.scoped),
                ..Default::default()
            },
            vapor: use_vapor,
            scope_id: None,
        };

        let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())
            .map_err(|message| JsValue::from_str(&message))?;

        let compile_result = sfc_compile_for_adapter(
            &descriptor,
            sfc_opts,
            template_syntax,
            CustomElementMatcher::from_patterns(crate::types::custom_element_patterns(
                opts.custom_elements.as_deref(),
            )),
            codegen_options,
            if standalone {
                SfcScriptOutputMode::InlineTemplate
            } else {
                SfcScriptOutputMode::SeparateTemplate
            },
        );
        let sfc_result = match compile_result {
            Ok(r) => r,
            Err(e) => return Err(JsValue::from_str(&e.message)),
        };

        let script_code = if source_is_ts && !output_is_ts {
            transform_typescript_to_js(&sfc_result.code).to_string()
        } else {
            sfc_result.code.to_string()
        };

        if source_is_ts
            && !output_is_ts
            && let Some(template_result) = template_result.as_mut()
        {
            template_result.code = transform_typescript_to_js(&template_result.code).to_string();
        }

        let binding_metadata = sfc_result
            .bindings
            .as_ref()
            .and_then(|b| serde_json::to_value(&b.bindings).ok());
        let macro_artifacts = sfc_result
            .macro_artifacts
            .iter()
            .map(macro_artifact_to_wasm)
            .collect();

        let result = SfcWasmResult {
            descriptor: descriptor_to_wasm(&descriptor),
            template: template_result,
            script: SfcScriptResult {
                code: script_code,
                bindings: sfc_result
                    .bindings
                    .map(|b| serde_json::to_value(&b).unwrap_or_default()),
            },
            css: sfc_result.css.map(Into::into),
            errors: sfc_result
                .errors
                .into_iter()
                .map(|e| e.message.into())
                .collect(),
            warnings: sfc_result
                .warnings
                .into_iter()
                .map(|e| e.message.into())
                .collect(),
            binding_metadata,
            macro_artifacts,
        };

        to_json_js_value(&result)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
