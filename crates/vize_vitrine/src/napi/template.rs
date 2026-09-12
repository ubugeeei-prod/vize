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
    codegen::generate_with_experimental_options,
    lane::transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id,
    options::{
        CodegenExperimentalOptions, CodegenMode, CodegenOptions, ParserOptions, TransformOptions,
    },
    parser::parse_with_options_custom_elements_and_template_syntax,
};
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions,
    compile_vapor_with_custom_elements_template_syntax_and_experimental_options,
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
        filename: opts
            .filename
            .clone()
            .unwrap_or_else(|| "template.vue".to_string())
            .into(),
        component_name: self_component_name(&opts).map(Into::into),
        source_map: opts.source_map.unwrap_or(false),
        ssr: opts.ssr.unwrap_or(false),
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(is_module_mode),
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
    let codegen_experimental_opts = CodegenExperimentalOptions {
        component_name: None,
        self_component: opts.experimental_self_component.unwrap_or(false),
    };
    let result = generate_with_experimental_options(&root, codegen_opts, codegen_experimental_opts);
    let map = result
        .map
        .map(|map| serde_json::from_str(map.as_str()))
        .transpose()
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;

    // Collect helpers
    let helpers: Vec<String> = root.helpers.iter().map(|h| h.name().to_string()).collect();

    // Build AST JSON
    let ast = build_ast_json(&root);

    Ok(CompileResult {
        code: result.code.to_string(),
        preamble: result.preamble.to_string(),
        ast,
        map,
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
    let vapor_experimental_opts = VaporCompilerExperimentalOptions {
        component_name: self_component_name(&opts).map(Into::into),
        self_component: opts.experimental_self_component.unwrap_or(false),
        source_map: opts.source_map.unwrap_or(false),
        source_map_filename: opts.filename.clone().map(Into::into),
    };
    let result = compile_vapor_with_custom_elements_template_syntax_and_experimental_options(
        &allocator,
        &template,
        vapor_opts,
        template_syntax,
        vize_atelier_core::options::CustomElementMatcher::from_patterns(
            crate::types::custom_element_patterns(opts.custom_elements.as_deref()),
        ),
        vapor_experimental_opts,
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
    let map = result
        .map
        .map(|map| serde_json::from_str(map.as_str()))
        .transpose()
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;

    Ok(CompileResult {
        code: result.code.into(),
        preamble: String::new(),
        ast: serde_json::json!({}),
        map,
        helpers: vec![],
        templates: Some(result.templates.iter().map(|s| s.to_string()).collect()),
    })
}

fn self_component_name(opts: &CompilerOptions) -> Option<String> {
    opts.component_name
        .clone()
        .or_else(|| component_name_from_filename(opts.filename.as_deref()))
}

fn component_name_from_filename(filename: Option<&str>) -> Option<String> {
    let filename = filename?;
    let stem = std::path::Path::new(filename).file_stem()?.to_str()?.trim();
    (!stem.is_empty()).then(|| stem.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_source_map_uses_requested_filename() {
        let result = compile(
            "<div>{{ msg }}</div>".to_string(),
            Some(CompilerOptions {
                filename: Some("src/Napi.vue".to_string()),
                source_map: Some(true),
                ..Default::default()
            }),
        )
        .expect("compile should succeed");
        let map = result.map.expect("sourceMap should attach a map");

        assert_eq!(map["file"].as_str(), Some("src/Napi.vue"));
        assert_eq!(map["sources"][0].as_str(), Some("src/Napi.vue"));
    }
}
