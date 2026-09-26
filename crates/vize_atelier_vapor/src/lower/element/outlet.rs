//! Outlet selectors and their ordinary props have distinct typed key identities.

use super::{
    Box, ElementNode, ExpressionNode, IRProp, PropNode, SimpleExpressionNode, SourceLocation,
    TransformContext, Vec,
};

pub(super) fn get_slot_outlet_name<'a>(
    ctx: &TransformContext<'a>,
    el: &ElementNode<'a>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) => {
                if attr.name == "name"
                    && let Some(ref value) = attr.value
                {
                    return Box::new_in(
                        SimpleExpressionNode::new(value.content, true, SourceLocation::STUB),
                        &ctx.allocator,
                    );
                }
            }
            PropNode::Directive(dir) => {
                if dir.name == "bind"
                    && let Some(ExpressionNode::Simple(arg)) = dir.arg.as_ref()
                    && arg.is_static
                    && arg.content == "name"
                    && let Some(ExpressionNode::Simple(exp)) = dir.exp.as_ref()
                {
                    return Box::new_in(SimpleExpressionNode::from_node(exp), &ctx.allocator);
                }
            }
        }
    }

    Box::new_in(
        SimpleExpressionNode::new("default", true, SourceLocation::STUB),
        &ctx.allocator,
    )
}

pub(super) fn get_slot_outlet_props<'a>(
    ctx: &TransformContext<'a>,
    el: &ElementNode<'a>,
) -> Vec<'a, IRProp<'a>> {
    let mut props = Vec::new_in(&ctx.allocator);

    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) => {
                if attr.name == "name" {
                    continue;
                }

                let key = Box::new_in(
                    SimpleExpressionNode::new(attr.name, true, SourceLocation::STUB),
                    &ctx.allocator,
                );
                let mut values = Vec::new_in(&ctx.allocator);
                if let Some(ref value) = attr.value {
                    values.push(Box::new_in(
                        SimpleExpressionNode::new(value.content, true, SourceLocation::STUB),
                        &ctx.allocator,
                    ));
                }

                props.push(IRProp::new(key, values, false));
            }
            PropNode::Directive(dir) => {
                if dir.name != "bind" {
                    continue;
                }

                match (dir.arg.as_ref(), dir.exp.as_ref()) {
                    (Some(ExpressionNode::Simple(arg)), Some(ExpressionNode::Simple(exp))) => {
                        if arg.is_static && arg.content == "name" {
                            continue;
                        }

                        let key = Box::new_in(SimpleExpressionNode::from_node(arg), &ctx.allocator);
                        let mut values = Vec::new_in(&ctx.allocator);
                        values.push(Box::new_in(
                            SimpleExpressionNode::from_node(exp),
                            &ctx.allocator,
                        ));

                        props.push(IRProp::new(key, values, false));
                    }
                    (None, Some(ExpressionNode::Simple(exp))) => {
                        let key = Box::new_in(
                            SimpleExpressionNode::new("$", true, SourceLocation::STUB),
                            &ctx.allocator,
                        );
                        let mut values = Vec::new_in(&ctx.allocator);
                        values.push(Box::new_in(
                            SimpleExpressionNode::from_node(exp),
                            &ctx.allocator,
                        ));

                        props.push(IRProp::new(key, values, false));
                    }
                    _ => {}
                }
            }
        }
    }

    props
}
