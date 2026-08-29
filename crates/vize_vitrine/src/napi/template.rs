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
use vize_s0::Allocator;

use crate::{CompileResult, CompilerOptions, template_syntax::resolve_template_syntax};
use vize_atelier_core::{
    codegen::generate,
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::{CodegenMode, CodegenOptions, ParserOptions, TransformOptions},
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_atelier_vapor::{
    VaporCompilerOptions, compile_vapor_with_custom_elements_and_template_syntax,
};

/// Compile Vue template to VDom render function
#[napi]
pub fn compile(template: String, options: Option<CompilerOptions>) -> Result<CompileResult> {
    let opts = options.unwrap_or_default();
    let allocator = Allocator::new();
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())
        .map_err(|message| Error::new(Status::InvalidArg, message))?;

    // Parse
    let custom_element_patterns =
        crate::types::custom_element_patterns(opts.custom_elements.as_deref());
    let custom_elements =
        vize_atelier_core::options::CustomElementMatcher::from_patterns(custom_element_patterns);
    let parser_opts = ParserOptions {
        custom_renderer: opts.custom_renderer.unwrap_or(false),
        experimental_in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
        ..Default::default()
    };
    let (mut root, errors) = parse_with_options_custom_elements_and_template_syntax(
        &allocator,
        &template,
        parser_opts,
        custom_elements.clone(),
        template_syntax,
    );

    let fatal: Vec<_> = errors.iter().filter(|e| !e.is_recoverable()).collect();
    if !fatal.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Parse errors: {:?}", fatal),
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
        custom_renderer: opts.custom_renderer.unwrap_or(false),
        experimental_patterned_template: opts.experimental_patterned_template.unwrap_or(false),
        ..Default::default()
    };
    transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(
        &allocator,
        &mut root,
        transform_opts,
        None,
        custom_elements,
        template_syntax.is_quirks(),
        None,
    );

    // Codegen
    let codegen_opts = CodegenOptions {
        mode: if is_module_mode {
            CodegenMode::Module
        } else {
            CodegenMode::Function
        },
        source_map: opts.source_map.unwrap_or(false),
        ssr: opts.ssr.unwrap_or(false),
        runtime_module_name: opts
            .runtime_module_name
            .clone()
            .unwrap_or_else(|| "vue".to_string())
            .into(),
        runtime_global_name: opts
            .runtime_global_name
            .clone()
            .unwrap_or_else(|| "Vue".to_string())
            .into(),
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
    let allocator = Allocator::new();
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())
        .map_err(|message| Error::new(Status::InvalidArg, message))?;

    // Use actual Vapor compiler
    let vapor_opts = VaporCompilerOptions {
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(false),
        ssr: opts.ssr.unwrap_or(false),
        custom_renderer: opts.custom_renderer.unwrap_or(false),
        experimental_in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
        experimental_patterned_template: opts.experimental_patterned_template.unwrap_or(false),
        ..Default::default()
    };
    let result = compile_vapor_with_custom_elements_and_template_syntax(
        &allocator,
        &template,
        vapor_opts,
        template_syntax,
        vize_atelier_core::options::CustomElementMatcher::from_patterns(
            crate::types::custom_element_patterns(opts.custom_elements.as_deref()),
        ),
    );

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
    options: Option<CompilerOptions>,
) -> Result<serde_json::Value> {
    let allocator = Allocator::new();
    let opts = options.unwrap_or_default();
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())
        .map_err(|message| Error::new(Status::InvalidArg, message))?;

    let (root, errors) = parse_with_options_custom_elements_and_template_syntax(
        &allocator,
        &template,
        ParserOptions {
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            experimental_in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
            ..Default::default()
        },
        vize_atelier_core::options::CustomElementMatcher::from_patterns(
            crate::types::custom_element_patterns(opts.custom_elements.as_deref()),
        ),
        template_syntax,
    );

    if !errors.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            crate::parse_errors::message(&errors, &template),
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
                "tag": el.tag,
                "tagType": format!("{:?}", el.tag_type),
                "props": el.props.len(),
                "children": el.children.len(),
                "isSelfClosing": el.is_self_closing,
            }),
            TemplateChildNode::Text(text) => serde_json::json!({
                "type": "TEXT",
                "content": text.content,
            }),
            TemplateChildNode::Comment(comment) => serde_json::json!({
                "type": "COMMENT",
                "content": comment.content,
            }),
            TemplateChildNode::Interpolation(interp) => serde_json::json!({
                "type": "INTERPOLATION",
                "content": match &interp.content {
                    vize_atelier_core::ExpressionNode::Simple(exp) => exp.content,
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
        "comments": root.comments.iter().map(|comment| serde_json::json!({
            "type": "COMMENT",
            "kind": format!("{:?}", comment.kind),
            "content": comment.content,
        })).collect::<Vec<_>>(),
        "helpers": root.helpers.iter().map(|h| h.name()).collect::<Vec<_>>(),
        "components": root.components.iter().copied().collect::<Vec<_>>(),
        "directives": root.directives.iter().copied().collect::<Vec<_>>(),
    })
}
