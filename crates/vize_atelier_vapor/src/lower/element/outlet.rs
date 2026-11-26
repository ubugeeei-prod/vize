//! Outlet selectors and their ordinary props have distinct typed key identities.

use super::{
    BlockIRNode, Box, ElementNode, ExpressionNode, IRProp, OperationNode, PropNode,
    SimpleExpressionNode, SlotOutletIRNode, SourceLocation, TransformContext, Vec,
    transform_children,
};
use crate::ir::{SlotLayout, finish_slot_fallback};

/// Lower outlet fallback content, finishing its boundary inside slot content.
pub(super) fn lower_fallback<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
) -> Option<Box<'a, BlockIRNode<'a>>> {
    if el.children.is_empty() {
        return None;
    }
    let mut fallback = transform_children(ctx, &el.children);
    if ctx.slot_content_depth > 0 {
        let layout = SlotLayout::new(&ctx.templates, &ctx.element_template_map);
        finish_slot_fallback(&mut fallback, &layout);
    }
    Some(Box::new_in(fallback, &ctx.allocator))
}

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
                if is_template_ref_prop(attr.name) {
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
                        if arg.is_static && is_template_ref_prop(arg.content) {
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

/// A template ref on an outlet targets the rendered slot fragment, so it is
/// registered for the outlet instead of being passed to the slot as a prop.
fn is_template_ref_prop(name: &str) -> bool {
    matches!(name, "ref" | "ref_for" | "ref_key")
}

/// Slot props only copy original facts; scope must be applied before fallback effects.
pub(in crate::lower) fn transform_slot<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let (props, scope, name) = get_slot_outlet_props_and_scope(ctx, el);
    let (once, warning, _) = scope.finish();
    if let Some(warning) = warning {
        ctx.push_warning(warning);
    }
    if once {
        ctx.enter_non_reactive_scope();
    }
    let id = ctx.next_id();
    let fallback = lower_fallback(ctx, el);
    block
        .operation
        .push(OperationNode::SlotOutlet(SlotOutletIRNode {
            flags: 0,
            id,
            name,
            props,
            fallback,
            parent: None,
            anchor: None,
        }));
    super::template::transform_template_ref(ctx, el, id, block);
    block.returns.push(id);
    if once {
        ctx.exit_non_reactive_scope();
    }
}
