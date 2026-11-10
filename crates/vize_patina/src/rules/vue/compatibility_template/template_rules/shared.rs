//! Shared helpers for template-specific compatibility rules.

use crate::context::LintContext;
use vize_l0::{String, cstr, is_native_tag};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, TemplateChildNode};

pub(super) fn static_argument<'a>(directive: &'a DirectiveNode<'a>) -> Option<&'a str> {
    match &directive.arg {
        Some(ExpressionNode::Simple(argument)) if argument.is_static => Some(argument.content),
        _ => None,
    }
}

pub(super) fn simple_expression<'a>(expression: Option<&'a ExpressionNode<'a>>) -> Option<&'a str> {
    match expression {
        Some(ExpressionNode::Simple(expression)) => Some(expression.content),
        _ => None,
    }
}

pub(super) fn mirrors_assignment(binding: &str, handler: &str) -> bool {
    let binding = compact_js(binding);
    let handler = compact_js(handler);
    handler == cstr!("{}=$event", binding)
        || handler.split_once("=>").is_some_and(|(parameter, body)| {
            let parameter = parameter.trim_matches(['(', ')']);
            body == cstr!("{}={}", binding, parameter)
        })
}

fn compact_js(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn has_key(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attribute) => attribute.name == "key",
        PropNode::Directive(directive) => {
            directive.name == "bind" && static_argument(directive) == Some("key")
        }
    })
}

pub(super) fn check_conditional_siblings(
    ctx: &mut LintContext<'_>,
    children: &[TemplateChildNode<'_>],
) {
    let elements: Vec<_> = children
        .iter()
        .filter_map(|child| match child {
            TemplateChildNode::Element(element)
                if !is_native_tag(element.tag)
                    && !matches!(element.tag, "component" | "slot" | "template")
                    && has_conditional_directive(element) =>
            {
                Some(element.as_ref())
            }
            _ => None,
        })
        .collect();
    for element in elements.iter().filter(|element| !has_key(element)) {
        if elements
            .iter()
            .filter(|sibling| sibling.tag == element.tag)
            .count()
            > 1
        {
            ctx.warn_with_help(
                cstr!(
                    "Conditionally rendered repeated component `{}` should have a key",
                    element.tag
                ),
                &element.loc,
                "Add a unique `key` or `:key` to each conditional branch",
            );
        }
    }
}

fn has_conditional_directive(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| {
        matches!(prop, PropNode::Directive(directive)
            if matches!(directive.name, "if" | "else-if" | "else"))
    })
}
