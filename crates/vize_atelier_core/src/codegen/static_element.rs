//! Static-element serialization for hoisted subtrees.
//!
//! Serializes a fully-static element (and its static descendants) as nested
//! `createElementVNode(...)` byte output. Split out of `generate` to keep that
//! file focused on the dynamic JS-IR serialization path.

use crate::{RuntimeHelper, TemplateChildNode};

use super::{context::CodegenContext, helpers::escape_js_string};
use vize_carton::String;

/// Serialize a fully-static element as a nested `createElementVNode(...)` for a
/// hoisted subtree. Children recurse the same way; text children collapse to a
/// string literal, matching @vue/compiler-core's hoisted static output.
pub(super) fn generate_static_element_to_bytes(
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
