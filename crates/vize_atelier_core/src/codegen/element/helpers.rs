//! Element helper functions and predicates.
//!
//! Utility functions for checking element properties like directives,
//! renderable props, and special element types.

use crate::ast::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode,
};
use vize_carton::is_builtin_directive;

use super::super::{
    context::CodegenContext, node::generate_node, props::is_supported_directive,
    v_for::generate_for, v_if::generate_if,
};

/// Check if element has v-once directive
pub fn has_v_once(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        if let PropNode::Directive(dir) = prop {
            dir.name.as_str() == "once"
        } else {
            false
        }
    })
}

/// Check if a template child node is whitespace-only text or a comment.
/// Used to skip generating empty default slots for components.
pub(crate) fn is_whitespace_or_comment(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Text(t) => t.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => true,
        _ => false,
    }
}

/// Check if element has v-show directive
pub fn has_vshow_directive(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        if let PropNode::Directive(dir) = prop {
            dir.name.as_str() == "show" && dir.exp.is_some()
        } else {
            false
        }
    })
}

/// Check if element has custom directives
pub fn has_custom_directives(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        if let PropNode::Directive(dir) = prop {
            !is_builtin_directive(&dir.name)
        } else {
            false
        }
    })
}

/// Get custom directives from element
pub fn get_custom_directives<'a, 'b>(el: &'b ElementNode<'a>) -> Vec<&'b DirectiveNode<'a>> {
    el.props
        .iter()
        .filter_map(|prop| {
            if let PropNode::Directive(dir) = prop {
                if !is_builtin_directive(&dir.name) {
                    return Some(dir.as_ref());
                }
            }
            None
        })
        .collect()
}

/// Check if native element has v-model directive
pub fn has_vmodel_directive(el: &ElementNode<'_>) -> bool {
    // Only native elements use withDirectives for v-model
    if el.tag_type != ElementType::Element {
        return false;
    }
    // Only input, textarea, select support v-model
    if !matches!(el.tag.as_str(), "input" | "textarea" | "select") {
        return false;
    }
    el.props.iter().any(|prop| {
        if let PropNode::Directive(dir) = prop {
            dir.name.as_str() == "model"
        } else {
            false
        }
    })
}

/// Get v-model directive from element
pub(crate) fn get_vmodel_directive<'a, 'b>(
    el: &'b ElementNode<'a>,
) -> Option<&'b DirectiveNode<'a>> {
    el.props.iter().find_map(|prop| {
        if let PropNode::Directive(dir) = prop {
            if dir.name.as_str() == "model" {
                return Some(dir.as_ref());
            }
        }
        None
    })
}

/// Check if a prop is the `is` attribute or `:is` binding (used by dynamic components)
pub(crate) fn is_is_prop(p: &PropNode<'_>) -> bool {
    match p {
        PropNode::Attribute(attr) => attr.name == "is",
        PropNode::Directive(dir) => {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    return arg.content == "is";
                }
            }
            false
        }
    }
}

/// Check if a tag is Vue's dynamic component special tag.
pub(crate) fn is_dynamic_component_tag(tag: &str) -> bool {
    matches!(tag, "component" | "Component")
}

/// Check if a single prop is renderable (not v-show or unsupported directive)
pub(crate) fn is_renderable_prop(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(_) => true,
        PropNode::Directive(dir) => is_supported_directive(dir),
    }
}

/// Check if element has any renderable props
pub fn has_renderable_props(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| is_renderable_prop(prop))
}

/// Generate root node (wrapped in block)
pub fn generate_root_node(ctx: &mut CodegenContext, node: &TemplateChildNode<'_>) {
    match node {
        TemplateChildNode::Element(el) => super::block::generate_element_block(ctx, el),
        TemplateChildNode::If(if_node) => generate_if(ctx, if_node),
        TemplateChildNode::For(for_node) => generate_for(ctx, for_node),
        _ => generate_node(ctx, node),
    }
}
