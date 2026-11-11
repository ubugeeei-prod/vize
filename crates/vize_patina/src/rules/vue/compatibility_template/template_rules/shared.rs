//! Shared helpers for template-specific compatibility rules.

use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

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

pub(super) fn has_key(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attribute) => attribute.name == "key",
        PropNode::Directive(directive) => {
            directive.name == "bind" && static_argument(directive) == Some("key")
        }
    })
}
