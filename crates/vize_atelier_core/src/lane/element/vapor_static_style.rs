//! Pure literal styles enter the existing static-template attribute path.

use vize_l0::{Box, String};
use vize_relief::{
    AttributeNode, DirectiveNode, ElementNode, ExpressionNode, PropNode, TextNode,
    literal_object_style,
};

use super::TransformContext;

/// The CSS of a `:style` whose retained value is a literal object. The props
/// loops call this while they already dispatch on each directive, so only an
/// arg spelled `style` pays for more than one length check.
#[inline(always)]
pub(super) fn literal_css(directive: &DirectiveNode<'_>) -> Option<String> {
    match &directive.arg {
        Some(ExpressionNode::Simple(name)) if name.content == "style" => evaluate(directive),
        _ => None,
    }
}

#[inline(never)]
fn evaluate(directive: &DirectiveNode<'_>) -> Option<String> {
    let Some(ExpressionNode::Simple(value)) = &directive.exp else {
        return None;
    };
    let expression = crate::retained::retained_whole_expression(value)?;
    literal_object_style(expression.ast)
}

/// Replace the binding at `index` with a static `css` attribute when it is the
/// element's only style source.
pub(super) fn fold_at<'a>(
    ctx: &TransformContext<'a>,
    element: &mut ElementNode<'a>,
    index: usize,
    css: &str,
) -> bool {
    let mut selected = None;
    for (position, prop) in element.props.iter().enumerate() {
        match prop {
            PropNode::Attribute(attribute) if attribute.name.eq_ignore_ascii_case("style") => {
                return false;
            }
            PropNode::Directive(directive) if directive.name == "bind" => {
                let Some(ExpressionNode::Simple(name)) = &directive.arg else {
                    return false;
                };
                if !name.is_static {
                    return false;
                }
                if name.content.eq_ignore_ascii_case("style") {
                    if name.content != "style"
                        || selected.is_some()
                        || !directive.modifiers.is_empty()
                    {
                        return false;
                    }
                    selected = Some(position);
                }
            }
            _ => {}
        }
    }
    if selected != Some(index) {
        return false;
    }
    let Some(PropNode::Directive(directive)) = element.props.get(index) else {
        return false;
    };
    let (Some(ExpressionNode::Simple(name)), Some(ExpressionNode::Simple(value))) =
        (&directive.arg, &directive.exp)
    else {
        return false;
    };
    let attribute = AttributeNode {
        name: "style",
        name_loc: name.loc.clone(),
        value: Some(TextNode::new(
            ctx.allocator.alloc_str(css),
            value.loc.clone(),
        )),
        loc: directive.loc.clone(),
    };
    let Some(prop) = element.props.get_mut(index) else {
        return false;
    };
    *prop = PropNode::Attribute(Box::new_in(attribute, &ctx.allocator));
    true
}
