//! Pure literal styles enter the existing static-template attribute path.

use vize_l0::Box;
use vize_relief::{
    AttributeNode, ElementNode, ExpressionNode, PropNode, TextNode, literal_object_style,
};

use super::TransformContext;

pub(super) fn fold<'a>(ctx: &TransformContext<'a>, element: &mut ElementNode<'a>) {
    let mut selected = None;
    for (index, prop) in element.props.iter().enumerate() {
        match prop {
            PropNode::Attribute(attribute) if attribute.name.eq_ignore_ascii_case("style") => {
                return;
            }
            PropNode::Directive(directive) if directive.name == "bind" => {
                let Some(ExpressionNode::Simple(name)) = &directive.arg else {
                    return;
                };
                if !name.is_static {
                    return;
                }
                if name.content.eq_ignore_ascii_case("style") {
                    if name.content != "style"
                        || selected.is_some()
                        || !directive.modifiers.is_empty()
                    {
                        return;
                    }
                    selected = Some(index);
                }
            }
            _ => {}
        }
    }
    let Some(index) = selected else {
        return;
    };
    let Some(PropNode::Directive(directive)) = element.props.get(index) else {
        return;
    };
    let Some(ExpressionNode::Simple(name)) = &directive.arg else {
        return;
    };
    let Some(ExpressionNode::Simple(value)) = &directive.exp else {
        return;
    };
    let Some(expression) = crate::retained::retained_whole_expression(value) else {
        return;
    };
    let Some(css) = literal_object_style(expression.ast) else {
        return;
    };
    let attribute = AttributeNode {
        name: "style",
        name_loc: name.loc.clone(),
        value: Some(TextNode::new(
            ctx.allocator.alloc_str(&css),
            value.loc.clone(),
        )),
        loc: directive.loc.clone(),
    };
    if let Some(prop) = element.props.get_mut(index) {
        *prop = PropNode::Attribute(Box::new_in(attribute, &ctx.allocator));
    }
}
