//! Runtime-helper collection for hoisted nodes.
//!
//! Pre-scans the hoisted JS IR to collect the `RuntimeHelper`s its VNodes need.
//! Split out of `generate` to keep that file focused on serialization.

use crate::{
    JsChildNode, PropsExpression, RuntimeHelper, TemplateChildNode, VNodeCall, VNodeChildren,
    VNodeTag,
};

/// Collect runtime helpers needed by hoisted nodes.
///
/// Since `generate_hoists()` takes `&CodegenContext` (immutable), helpers used in hoisted
/// VNodes are not tracked via `use_helper()`. This function pre-scans hoists to collect them.
pub(super) fn collect_hoist_helpers(
    hoists: &[Option<JsChildNode<'_>>],
    helpers: &mut Vec<RuntimeHelper>,
) {
    for node in hoists.iter().flatten() {
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
