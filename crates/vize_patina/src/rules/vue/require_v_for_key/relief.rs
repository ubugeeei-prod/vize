//! The relief-AST form of the rule, used where the markup facade is not.

use super::components::{relief_element_is_custom_component, relief_template_has_slot_directive};
use super::object_has_static_key;
use crate::context::LintContext;
use crate::markup::MarkupElement;
use vize_l0::ensure_sufficient_stack;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, TemplateChildNode};

fn relief_element_has_key(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name == "key",
        PropNode::Directive(dir) => {
            if relief_directive_is_bound_key(dir) {
                return true;
            }
            dir.name == "bind"
                && dir.arg.is_none()
                && matches!(&dir.exp, Some(ExpressionNode::Simple(expression)) if object_has_static_key(expression.content))
        }
    })
}

fn relief_directive_is_bound_key(directive: &DirectiveNode<'_>) -> bool {
    directive.name == "bind"
        && matches!(
            directive.arg.as_ref(),
            Some(ExpressionNode::Simple(arg)) if arg.content == "key"
        )
}

/// The relief form of [`RequireVForKey::check_keyed_element`].
pub(super) fn relief_check(ctx: &mut LintContext<'_>, element: &ElementNode<'_>, root: bool) {
    let keyed = if root {
        relief_element_has_key(element)
    } else {
        element
            .props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(directive) if relief_directive_is_bound_key(directive)))
    };
    if keyed {
        return;
    }
    if matches!(element.tag, "template" | "slot") {
        if !relief_template_has_slot_directive(element) {
            for child in &element.children {
                if let TemplateChildNode::Element(child) = child
                    && !child.props.iter().any(|prop| {
                        matches!(prop, PropNode::Directive(directive) if directive.name == "for")
                    })
                {
                    ensure_sufficient_stack(|| relief_check(ctx, child, false));
                }
            }
        }
        return;
    }
    if relief_element_is_custom_component(element) {
        return;
    }
    let range = MarkupElement::new(element).open_tag_range(ctx.source);
    let message = ctx.t_fmt("vue/require-v-for-key.message", &[("tag", element.tag)]);
    let help = ctx.t("vue/require-v-for-key.help");
    ctx.error_at_with_help(message, range, help);
}
