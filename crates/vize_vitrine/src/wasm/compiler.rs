//! The `Compiler` WASM class, its free-function aliases, and the internal
//! template/SFC compilation pipeline.

use vize_carton::Bump;
use wasm_bindgen::prelude::*;

use crate::{CompileResult, CompilerOptions, template_syntax::resolve_template_syntax};
use vize_atelier_core::options::CodegenMode;
use vize_atelier_core::parser::parse;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_atelier_sfc::compile_script::typescript::transform_typescript_to_js;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
    TemplateCompileOptions, compile_sfc_with_template_syntax as sfc_compile_with_template_syntax,
    parse_sfc,
};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_template_syntax};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor_with_template_syntax};

use super::options::{parse_compiler_options, parse_css_options};
use super::serde::{to_js_value, to_json_js_value};
use super::sfc_types::{
    SfcScriptResult, SfcWasmResult, descriptor_to_wasm, macro_artifact_to_wasm,
};

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
    pub fn parse(&self, template: &str, _options: JsValue) -> Result<JsValue, JsValue> {
        let allocator = Bump::new();

        let (root, errors) = parse(&allocator, template);

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

    /// Compile SFC template block
    #[wasm_bindgen(js_name = "compileSfc")]
    pub fn compile_sfc(&self, source: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let parsed = parse_compiler_options(&options);
        let opts = parsed.options;

        let filename: vize_carton::CompactString = opts
            .filename
            .clone()
            .unwrap_or_else(|| "anonymous.vue".to_string())
            .into();

        let parse_opts = SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        };

        // Parse SFC
        let descriptor = match parse_sfc(source, parse_opts) {
            Ok(d) => d,
            Err(e) => return Err(JsValue::from_str(&e.message)),
        };

        // Detect vapor mode from script setup attrs or options
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

        // Detect TypeScript from script lang attribute (for source detection)
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

        // Determine output format: preserve TypeScript or downcompile to JavaScript
        // script_ext option: "preserve" keeps TypeScript, "downcompile" (default) transpiles to JS
        let output_is_ts = opts
            .script_ext
            .as_deref()
            .map(|ext| ext == "preserve")
            .unwrap_or(false); // Default to downcompile (transpile to JS)

        // Update opts with source detection for backwards compatibility
        let mut opts = opts;
        if source_is_ts {
            opts.is_ts = Some(true);
        }

        // Compile template if present
        let mut template_result = if let Some(template) = &descriptor.template {
            match compile_internal(&template.content, &opts, use_vapor, None) {
                Ok(r) => Some(r),
                Err(e) => return Err(JsValue::from_str(&e)),
            }
        } else {
            None
        };

        // Full SFC compilation using sfc_compile
        // Use output_is_ts to control whether TypeScript is preserved or transpiled
        let standalone = opts.mode.as_deref() == Some("function");
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
                compiler_options: Some(DomCompilerOptions::default()),
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

        // Compile the full SFC
        let compile_result =
            sfc_compile_with_template_syntax(&descriptor, sfc_opts, template_syntax);
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

        // Build result with compiled script code
        // Convert descriptor to owned for serialization
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

/// Internal compile function
fn compile_internal(
    template: &str,
    opts: &CompilerOptions,
    vapor: bool,
    binding_metadata: Option<vize_atelier_core::options::BindingMetadata>,
) -> Result<CompileResult, String> {
    let allocator = Bump::new();
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())?;

    // SSR mode - use dedicated SSR compiler
    if opts.ssr.unwrap_or(false) && !vapor && binding_metadata.is_none() {
        let ssr_opts = SsrCompilerOptions {
            is_ts: opts.is_ts.unwrap_or(false),
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            ..Default::default()
        };
        let (root, errors, result) =
            compile_ssr_with_template_syntax(&allocator, template, ssr_opts, template_syntax);

        let fatal: Vec<_> = errors
            .iter()
            .filter(|error| !error.is_recoverable())
            .collect();
        if !fatal.is_empty() {
            return Err(format!("SSR compile errors: {:?}", fatal));
        }

        // Collect helpers
        let helpers: Vec<String> = root.helpers.iter().map(|h| h.name().to_string()).collect();

        // Build AST JSON
        let ast = build_ast_json(&root);

        return Ok(CompileResult {
            code: result.code.to_string(),
            preamble: result.preamble.to_string(),
            ast,
            map: None,
            helpers,
            templates: None,
        });
    }

    if vapor {
        // Use actual Vapor compiler
        let vapor_opts = VaporCompilerOptions {
            prefix_identifiers: opts.prefix_identifiers.unwrap_or(false),
            ssr: opts.ssr.unwrap_or(false),
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            binding_metadata,
            ..Default::default()
        };
        let result =
            compile_vapor_with_template_syntax(&allocator, template, vapor_opts, template_syntax);

        if !result.error_messages.is_empty() {
            return Err(result
                .error_messages
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n"));
        }

        return Ok(CompileResult {
            code: result.code.to_string(),
            preamble: String::new(),
            ast: serde_json::json!({}),
            map: None,
            helpers: vec![],
            templates: Some(
                result
                    .templates
                    .into_iter()
                    .map(|t| t.to_string())
                    .collect(),
            ),
        });
    }

    // VDOM mode - use vize_atelier_dom which includes proper v-model transform
    let has_binding_metadata = binding_metadata.is_some();
    let dom_opts = DomCompilerOptions {
        mode: match opts.mode.as_deref() {
            Some("module") => CodegenMode::Module,
            _ => CodegenMode::Function,
        },
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(has_binding_metadata),
        hoist_static: opts.hoist_static.unwrap_or(has_binding_metadata),
        cache_handlers: opts.cache_handlers.unwrap_or(has_binding_metadata),
        scope_id: opts.scope_id.clone().map(|s| s.into()),
        ssr: opts.ssr.unwrap_or(false),
        source_map: opts.source_map.unwrap_or(false),
        is_ts: opts.is_ts.unwrap_or(false),
        custom_renderer: opts.custom_renderer.unwrap_or(false),
        binding_metadata,
        inline: has_binding_metadata,
        ..Default::default()
    };

    let (root, errors, result) =
        compile_template_with_template_syntax(&allocator, template, dom_opts, template_syntax);

    let fatal: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_recoverable())
        .collect();
    if !fatal.is_empty() {
        return Err(format!("Compile errors: {:?}", fatal));
    }

    // Collect helpers
    let helpers: Vec<String> = root.helpers.iter().map(|h| h.name().to_string()).collect();

    // Build AST JSON
    let ast = build_ast_json(&root);

    Ok(CompileResult {
        code: result.code.to_string(),
        preamble: result.preamble.to_string(),
        ast,
        map: None,
        helpers,
        templates: None,
    })
}

/// Build AST JSON from root node
fn build_ast_json(root: &vize_atelier_core::RootNode<'_>) -> serde_json::Value {
    use vize_atelier_core::TemplateChildNode;

    fn build_children(children: &[TemplateChildNode<'_>]) -> Vec<serde_json::Value> {
        children
            .iter()
            .map(|child| build_child_json(child))
            .collect()
    }

    fn build_child_json(child: &TemplateChildNode<'_>) -> serde_json::Value {
        match child {
            TemplateChildNode::Element(el) => {
                let props: Vec<serde_json::Value> = el
                    .props
                    .iter()
                    .map(|prop| match prop {
                        vize_atelier_core::PropNode::Attribute(attr) => serde_json::json!({
                            "type": "ATTRIBUTE",
                            "name": attr.name.as_str(),
                            "value": attr.value.as_ref().map(|v| v.content.as_str()),
                        }),
                        vize_atelier_core::PropNode::Directive(dir) => serde_json::json!({
                            "type": "DIRECTIVE",
                            "name": dir.name.as_str(),
                            "arg": dir.arg.as_ref().map(|a| match a {
                                vize_atelier_core::ExpressionNode::Simple(exp) => exp.content.as_str().to_string(),
                                _ => "<compound>".to_string(),
                            }),
                            "exp": dir.exp.as_ref().map(|e| match e {
                                vize_atelier_core::ExpressionNode::Simple(exp) => exp.content.as_str().to_string(),
                                _ => "<compound>".to_string(),
                            }),
                            "modifiers": dir.modifiers.iter().map(|m: &vize_atelier_core::SimpleExpressionNode| m.content.as_str()).collect::<Vec<_>>(),
                        }),
                    })
                    .collect();

                serde_json::json!({
                    "type": "ELEMENT",
                    "tag": el.tag.as_str(),
                    "tagType": format!("{:?}", el.tag_type),
                    "props": props,
                    "children": build_children(&el.children),
                    "isSelfClosing": el.is_self_closing,
                })
            }
            TemplateChildNode::Text(text) => serde_json::json!({
                "type": "TEXT",
                "content": text.content.as_str(),
            }),
            TemplateChildNode::Comment(comment) => serde_json::json!({
                "type": "COMMENT",
                "content": comment.content.as_str(),
            }),
            TemplateChildNode::Interpolation(interp) => serde_json::json!({
                "type": "INTERPOLATION",
                "content": match &interp.content {
                    vize_atelier_core::ExpressionNode::Simple(exp) => exp.content.as_str(),
                    _ => "<compound>",
                }
            }),
            _ => serde_json::json!({
                "type": "UNKNOWN"
            }),
        }
    }

    let children = build_children(&root.children);

    serde_json::json!({
        "type": "ROOT",
        "children": children,
        "helpers": root.helpers.iter().map(|h| h.name()).collect::<Vec<_>>(),
        "components": root.components.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "directives": root.directives.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
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
