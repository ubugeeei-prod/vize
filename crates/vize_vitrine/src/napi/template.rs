//! NAPI bindings for Vue template compilation.
//!
//! Provides compile, compileVapor, and parseTemplate functions
//! for direct template-to-render-function compilation.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use vize_carton::Bump;

use crate::{CompileResult, CompilerOptions};
use vize_atelier_core::{
    codegen::generate,
    options::{CodegenMode, CodegenOptions, TransformOptions},
    parser::parse,
    transform::transform,
};
use vize_atelier_vapor::{compile_vapor as vapor_compile, VaporCompilerOptions};

/// Compile Vue template to VDom render function
#[napi]
pub fn compile(template: String, options: Option<CompilerOptions>) -> Result<CompileResult> {
    let opts = options.unwrap_or_default();
    let allocator = Bump::new();

    // Parse
    let (mut root, errors) = parse(&allocator, &template);

    if !errors.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Parse errors: {:?}", errors),
        ));
    }

    // Determine mode
    let is_module_mode = opts.mode.as_deref() == Some("module");

    // Transform
    // In module mode, prefix_identifiers defaults to true (like Vue)
    let transform_opts = TransformOptions {
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(is_module_mode),
        hoist_static: opts.hoist_static.unwrap_or(false),
        cache_handlers: opts.cache_handlers.unwrap_or(false),
        scope_id: opts.scope_id.clone().map(|s| s.into()),
        ssr: opts.ssr.unwrap_or(false),
        ..Default::default()
    };
    transform(&allocator, &mut root, transform_opts, None);

    // Codegen
    let codegen_opts = CodegenOptions {
        mode: if is_module_mode {
            CodegenMode::Module
        } else {
            CodegenMode::Function
        },
        source_map: opts.source_map.unwrap_or(false),
        ssr: opts.ssr.unwrap_or(false),
        ..Default::default()
    };
    let result = generate(&root, codegen_opts);

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

/// Compile Vue template to Vapor mode
#[napi(js_name = "compileVapor")]
pub fn compile_vapor(template: String, options: Option<CompilerOptions>) -> Result<CompileResult> {
    let opts = options.unwrap_or_default();
    let allocator = Bump::new();

    // Use actual Vapor compiler
    let vapor_opts = VaporCompilerOptions {
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(false),
        ssr: opts.ssr.unwrap_or(false),
        ..Default::default()
    };
    let result = vapor_compile(&allocator, &template, vapor_opts);

    if !result.error_messages.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            result
                .error_messages
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }

    Ok(CompileResult {
        code: result.code.into(),
        preamble: String::new(),
        ast: serde_json::json!({}),
        map: None,
        helpers: vec![],
        templates: Some(result.templates.iter().map(|s| s.to_string()).collect()),
    })
}

/// Parse template to AST only
#[napi]
pub fn parse_template(
    template: String,
    _options: Option<CompilerOptions>,
) -> Result<serde_json::Value> {
    let allocator = Bump::new();

    let (root, errors) = parse(&allocator, &template);

    if !errors.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Parse errors: {:?}", errors),
        ));
    }

    Ok(build_ast_json(&root))
}

/// Build AST JSON from root node.
fn build_ast_json(root: &vize_atelier_core::RootNode<'_>) -> serde_json::Value {
    use vize_atelier_core::TemplateChildNode;

    let children: Vec<serde_json::Value> = root
        .children
        .iter()
        .map(|child| match child {
            TemplateChildNode::Element(el) => serde_json::json!({
                "type": "ELEMENT",
                "tag": el.tag.as_str(),
                "tagType": format!("{:?}", el.tag_type),
                "props": el.props.len(),
                "children": el.children.len(),
                "isSelfClosing": el.is_self_closing,
            }),
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
        })
        .collect();

    serde_json::json!({
        "type": "ROOT",
        "children": children,
        "helpers": root.helpers.iter().map(|h| h.name()).collect::<Vec<_>>(),
        "components": root.components.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "directives": root.directives.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
    })
}
