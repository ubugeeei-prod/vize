//! The `Compiler` WASM class, its free-function aliases, and the internal
//! template/SFC compilation pipeline.

use vize_carton::Bump;
use wasm_bindgen::prelude::*;

use crate::{CompileResult, CompilerOptions, template_syntax::resolve_template_syntax};
use vize_armature::parse_with_options;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_template_syntax};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor_with_template_syntax};
use vize_relief::{
    CodegenMode, ExpressionNode, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode,
};

use super::experimentals::{
    experimental_dom_options, experimental_flags, experimental_parser_options,
};
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
        let allocator = Bump::new();

        let (root, errors) = parse_with_options(
            &allocator,
            template,
            experimental_parser_options(&parsed.options),
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

/// Internal compile function
pub(super) fn compile_internal(
    template: &str,
    opts: &CompilerOptions,
    vapor: bool,
    binding_metadata: Option<vize_carton::BindingMetadata>,
) -> Result<CompileResult, String> {
    let allocator = Bump::new();
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())?;
    let (experimental_in_tag_comments, experimental_patterned_template) = experimental_flags(opts);
    let binding_metadata = backend_binding_metadata(binding_metadata);

    if opts.ssr.unwrap_or(false) && !vapor && binding_metadata.is_none() {
        let ssr_opts = SsrCompilerOptions {
            is_ts: opts.is_ts.unwrap_or(false),
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            experimental_in_tag_comments,
            experimental_patterned_template,
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
            experimental_in_tag_comments,
            experimental_patterned_template,
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
        ..experimental_dom_options(opts)
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

/// Lossless compatibility projection for standalone low-level template APIs.
#[allow(deprecated)]
fn backend_binding_metadata(
    metadata: Option<vize_carton::BindingMetadata>,
) -> Option<vize_relief::BindingMetadata> {
    metadata.map(Into::into)
}

/// Build AST JSON from root node
fn build_ast_json(root: &RootNode<'_>) -> serde_json::Value {
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
                        PropNode::Attribute(attr) => serde_json::json!({
                            "type": "ATTRIBUTE",
                            "name": attr.name.as_str(),
                            "value": attr.value.as_ref().map(|v| v.content.as_str()),
                        }),
                        PropNode::Directive(dir) => serde_json::json!({
                            "type": "DIRECTIVE",
                            "name": dir.name.as_str(),
                            "arg": dir.arg.as_ref().map(|a| match a {
                                ExpressionNode::Simple(exp) => exp.content.as_str().to_string(),
                                _ => "<compound>".to_string(),
                            }),
                            "exp": dir.exp.as_ref().map(|e| match e {
                                ExpressionNode::Simple(exp) => exp.content.as_str().to_string(),
                                _ => "<compound>".to_string(),
                            }),
                            "modifiers": dir.modifiers.iter().map(|m: &SimpleExpressionNode| m.content.as_str()).collect::<Vec<_>>(),
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
                    ExpressionNode::Simple(exp) => exp.content.as_str(),
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
