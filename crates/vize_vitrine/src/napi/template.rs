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

use crate::{
    CompileResult, CompilerOptions,
    template_artifact::{TemplateHostDefaults, compile_template_product},
};
use vize_armature::parse_with_options;
use vize_relief::{ExpressionNode, ParserOptions, RootNode, TemplateChildNode};

/// Compile Vue template to VDom render function
#[napi]
pub fn compile(template: String, options: Option<CompilerOptions>) -> Result<CompileResult> {
    compile_product(template.as_str(), &options.unwrap_or_default(), false)
}

/// Compile Vue template to Vapor mode
#[napi(js_name = "compileVapor")]
pub fn compile_vapor(template: String, options: Option<CompilerOptions>) -> Result<CompileResult> {
    compile_product(template.as_str(), &options.unwrap_or_default(), true)
}

fn compile_product(
    template: &str,
    options: &CompilerOptions,
    vapor: bool,
) -> Result<CompileResult> {
    let artifact =
        compile_template_product(template, options, vapor, None, TemplateHostDefaults::Napi)
            .map_err(|message| Error::new(Status::GenericFailure, message))?;
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

/// Parse template to AST only
#[napi]
pub fn parse_template(
    template: String,
    options: Option<CompilerOptions>,
) -> Result<serde_json::Value> {
    let allocator = Bump::new();
    let opts = options.unwrap_or_default();

    let (root, errors) = parse_with_options(
        &allocator,
        &template,
        ParserOptions {
            experimental_in_tag_comments: opts.experimental_in_tag_comments.unwrap_or(false),
            ..Default::default()
        },
    );

    if !errors.is_empty() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("Parse errors: {:?}", errors),
        ));
    }

    Ok(build_ast_json(&root))
}

/// Build AST JSON from root node.
fn build_ast_json(root: &RootNode<'_>) -> serde_json::Value {
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
                    ExpressionNode::Simple(exp) => exp.content.as_str(),
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
            "content": comment.content.as_str(),
        })).collect::<Vec<_>>(),
        "helpers": root.helpers.iter().map(|h| h.name()).collect::<Vec<_>>(),
        "components": root.components.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "directives": root.directives.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
    })
}
