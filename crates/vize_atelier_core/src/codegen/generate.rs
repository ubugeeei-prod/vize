//! Hoist generation and JS node serialization.
//!
//! Generates hoisted variable declarations and serializes JS child nodes,
//! VNode calls, props expressions, and children to byte output.

use crate::ast::{
    DynamicProps, ExpressionNode, JsChildNode, PropsExpression, RootNode, RuntimeHelper,
    TemplateTextChildNode, VNodeCall, VNodeChildren, VNodeTag,
};

use super::{context::CodegenContext, helpers::escape_js_string};
use vize_carton::String;
use vize_carton::ToCompactString;

/// Generate hoisted variable declarations.
pub(super) fn generate_hoists(ctx: &CodegenContext, root: &RootNode<'_>) -> String {
    let mut hoists_code: Vec<u8> = Vec::new();

    for (i, hoist) in root.hoists.iter().enumerate() {
        if let Some(node) = hoist {
            hoists_code.extend_from_slice(b"const _hoisted_");
            hoists_code.extend_from_slice((i + 1).to_compact_string().as_bytes());
            hoists_code.extend_from_slice(b" = ");
            // Only add /*#__PURE__*/ for VNodeCall (createElementVNode calls)
            if matches!(node, JsChildNode::VNodeCall(_)) {
                hoists_code.extend_from_slice(b"/*#__PURE__*/ ");
            }
            generate_js_child_node_to_bytes(ctx, node, &mut hoists_code);
            hoists_code.push(b'\n');
        }
    }

    // SAFETY: We only push valid UTF-8 strings
    unsafe { String::from_utf8_unchecked(hoists_code) }
}

/// Collect runtime helpers needed by hoisted nodes.
///
/// Since `generate_hoists()` takes `&CodegenContext` (immutable), helpers used in hoisted
/// VNodes are not tracked via `use_helper()`. This function pre-scans hoists to collect them.
pub(super) fn collect_hoist_helpers(root: &RootNode<'_>, helpers: &mut Vec<RuntimeHelper>) {
    for node in root.hoists.iter().flatten() {
        collect_helpers_from_js_child_node(node, helpers);
    }
}

fn collect_helpers_from_js_child_node(node: &JsChildNode<'_>, helpers: &mut Vec<RuntimeHelper>) {
    match node {
        JsChildNode::VNodeCall(vnode) => collect_helpers_from_vnode_call(vnode, helpers),
        JsChildNode::Object(obj) => {
            for prop in &obj.properties {
                collect_helpers_from_js_child_node(&prop.value, helpers);
            }
        }
        _ => {}
    }
}

fn collect_helpers_from_vnode_call(vnode: &VNodeCall<'_>, helpers: &mut Vec<RuntimeHelper>) {
    // Match the logic in generate_vnode_call_to_bytes
    if vnode.is_block {
        helpers.push(RuntimeHelper::OpenBlock);
        if vnode.is_component {
            helpers.push(RuntimeHelper::CreateBlock);
        } else {
            helpers.push(RuntimeHelper::CreateElementBlock);
        }
    } else if vnode.is_component {
        helpers.push(RuntimeHelper::CreateVNode);
    } else {
        helpers.push(RuntimeHelper::CreateElementVNode);
    }

    // Tag symbol (e.g., Fragment)
    if let VNodeTag::Symbol(helper) = &vnode.tag {
        helpers.push(*helper);
    }

    // Recurse into props (may contain nested VNodeCalls)
    if let Some(props) = &vnode.props {
        collect_helpers_from_props(props, helpers);
    }
}

fn collect_helpers_from_props(props: &PropsExpression<'_>, helpers: &mut Vec<RuntimeHelper>) {
    if let PropsExpression::Object(obj) = props {
        for prop in &obj.properties {
            collect_helpers_from_js_child_node(&prop.value, helpers);
        }
    }
}

/// Generate `JsChildNode` to bytes.
fn generate_js_child_node_to_bytes(
    ctx: &CodegenContext,
    node: &JsChildNode<'_>,
    out: &mut Vec<u8>,
) {
    match node {
        JsChildNode::VNodeCall(vnode) => generate_vnode_call_to_bytes(ctx, vnode, out),
        JsChildNode::SimpleExpression(exp) => {
            if exp.is_static {
                out.push(b'"');
                // Escape special characters in static string values (newlines, quotes, etc.)
                let escaped = escape_js_string(&exp.content);
                out.extend_from_slice(escaped.as_bytes());
                out.push(b'"');
            } else {
                // Expression should already be processed by transform
                out.extend_from_slice(exp.content.as_bytes());
            }
        }
        JsChildNode::Object(obj) => {
            out.extend_from_slice(b"{ ");
            for (i, prop) in obj.properties.iter().enumerate() {
                if i > 0 {
                    out.extend_from_slice(b", ");
                }
                // Key - quote if contains special characters like hyphens
                match &prop.key {
                    ExpressionNode::Simple(exp) => {
                        let key = &exp.content;
                        let needs_quote = !crate::codegen::helpers::is_valid_js_identifier(key);
                        if needs_quote {
                            out.push(b'"');
                            out.extend_from_slice(key.as_bytes());
                            out.push(b'"');
                        } else {
                            out.extend_from_slice(key.as_bytes());
                        }
                        out.extend_from_slice(b": ");
                    }
                    ExpressionNode::Compound(_) => out.extend_from_slice(b"null: "),
                }
                // Value
                generate_js_child_node_to_bytes(ctx, &prop.value, out);
            }
            out.extend_from_slice(b" }");
        }
        _ => out.extend_from_slice(b"null /* unsupported */"),
    }
}

/// Generate `VNodeCall` to bytes.
fn generate_vnode_call_to_bytes(ctx: &CodegenContext, vnode: &VNodeCall<'_>, out: &mut Vec<u8>) {
    // Block nodes use openBlock + createBlock/createElementBlock
    if vnode.is_block {
        out.push(b'(');
        out.extend_from_slice(ctx.helper(RuntimeHelper::OpenBlock).as_bytes());
        out.extend_from_slice(b"(), ");
        if vnode.is_component {
            out.extend_from_slice(ctx.helper(RuntimeHelper::CreateBlock).as_bytes());
        } else {
            out.extend_from_slice(ctx.helper(RuntimeHelper::CreateElementBlock).as_bytes());
        }
    } else if vnode.is_component {
        out.extend_from_slice(ctx.helper(RuntimeHelper::CreateVNode).as_bytes());
    } else {
        out.extend_from_slice(ctx.helper(RuntimeHelper::CreateElementVNode).as_bytes());
    }
    out.push(b'(');

    // Tag
    match &vnode.tag {
        VNodeTag::String(s) => {
            out.push(b'"');
            out.extend_from_slice(s.as_bytes());
            out.push(b'"');
        }
        VNodeTag::Symbol(helper) => out.extend_from_slice(ctx.helper(*helper).as_bytes()),
        VNodeTag::Call(_) => out.extend_from_slice(b"null"),
    }

    // Props
    if let Some(props) = &vnode.props {
        out.extend_from_slice(b", ");
        generate_props_expression_to_bytes(ctx, props, out);
    } else if vnode.children.is_some() || vnode.patch_flag.is_some() {
        out.extend_from_slice(b", null");
    }

    // Children
    if let Some(children) = &vnode.children {
        out.extend_from_slice(b", ");
        generate_vnode_children_to_bytes(ctx, children, out);
    } else if vnode.patch_flag.is_some() {
        out.extend_from_slice(b", null");
    }

    // Patch flag
    if let Some(patch_flag) = &vnode.patch_flag {
        out.extend_from_slice(b", ");
        out.extend_from_slice(patch_flag.bits().to_compact_string().as_bytes());
        out.extend_from_slice(b" /* ");
        let mut debug = String::default();
        use std::fmt::Write as _;
        let _ = write!(&mut debug, "{:?}", patch_flag);
        out.extend_from_slice(debug.as_bytes());
        out.extend_from_slice(b" */");
    }

    // Dynamic props
    if let Some(dynamic_props) = &vnode.dynamic_props {
        out.extend_from_slice(b", ");
        match dynamic_props {
            DynamicProps::String(s) => {
                out.extend_from_slice(s.as_bytes());
            }
            DynamicProps::Simple(exp) => {
                out.extend_from_slice(exp.content.as_bytes());
            }
        }
    }

    out.push(b')');

    // Close block wrapper
    if vnode.is_block {
        out.push(b')');
    }
}

/// Generate `PropsExpression` to bytes.
fn generate_props_expression_to_bytes(
    ctx: &CodegenContext,
    props: &PropsExpression<'_>,
    out: &mut Vec<u8>,
) {
    match props {
        PropsExpression::Object(obj) => {
            out.extend_from_slice(b"{ ");
            for (i, prop) in obj.properties.iter().enumerate() {
                if i > 0 {
                    out.extend_from_slice(b", ");
                }
                // Key - quote if contains special characters like hyphens
                match &prop.key {
                    ExpressionNode::Simple(exp) => {
                        let key = &exp.content;
                        let needs_quote = !crate::codegen::helpers::is_valid_js_identifier(key);
                        if needs_quote {
                            out.push(b'"');
                            out.extend_from_slice(key.as_bytes());
                            out.push(b'"');
                        } else {
                            out.extend_from_slice(key.as_bytes());
                        }
                        out.extend_from_slice(b": ");
                    }
                    ExpressionNode::Compound(_) => out.extend_from_slice(b"null: "),
                }
                // Value
                generate_js_child_node_to_bytes(ctx, &prop.value, out);
            }
            out.extend_from_slice(b" }");
        }
        PropsExpression::Simple(exp) => {
            if exp.is_static {
                out.push(b'"');
                out.extend_from_slice(exp.content.as_bytes());
                out.push(b'"');
            } else {
                // Expression should already be processed by transform
                out.extend_from_slice(exp.content.as_bytes());
            }
        }
        PropsExpression::Call(_) => out.extend_from_slice(b"null"),
    }
}

/// Generate `VNodeChildren` to bytes.
fn generate_vnode_children_to_bytes(
    _ctx: &CodegenContext,
    children: &VNodeChildren<'_>,
    out: &mut Vec<u8>,
) {
    match children {
        VNodeChildren::Single(text_child) => match text_child {
            TemplateTextChildNode::Text(text) => {
                out.push(b'"');
                out.extend_from_slice(escape_js_string(&text.content).as_bytes());
                out.push(b'"');
            }
            TemplateTextChildNode::Interpolation(_) => out.extend_from_slice(b"null"),
            TemplateTextChildNode::Compound(_) => out.extend_from_slice(b"null"),
        },
        VNodeChildren::Simple(exp) => {
            if exp.is_static {
                out.push(b'"');
                out.extend_from_slice(escape_js_string(&exp.content).as_bytes());
                out.push(b'"');
            } else {
                // Expression should already be processed by transform
                out.extend_from_slice(exp.content.as_bytes());
            }
        }
        _ => out.extend_from_slice(b"null"),
    }
}
