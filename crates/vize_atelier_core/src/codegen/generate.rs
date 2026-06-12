//! Hoist generation and JS node serialization.
//!
//! Generates hoisted variable declarations and serializes JS child nodes,
//! VNode calls, props expressions, and children to byte output.

use crate::{
    DynamicProps, ExpressionNode, JsChildNode, PropsExpression, RootNode, RuntimeHelper,
    TemplateChildNode, TemplateTextChildNode, VNodeCall, VNodeChildren, VNodeTag,
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

    // Recurse into a hoisted nested-static subtree's children so the helpers
    // used by descendant `createElementVNode` / `createTextVNode` calls are
    // declared in the import preamble.
    if let Some(VNodeChildren::Multiple(children)) = &vnode.children {
        collect_helpers_from_static_children(children, helpers);
    }
}

/// Collect helpers for a hoisted static children list, matching exactly what
/// [`generate_static_element_to_bytes`] / the `Multiple` codegen branch emit:
/// element children always need `createElementVNode`; a text child only needs
/// `createTextVNode` when it is emitted in array form (i.e. siblings include an
/// element), since a single/all-text run collapses to a string literal.
fn collect_helpers_from_static_children(
    children: &[TemplateChildNode<'_>],
    helpers: &mut Vec<RuntimeHelper>,
) {
    let has_element = children
        .iter()
        .any(|c| matches!(c, TemplateChildNode::Element(_)));
    for child in children.iter() {
        match child {
            TemplateChildNode::Element(el) => {
                helpers.push(RuntimeHelper::CreateElementVNode);
                collect_helpers_from_static_children(&el.children, helpers);
            }
            TemplateChildNode::Text(_) if has_element => {
                helpers.push(RuntimeHelper::CreateText);
            }
            _ => {}
        }
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
            // Vue keeps the first occurrence on duplicate attributes; vize's
            // parser records both nodes so the linter can flag the repeat,
            // so dedupe by key name here before emitting the props object.
            // (#958)
            let mut seen: vize_carton::FxHashSet<vize_carton::String> =
                vize_carton::FxHashSet::default();
            let mut emitted = 0usize;
            for prop in obj.properties.iter() {
                let key_string = match &prop.key {
                    ExpressionNode::Simple(exp) if exp.is_static => Some(exp.content.clone()),
                    _ => None,
                };
                if let Some(key) = &key_string {
                    if seen.contains(key.as_str()) {
                        continue;
                    }
                    seen.insert(key.clone());
                }
                if emitted > 0 {
                    out.push_str(", ");
                }
                emitted += 1;
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
    ctx: &CodegenContext,
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
        // A fully-static nested subtree hoisted as one recursive VNodeCall:
        // render the moved children as `[createElementVNode(...), "text", ...]`.
        VNodeChildren::Multiple(children) => {
            out.push('[');
            let mut emitted = 0usize;
            for child in children.iter() {
                match child {
                    TemplateChildNode::Element(el) => {
                        if emitted > 0 {
                            out.push_str(", ");
                        }
                        emitted += 1;
                        generate_static_element_to_bytes(ctx, el, out);
                    }
                    TemplateChildNode::Text(text) => {
                        if emitted > 0 {
                            out.push_str(", ");
                        }
                        emitted += 1;
                        let helper = ctx.helper(RuntimeHelper::CreateText);
                        out.push_str(helper);
                        out.push_str("(\"");
                        out.push_str(escape_js_string(&text.content).as_str());
                        out.push_str("\")");
                    }
                    _ => {}
                }
            }
            out.push(']');
        }
        _ => out.push_str("null"),
    }
}

/// Serialize a fully-static element as a nested `createElementVNode(...)` for a
/// hoisted subtree. Children recurse the same way; text children collapse to a
/// string literal, matching @vue/compiler-core's hoisted static output.
fn generate_static_element_to_bytes(
    ctx: &CodegenContext,
    el: &crate::ElementNode<'_>,
    out: &mut String,
) {
    out.push_str(ctx.helper(RuntimeHelper::CreateElementVNode));
    out.push_str("(\"");
    out.push_str(el.tag.as_str());
    out.push('"');

    // Props: static attributes and constant v-bind only (the subtree is static).
    let props = build_static_props(el);
    if let Some(props) = &props {
        out.push_str(", ");
        out.push_str(props.as_str());
    } else if !el.children.is_empty() {
        out.push_str(", null");
    }

    if !el.children.is_empty() {
        out.push_str(", ");
        // Single text child collapses to a string literal.
        if el.children.len() == 1
            && let TemplateChildNode::Text(text) = &el.children[0]
        {
            out.push('"');
            out.push_str(escape_js_string(&text.content).as_str());
            out.push('"');
        } else if el
            .children
            .iter()
            .all(|c| matches!(c, TemplateChildNode::Text(_)))
        {
            let mut combined = String::default();
            for c in el.children.iter() {
                if let TemplateChildNode::Text(t) = c {
                    combined.push_str(t.content.as_str());
                }
            }
            out.push('"');
            out.push_str(escape_js_string(&combined).as_str());
            out.push('"');
        } else {
            out.push('[');
            let mut emitted = 0usize;
            for c in el.children.iter() {
                match c {
                    TemplateChildNode::Element(child_el) => {
                        if emitted > 0 {
                            out.push_str(", ");
                        }
                        emitted += 1;
                        generate_static_element_to_bytes(ctx, child_el, out);
                    }
                    TemplateChildNode::Text(text) => {
                        if emitted > 0 {
                            out.push_str(", ");
                        }
                        emitted += 1;
                        out.push_str(ctx.helper(RuntimeHelper::CreateText));
                        out.push_str("(\"");
                        out.push_str(escape_js_string(&text.content).as_str());
                        out.push_str("\")");
                    }
                    _ => {}
                }
            }
            out.push(']');
        }
    }

    out.push(')');
}

/// Build the props-object literal for a static element, or `None` when it has
/// no renderable static props. Mirrors the dedupe and quoting rules used by the
/// main props codegen.
fn build_static_props(el: &crate::ElementNode<'_>) -> Option<String> {
    use crate::PropNode;

    let mut buf = String::default();
    buf.push_str("{ ");
    let mut seen: vize_carton::FxHashSet<vize_carton::String> = vize_carton::FxHashSet::default();
    let mut emitted = 0usize;

    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if attr.name == "ref" || seen.contains(attr.name.as_str()) {
                continue;
            }
            seen.insert(attr.name.clone());
            if emitted > 0 {
                buf.push_str(", ");
            }
            emitted += 1;
            let needs_quote = !crate::codegen::helpers::is_valid_js_identifier(&attr.name);
            if needs_quote {
                buf.push('"');
                buf.push_str(attr.name.as_str());
                buf.push('"');
            } else {
                buf.push_str(attr.name.as_str());
            }
            buf.push_str(": \"");
            if let Some(v) = &attr.value {
                buf.push_str(escape_js_string(&v.content).as_str());
            }
            buf.push('"');
        }
    }

    if emitted == 0 {
        return None;
    }
    buf.push_str(" }");
    Some(buf)
}
