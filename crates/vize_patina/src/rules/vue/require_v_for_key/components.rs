//! Classify repeated native leaves and custom component key ownership.

use crate::markup::{MarkupBindingKind, MarkupElement};
use vize_l0::is_native_tag;
use vize_relief::{ElementNode, ExpressionNode, PropNode};

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

pub(in crate::rules::vue) fn relief_element_is_custom_component(element: &ElementNode<'_>) -> bool {
    !is_native_tag(element.tag)
        || element.props.iter().any(|prop| match prop {
            PropNode::Attribute(attribute) => attribute.name == "is",
            PropNode::Directive(directive) => directive.name == "is"
                || (directive.name == "bind"
                    && matches!(directive.arg.as_ref(), Some(ExpressionNode::Simple(arg)) if arg.content == "is")),
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
