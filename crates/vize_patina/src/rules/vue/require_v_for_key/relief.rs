//! The relief-AST form of the rule, used where the markup facade is not.

use super::components::relief_element_is_custom_component;
use crate::context::LintContext;
use crate::markup::MarkupElement;
use vize_l0::ensure_sufficient_stack;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, TemplateChildNode};

fn relief_directive_is_bound_key(directive: &DirectiveNode<'_>) -> bool {
    directive.name == "bind"
        && matches!(
            directive.arg.as_ref(),
            Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == "key"
        )
}

/// The relief form of [`RequireVForKey::check_keyed_element`].
pub(super) fn relief_check(ctx: &mut LintContext<'_>, element: &ElementNode<'_>) {
    if element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if relief_directive_is_bound_key(directive)))
    {
        return;
    }
    if matches!(element.tag, "template" | "slot") {
        for child in &element.children {
            if let TemplateChildNode::Element(child) = child
                && !child.props.iter().any(|prop| {
                    matches!(prop, PropNode::Directive(directive) if directive.name == "for")
                })
            {
                ensure_sufficient_stack(|| relief_check(ctx, child));
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
