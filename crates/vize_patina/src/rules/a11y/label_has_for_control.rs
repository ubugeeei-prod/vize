//! Which controls and `aria-hidden` values satisfy `a11y/label-has-for`.

use crate::markup::{MarkupBindingKind, MarkupElement, MarkupElementKind};
use crate::rules::a11y::helpers::{
    is_aria_hidden_true, matches_configured_component, string_literal_value,
};
use crate::rules::a11y::markup_helpers;
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

const FORM_CONTROL_TAGS: &[&str] = &["input", "select", "textarea", "meter", "progress"];

pub(super) fn is_for_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("for") || name.eq_ignore_ascii_case("htmlFor")
}

pub(super) fn is_native_control_tag(tag: &str) -> bool {
    FORM_CONTROL_TAGS
        .iter()
        .any(|name| tag.eq_ignore_ascii_case(name))
}

fn expression_is_false(value: &str) -> bool {
    let value = value.trim();
    value == "false"
        || string_literal_value(value).is_some_and(|literal| literal.eq_ignore_ascii_case("false"))
}

pub(super) fn relief_hides_control(element: &ElementNode) -> bool {
    if is_aria_hidden_true(element) {
        return true;
    }
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) if attr.name.eq_ignore_ascii_case("aria-hidden") => attr
            .value
            .as_ref()
            .is_none_or(|value| !value.content.eq_ignore_ascii_case("false")),
        PropNode::Directive(dir) if dir.name == "bind" => {
            let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
                return false;
            };
            if !arg.is_static || !arg.content.eq_ignore_ascii_case("aria-hidden") {
                return false;
            }
            dir.exp.as_ref().is_none_or(|expression| match expression {
                ExpressionNode::Simple(simple) => !expression_is_false(simple.content),
                ExpressionNode::Compound(_) => true,
            })
        }
        _ => false,
    })
}

pub(super) fn relief_component_is_control(element: &ElementNode) -> bool {
    if element.tag_type != ElementType::Component || !element.tag.eq_ignore_ascii_case("component")
    {
        return false;
    }
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) if attr.name.eq_ignore_ascii_case("is") => attr
            .value
            .as_ref()
            .is_some_and(|value| is_native_control_tag(&value.content)),
        PropNode::Directive(dir) if dir.name == "bind" => {
            let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
                return false;
            };
            if !arg.is_static || !arg.content.eq_ignore_ascii_case("is") {
                return false;
            }
            matches!(&dir.exp, Some(ExpressionNode::Simple(simple)) if string_literal_value(simple.content).is_some_and(is_native_control_tag))
        }
        _ => false,
    })
}

pub(super) fn markup_hides_control(element: &MarkupElement<'_>) -> bool {
    if markup_helpers::is_aria_hidden_true_markup(element) {
        return true;
    }
    let mut hidden = false;
    element.walk_bindings(&mut |binding| {
        let aria = match binding.kind() {
            MarkupBindingKind::Attribute => {
                binding.is_unqualified_arg_eq_ignore_ascii_case("aria-hidden")
            }
            MarkupBindingKind::Bind => {
                binding.is_static_unqualified_arg_eq_ignore_ascii_case("aria-hidden")
            }
            _ => false,
        };
        if !aria {
            return;
        }
        if binding
            .static_value()
            .is_some_and(|value| value.eq_ignore_ascii_case("false"))
        {
            return;
        }
        if binding.expression().is_some_and(expression_is_false) {
            return;
        }
        hidden = true;
    });
    hidden
}

pub(super) fn markup_is_control(
    element: &MarkupElement<'_>,
    control_components: &[vize_l0::String],
) -> bool {
    if element.kind() == MarkupElementKind::Element
        && FORM_CONTROL_TAGS
            .iter()
            .any(|tag| element.is_unqualified_tag_exact(tag))
    {
        return true;
    }
    if matches_configured_component(element.tag(), control_components) {
        return true;
    }
    if element.kind() != MarkupElementKind::Component
        || !element.tag().eq_ignore_ascii_case("component")
    {
        return false;
    }
    let mut control = false;
    element.walk_bindings(&mut |binding| {
        control |= match binding.kind() {
            MarkupBindingKind::Attribute => {
                binding.is_unqualified_arg_eq_ignore_ascii_case("is")
                    && binding
                        .static_value()
                        .is_some_and(|value| is_native_control_tag(value))
            }
            MarkupBindingKind::Bind => {
                binding.is_static_unqualified_arg_eq_ignore_ascii_case("is")
                    && binding
                        .expression()
                        .and_then(string_literal_value)
                        .is_some_and(is_native_control_tag)
            }
            _ => false,
        };
    });
    control
}
