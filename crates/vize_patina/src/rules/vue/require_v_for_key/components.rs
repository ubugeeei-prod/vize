//! Classify repeated native leaves and custom component key ownership.

use crate::markup::{MarkupBindingKind, MarkupElement, MarkupNode};
use vize_l0::{ensure_sufficient_stack, is_native_tag};
use vize_relief::{ElementNode, ExpressionNode, PropNode, TemplateChildNode};

pub(super) fn markup_element_is_custom_component(element: &MarkupElement<'_>) -> bool {
    if element.is_component() || !is_native_tag(element.tag()) || element.has_directive("is") {
        return true;
    }
    let mut dynamic = false;
    element.walk_bindings(&mut |binding| {
        dynamic |= matches!(
            binding.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) && binding.is_unqualified_arg_exact("is");
    });
    dynamic
}

pub(super) fn has_markup_native_child(element: &MarkupElement<'_>, exact: bool) -> bool {
    let mut found = false;
    element.walk_children(&mut |child| {
        if !found && let MarkupNode::Element(child) = child {
            found = if matches!(child.tag(), "template" | "slot")
                || (!exact && (child.is_tag("template") || child.is_tag("slot")))
            {
                ensure_sufficient_stack(|| has_markup_native_child(&child, exact))
            } else {
                !markup_element_is_custom_component(&child)
            };
        }
    });
    found
}

pub(in crate::rules::vue) fn relief_element_is_custom_component(element: &ElementNode<'_>) -> bool {
    !is_native_tag(element.tag)
        || element.props.iter().any(|prop| match prop {
            PropNode::Attribute(attribute) => attribute.name == "is",
            PropNode::Directive(directive) => directive.name == "is"
                || (directive.name == "bind"
                    && matches!(directive.arg.as_ref(), Some(ExpressionNode::Simple(arg)) if arg.content == "is")),
        })
}

pub(super) fn relief_has_native_child(element: &ElementNode<'_>) -> bool {
    element.children.iter().any(|child| {
        if let TemplateChildNode::Element(child) = child {
            if matches!(child.tag, "template" | "slot") {
                ensure_sufficient_stack(|| relief_has_native_child(child))
            } else {
                !relief_element_is_custom_component(child)
            }
        } else {
            false
        }
    })
}

pub(super) fn relief_template_has_slot_directive(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(directive) if directive.name == "slot"
        )
    })
}
