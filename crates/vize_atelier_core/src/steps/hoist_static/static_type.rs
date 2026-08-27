//! Whole-subtree staticness classification for the hoisting transform.
//!
//! Every predicate here answers a question about an element *and everything
//! under it*, so each one descends the tree once per nesting level. That makes
//! them recursion steps in the sense of `vize_s0::recursion`: their call
//! depth is bounded by the input, not by the code, so each descent is wrapped in
//! `ensure_sufficient_stack` — otherwise a deeply nested template aborts the
//! process instead of compiling.

use vize_s0::ensure_sufficient_stack;

use super::is_hoistable_static_prop;
use crate::{ElementNode, ElementType, PropNode, TemplateChildNode};

/// Static type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticType {
    NotStatic = 0,
    FullyStatic = 1,
    HasDynamicText = 2,
}

/// Check if a node is fully static (can be hoisted)
pub fn is_static_node(node: &TemplateChildNode<'_>) -> bool {
    match node {
        TemplateChildNode::Text(_) => true,
        TemplateChildNode::Comment(_) => true,
        TemplateChildNode::Element(el) => ensure_sufficient_stack(|| is_static_element(el)),
        TemplateChildNode::Interpolation(_) => false,
        TemplateChildNode::If(_) => false,
        TemplateChildNode::For(_) => false,
        _ => false,
    }
}

/// Check if an element is fully static
fn is_static_element(el: &ElementNode<'_>) -> bool {
    // Components are not static
    if el.tag_type != ElementType::Element {
        return false;
    }

    // Check for dynamic props or ref
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(_) if el.tag == "svg" => return false,
            PropNode::Directive(_) if !is_hoistable_static_prop(prop) => return false,
            PropNode::Directive(_) => {}
            PropNode::Attribute(attr) => {
                // ref attribute prevents hoisting - refs need runtime owner context
                if attr.name == "ref" {
                    return false;
                }
            }
        }
    }

    // Check children recursively. Nested fully-static element subtrees are
    // hoistable: `create_children_expression` builds them into a single
    // recursive VNodeCall, matching @vue/compiler-core's static caching of a
    // top-most static element with its whole subtree as one cached vnode.
    for child in el.children.iter() {
        // Comments prevent hoisting since they can't be serialized to VNodeCall children
        if matches!(child, TemplateChildNode::Comment(_)) {
            return false;
        }
        if !is_static_node(child) {
            return false;
        }
    }

    true
}

/// Get the static type of a node
pub fn get_static_type(node: &TemplateChildNode<'_>) -> StaticType {
    match node {
        TemplateChildNode::Text(_) => StaticType::FullyStatic,
        TemplateChildNode::Comment(_) => StaticType::FullyStatic,
        TemplateChildNode::Element(el) => ensure_sufficient_stack(|| get_element_static_type(el)),
        TemplateChildNode::Interpolation(_) => StaticType::NotStatic,
        _ => StaticType::NotStatic,
    }
}

fn get_element_static_type(el: &ElementNode<'_>) -> StaticType {
    if el.tag_type != ElementType::Element {
        return StaticType::NotStatic;
    }

    // Check for any dynamic content
    let mut has_dynamic_text = false;

    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(_) if el.tag == "svg" => {
                return StaticType::NotStatic;
            }
            PropNode::Directive(_) if !is_hoistable_static_prop(prop) => {
                // Non-constant directives make the element dynamic.
                return StaticType::NotStatic;
            }
            PropNode::Directive(_) => {}
            PropNode::Attribute(attr) => {
                // ref attribute prevents hoisting - refs need runtime owner context
                if attr.name == "ref" {
                    return StaticType::NotStatic;
                }
            }
        }
    }

    // Check children
    for child in el.children.iter() {
        match child {
            TemplateChildNode::Interpolation(_) => {
                has_dynamic_text = true;
            }
            // A nested element keeps the parent fully static only when the
            // child subtree is itself fully static (no dynamic text either):
            // such a subtree is serialized into one recursive VNodeCall by
            // `create_children_expression`. Any dynamic content disqualifies it.
            TemplateChildNode::Element(child_el) => {
                match ensure_sufficient_stack(|| get_element_static_type(child_el)) {
                    StaticType::FullyStatic => {}
                    _ => return StaticType::NotStatic,
                }
            }
            TemplateChildNode::If(_) | TemplateChildNode::For(_) => {
                return StaticType::NotStatic;
            }
            // Comments prevent hoisting since they can't be serialized to VNodeCall children
            TemplateChildNode::Comment(_) => {
                return StaticType::NotStatic;
            }
            _ => {}
        }
    }

    if has_dynamic_text {
        StaticType::HasDynamicText
    } else {
        StaticType::FullyStatic
    }
}

pub(super) fn has_only_static_nested_children(el: &ElementNode<'_>) -> bool {
    if el.children.is_empty() {
        return false;
    }

    el.children.iter().all(is_static_nested_child)
}

fn is_static_nested_child(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => true,
        TemplateChildNode::Element(el) => {
            ensure_sufficient_stack(|| is_plain_static_nested_element(el))
        }
        _ => false,
    }
}

pub(super) fn has_only_native_element_descendants(el: &ElementNode<'_>) -> bool {
    el.children.iter().all(|child| match child {
        TemplateChildNode::Text(_)
        | TemplateChildNode::Interpolation(_)
        | TemplateChildNode::Comment(_) => true,
        TemplateChildNode::Element(child_el) if child_el.tag_type == ElementType::Element => {
            ensure_sufficient_stack(|| has_only_native_element_descendants(child_el))
        }
        _ => false,
    })
}

fn is_plain_static_nested_element(el: &ElementNode<'_>) -> bool {
    match el.tag_type {
        ElementType::Element => {
            props_are_static_attrs(el) && el.children.iter().all(is_static_nested_child)
        }
        ElementType::Slot => props_are_static_attrs(el),
        _ => false,
    }
}

fn props_are_static_attrs(el: &ElementNode<'_>) -> bool {
    el.props.iter().all(is_hoistable_static_prop)
}
