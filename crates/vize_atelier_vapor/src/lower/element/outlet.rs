//! Outlet selectors and their ordinary props have distinct typed key identities.

use super::{
    BlockIRNode, Box, ElementNode, ExpressionNode, IRProp, OperationNode, PropNode,
    SimpleExpressionNode, SlotOutletIRNode, SourceLocation, String, TransformContext, Vec,
    transform_children,
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
    get_slot_props::<false>(ctx, el).0
}

pub(super) fn get_slot_outlet_props_and_scope<'a, 'b>(
    ctx: &TransformContext<'a>,
    el: &'b ElementNode<'a>,
) -> (
    Vec<'a, IRProp<'a>>,
    super::super::key::ScopeDirectives<'a, 'b>,
    Box<'a, SimpleExpressionNode<'a>>,
) {
    let (props, scope, name) = get_slot_props::<true>(ctx, el);
    let name = name.unwrap_or_else(|| {
        Box::new_in(
            SimpleExpressionNode::new("default", true, SourceLocation::STUB),
            &ctx.allocator,
        )
    });
    (props, scope, name)
}

fn get_slot_props<'a, 'b, const CLASSIFY: bool>(
    ctx: &TransformContext<'a>,
    el: &'b ElementNode<'a>,
) -> (
    Vec<'a, IRProp<'a>>,
    super::super::key::ScopeDirectives<'a, 'b>,
    Option<Box<'a, SimpleExpressionNode<'a>>>,
) {
    let mut scope = super::super::key::ScopeDirectives::new(ctx.is_key_non_reactive());
    let mut name = None;
    let mut props = Vec::new_in(&ctx.allocator);

    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) => {
                if attr.name == "name" {
                    if CLASSIFY
                        && name.is_none()
                        && let Some(ref value) = attr.value
                    {
                        name = Some(Box::new_in(
                            SimpleExpressionNode::new(value.content, true, SourceLocation::STUB),
                            &ctx.allocator,
                        ));
                    }
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
                    if CLASSIFY {
                        scope.observe(dir);
                    }
                    continue;
                }

                match (dir.arg.as_ref(), dir.exp.as_ref()) {
                    (Some(ExpressionNode::Simple(arg)), Some(ExpressionNode::Simple(exp))) => {
                        if arg.is_static && arg.content == "name" {
                            if CLASSIFY && name.is_none() {
                                name = Some(Box::new_in(
                                    SimpleExpressionNode::from_node(exp),
                                    &ctx.allocator,
                                ));
                            }
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

    (props, scope, name)
}

/// Slot props only copy original facts; scope must be applied before fallback effects.
pub(in crate::lower) fn transform_slot<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let (props, scope, name) = get_slot_outlet_props_and_scope(ctx, el);
    let (once, error, _) = scope.finish();
    if let Some(error) = error {
        ctx.push_diagnostic(String::from(error));
    }
    if once {
        ctx.enter_non_reactive_scope();
    }
    let id = ctx.next_id();
    let fallback = (!el.children.is_empty()).then(|| transform_children(ctx, &el.children));
    block
        .operation
        .push(OperationNode::SlotOutlet(SlotOutletIRNode {
            id,
            name,
            props,
            fallback,
            parent: None,
            anchor: None,
        }));
    block.returns.push(id);
    if once {
        ctx.exit_non_reactive_scope();
    }
}
