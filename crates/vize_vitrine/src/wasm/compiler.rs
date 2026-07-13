//! The `Compiler` WASM class, its free-function aliases, and the internal
//! template/SFC compilation pipeline.

use vize_carton::Bump;
use wasm_bindgen::prelude::*;

use crate::{
    CompileResult, CompilerOptions,
    template_artifact::{TemplateHostDefaults, compile_template_product},
    template_syntax::resolve_template_syntax,
};
use vize_armature::parse_with_options_and_template_syntax;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

use super::ast::build_ast_json;
use super::experimentals::compiler_parser_options;
use super::options::{parse_compiler_options, parse_css_options};
use super::serde::{to_js_value, to_json_js_value};
use super::sfc_types::descriptor_to_wasm;

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

        match compile_template_query(template, &parsed.options, false, parsed.binding_metadata) {
            Ok(result) => to_json_js_value(&result),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Compile template to Vapor mode
    #[wasm_bindgen(js_name = "compileVapor")]
    pub fn compile_vapor(&self, template: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);

        match compile_template_query(template, &parsed.options, true, None) {
            Ok(result) => to_json_js_value(&result),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Parse template to AST
    #[wasm_bindgen]
    pub fn parse(&self, template: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);
        let allocator = Bump::new();
        let template_syntax = resolve_template_syntax(parsed.options.template_syntax.as_deref())
            .map_err(|message| JsValue::from_str(&message))?;

        let (root, errors) = parse_with_options_and_template_syntax(
            &allocator,
            template,
            compiler_parser_options(&parsed.options),
            template_syntax,
        );

        if !errors.is_empty() {
            return Err(JsValue::from_str(&format!("Parse errors: {:?}", errors)));
        }

        let ast = build_ast_json(&root);
        to_js_value(&ast)
    }

    /// Parse SFC (.vue file)
    #[wasm_bindgen(js_name = "parseSfc")]
    pub fn parse_sfc_method(&self, source: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let filename: vize_carton::CompactString =
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
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Query the complete raw-template product and project it to the WASM ABI.
pub(super) fn compile_template_query(
    template: &str,
    opts: &CompilerOptions,
    vapor: bool,
    binding_metadata: Option<vize_carton::BindingMetadata>,
) -> Result<CompileResult, String> {
    let artifact = compile_template_product(
        template,
        opts,
        vapor,
        binding_metadata,
        TemplateHostDefaults::Wasm,
    )?;
    if vapor {
        return Ok(CompileResult {
            code: artifact.code.to_string(),
            preamble: artifact.preamble.to_string(),
            ast: serde_json::json!({}),
            map: artifact.map.clone(),
            helpers: vec![],
            templates: artifact
                .templates
                .as_ref()
                .map(|templates| templates.iter().map(ToString::to_string).collect()),
        });
    }
    let allocator = Bump::new();
    let root = artifact.syntax.materialize(&allocator);
    Ok(CompileResult {
        code: artifact.code.to_string(),
        preamble: artifact.preamble.to_string(),
        ast: build_ast_json(&root),
        map: artifact.map.clone(),
        helpers: root
            .helpers
            .iter()
            .map(|helper| helper.name().to_string())
            .collect(),
        templates: None,
    })
}
/// Compile template to VDom (free function)
#[wasm_bindgen]
pub fn compile(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile(template, options)
}

/// Compile template to Vapor mode (free function)
#[wasm_bindgen(js_name = "compileVapor")]
pub fn compile_vapor_fn(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_vapor(template, options)
}

/// Parse template to AST (free function)
#[wasm_bindgen(js_name = "parseTemplate")]
pub fn parse_template(template: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().parse(template, options)
}

/// Parse SFC (free function)
#[wasm_bindgen(js_name = "parseSfc")]
pub fn parse_sfc_fn(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().parse_sfc_method(source, options)
}

/// Compile SFC (free function)
#[wasm_bindgen(js_name = "compileSfc")]
pub fn compile_sfc_fn(source: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_sfc(source, options)
}

/// Parse CSS to AST (free function)
#[wasm_bindgen(js_name = "parseCssAst")]
pub fn parse_css_ast_fn(css: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().parse_css_ast_method(css, options)
}

/// Print CSS from AST (free function)
#[wasm_bindgen(js_name = "printCssAst")]
pub fn print_css_ast_fn(ast: JsValue, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().print_css_ast_method(ast, options)
}

/// Compile CSS (free function)
#[wasm_bindgen(js_name = "compileCss")]
pub fn compile_css_fn(css: &str, options: JsValue) -> Result<JsValue, JsValue> {
    Compiler::new().compile_css_method(css, options)
}
