//! Serialization for fully-static nested VNodes.

use vize_s0::{String, ensure_sufficient_stack};

use crate::{ElementNode, PropNode, RuntimeHelper, TemplateChildNode};

use super::super::{context::CodegenContext, helpers::escape_js_string};

/// Serialize a fully-static element as a nested `createElementVNode(...)` for a
/// hoisted subtree. Children recurse the same way; text children collapse to a
/// string literal, matching @vue/compiler-core's hoisted static output.
pub(super) fn generate_static_element_to_bytes(
    ctx: &CodegenContext,
    el: &ElementNode<'_>,
    out: &mut String,
) {
    out.push_str(ctx.vnode_helper(RuntimeHelper::CreateElementVNode));
    out.push_str("(\"");
    out.push_str(el.tag);
    out.push('"');

    let props = build_static_props(el);
    if let Some(props) = &props {
        out.push_str(", ");
        out.push_str(props.as_str());
    } else if !el.children.is_empty() {
        out.push_str(", null");
    }

    if !el.children.is_empty() {
        out.push_str(", ");
        generate_static_children_to_bytes(ctx, el, out);
    }

    out.push(')');
}

fn generate_static_children_to_bytes(ctx: &CodegenContext, el: &ElementNode<'_>, out: &mut String) {
    if el.children.len() == 1
        && let TemplateChildNode::Text(text) = &el.children[0]
    {
        out.push('"');
        out.push_str(escape_js_string(text.content).as_str());
        out.push('"');
    } else if el
        .children
        .iter()
        .all(|child| matches!(child, TemplateChildNode::Text(_)))
    {
        let mut combined = String::default();
        for child in el.children.iter() {
            if let TemplateChildNode::Text(text) = child {
                combined.push_str(text.content);
            }
        }
        out.push('"');
        out.push_str(escape_js_string(&combined).as_str());
        out.push('"');
    } else {
        generate_static_child_array_to_bytes(ctx, &el.children, out);
    }
}

fn generate_static_child_array_to_bytes(
    ctx: &CodegenContext,
    children: &[TemplateChildNode<'_>],
    out: &mut String,
) {
    out.push('[');
    let mut emitted = 0usize;
    for child in children {
        match child {
            TemplateChildNode::Element(child_el) => {
                if emitted > 0 {
                    out.push_str(", ");
                }
                emitted += 1;
                ensure_sufficient_stack(|| {
                    generate_static_element_to_bytes(ctx, child_el, out);
                });
            }
            TemplateChildNode::Text(text) => {
                if emitted > 0 {
                    out.push_str(", ");
                }
                emitted += 1;
                out.push_str(ctx.helper(RuntimeHelper::CreateText));
                out.push_str("(\"");
                out.push_str(escape_js_string(text.content).as_str());
                out.push_str("\")");
            }
            _ => {}
        }
    }
    out.push(']');
}

/// Build the props-object literal for a static element, or `None` when it has
/// no renderable static props. Mirrors the dedupe and quoting rules used by the
/// main props codegen.
fn build_static_props(el: &ElementNode<'_>) -> Option<String> {
    let mut buf = String::default();
    buf.push_str("{ ");
    let mut seen: vize_s0::FxHashSet<vize_s0::String> = vize_s0::FxHashSet::default();
    let mut emitted = 0usize;

    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if attr.name == "ref" || seen.contains(attr.name) {
                continue;
            }
            seen.insert(attr.name.into());
            if emitted > 0 {
                buf.push_str(", ");
            }
            emitted += 1;
            let needs_quote = !crate::codegen::helpers::is_valid_js_identifier(attr.name);
            if needs_quote {
                buf.push('"');
                buf.push_str(attr.name);
                buf.push('"');
            } else {
                buf.push_str(attr.name);
            }
            buf.push_str(": \"");
            if let Some(value) = &attr.value {
                buf.push_str(escape_js_string(value.content).as_str());
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
