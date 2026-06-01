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
    let mut hoists_code = String::default();

    for (i, hoist) in root.hoists.iter().enumerate() {
        if let Some(node) = hoist {
            hoists_code.push_str("const _hoisted_");
            hoists_code.push_str((i + 1).to_compact_string().as_str());
            hoists_code.push_str(" = ");
            // Only add /*#__PURE__*/ for VNodeCall (createElementVNode calls)
            if matches!(node, JsChildNode::VNodeCall(_)) {
                hoists_code.push_str("/*#__PURE__*/ ");
            }
            generate_js_child_node_to_bytes(ctx, node, &mut hoists_code);
            hoists_code.push('\n');
        }
    }

    hoists_code
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
fn generate_js_child_node_to_bytes(ctx: &CodegenContext, node: &JsChildNode<'_>, out: &mut String) {
    match node {
        JsChildNode::VNodeCall(vnode) => generate_vnode_call_to_bytes(ctx, vnode, out),
        JsChildNode::SimpleExpression(exp) => {
            if exp.is_static {
                out.push('"');
                // Escape special characters in static string values (newlines, quotes, etc.)
                let escaped = escape_js_string(&exp.content);
                out.push_str(escaped.as_str());
                out.push('"');
            } else {
                // Expression should already be processed by transform
                out.push_str(exp.content.as_str());
            }
        }
        JsChildNode::Object(obj) => {
            out.push_str("{ ");
            for (i, prop) in obj.properties.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                // Key - quote if contains special characters like hyphens
                match &prop.key {
                    ExpressionNode::Simple(exp) => {
                        let key = &exp.content;
                        let needs_quote = !crate::codegen::helpers::is_valid_js_identifier(key);
                        if needs_quote {
                            out.push('"');
                            out.push_str(key.as_str());
                            out.push('"');
                        } else {
                            out.push_str(key.as_str());
                        }
                        out.push_str(": ");
                    }
                    ExpressionNode::Compound(_) => out.push_str("null: "),
                }
                // Value
                generate_js_child_node_to_bytes(ctx, &prop.value, out);
            }
            out.push_str(" }");
        }
        _ => out.push_str("null /* unsupported */"),
    }
}

/// Generate `VNodeCall` to bytes.
fn generate_vnode_call_to_bytes(ctx: &CodegenContext, vnode: &VNodeCall<'_>, out: &mut String) {
    // Block nodes use openBlock + createBlock/createElementBlock
    if vnode.is_block {
        out.push('(');
        out.push_str(ctx.helper(RuntimeHelper::OpenBlock));
        out.push_str("(), ");
        if vnode.is_component {
            out.push_str(ctx.helper(RuntimeHelper::CreateBlock));
        } else {
            out.push_str(ctx.helper(RuntimeHelper::CreateElementBlock));
        }
    } else if vnode.is_component {
        out.push_str(ctx.helper(RuntimeHelper::CreateVNode));
    } else {
        out.push_str(ctx.helper(RuntimeHelper::CreateElementVNode));
    }
    out.push('(');

    // Tag
    match &vnode.tag {
        VNodeTag::String(s) => {
            out.push('"');
            out.push_str(s.as_str());
            out.push('"');
        }
        VNodeTag::Symbol(helper) => out.push_str(ctx.helper(*helper)),
        VNodeTag::Call(_) => out.push_str("null"),
    }

    // Props
    if let Some(props) = &vnode.props {
        out.push_str(", ");
        generate_props_expression_to_bytes(ctx, props, out);
    } else if vnode.children.is_some() || vnode.patch_flag.is_some() {
        out.push_str(", null");
    }

    // Children
    if let Some(children) = &vnode.children {
        out.push_str(", ");
        generate_vnode_children_to_bytes(ctx, children, out);
    } else if vnode.patch_flag.is_some() {
        out.push_str(", null");
    }

    // Patch flag
    if let Some(patch_flag) = &vnode.patch_flag {
        out.push_str(", ");
        out.push_str(patch_flag.bits().to_compact_string().as_str());
        out.push_str(" /* ");
        let mut debug = String::default();
        use std::fmt::Write as _;
        let _ = write!(&mut debug, "{:?}", patch_flag);
        out.push_str(debug.as_str());
        out.push_str(" */");
    }

    // Dynamic props
    if let Some(dynamic_props) = &vnode.dynamic_props {
        out.push_str(", ");
        match dynamic_props {
            DynamicProps::String(s) => {
                out.push_str(s.as_str());
            }
            DynamicProps::Simple(exp) => {
                out.push_str(exp.content.as_str());
            }
        }
    }

    out.push(')');

    // Close block wrapper
    if vnode.is_block {
        out.push(')');
    }
}

/// Generate `PropsExpression` to bytes.
fn generate_props_expression_to_bytes(
    ctx: &CodegenContext,
    props: &PropsExpression<'_>,
    out: &mut String,
) {
    match props {
        PropsExpression::Object(obj) => {
            out.push_str("{ ");
            for (i, prop) in obj.properties.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                // Key - quote if contains special characters like hyphens
                match &prop.key {
                    ExpressionNode::Simple(exp) => {
                        let key = &exp.content;
                        let needs_quote = !crate::codegen::helpers::is_valid_js_identifier(key);
                        if needs_quote {
                            out.push('"');
                            out.push_str(key.as_str());
                            out.push('"');
                        } else {
                            out.push_str(key.as_str());
                        }
                        out.push_str(": ");
                    }
                    ExpressionNode::Compound(_) => out.push_str("null: "),
                }
                // Value
                generate_js_child_node_to_bytes(ctx, &prop.value, out);
            }
            out.push_str(" }");
        }
        PropsExpression::Simple(exp) => {
            if exp.is_static {
                out.push('"');
                out.push_str(exp.content.as_str());
                out.push('"');
            } else {
                // Expression should already be processed by transform
                out.push_str(exp.content.as_str());
            }
        }
        PropsExpression::Call(_) => out.push_str("null"),
    }
}

/// Generate `VNodeChildren` to bytes.
fn generate_vnode_children_to_bytes(
    _ctx: &CodegenContext,
    children: &VNodeChildren<'_>,
    out: &mut String,
) {
    match children {
        VNodeChildren::Single(text_child) => match text_child {
            TemplateTextChildNode::Text(text) => {
                out.push('"');
                out.push_str(escape_js_string(&text.content).as_str());
                out.push('"');
            }
            TemplateTextChildNode::Interpolation(_) => out.push_str("null"),
            TemplateTextChildNode::Compound(_) => out.push_str("null"),
        },
        VNodeChildren::Simple(exp) => {
            if exp.is_static {
                out.push('"');
                out.push_str(escape_js_string(&exp.content).as_str());
                out.push('"');
            } else {
                // Expression should already be processed by transform
                out.push_str(exp.content.as_str());
            }
        }
        _ => out.push_str("null"),
    }
}
