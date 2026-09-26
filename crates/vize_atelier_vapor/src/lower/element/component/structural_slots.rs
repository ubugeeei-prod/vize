//! Retained compatibility lowering mirrors the checked structural slot IR.

use super::{TransformContext, slots::resolve_named_slot, transform_children};
use crate::ir::{IRSlot, IRSlotControl, IRSlotLoop};
use vize_atelier_core::{
    ElementNode, ExpressionNode, IfBranchNode, PropNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode,
};
use vize_carton::Box;

fn carrier<'s, 'a>(node: &'s TemplateChildNode<'a>) -> Option<&'s ElementNode<'a>> {
    match node {
        TemplateChildNode::Element(element)
            if element.tag == "template"
                && element.props.iter().any(|prop| {
                    matches!(prop,
                PropNode::Directive(dir) if dir.name == "slot")
                }) =>
        {
            Some(element)
        }
        _ => None,
    }
}

pub(super) fn is_slot(node: &TemplateChildNode<'_>) -> bool {
    carrier(node).is_some()
        || match node {
            TemplateChildNode::If(node) => node.branches.iter().all(
                |branch| matches!(branch.children.as_slice(), [child] if carrier(child).is_some()),
            ),
            TemplateChildNode::For(node) => matches!(node.children.as_slice(),
            [child] if carrier(child).is_some()),
            _ => false,
        }
}

fn expression<'a>(
    ctx: &TransformContext<'a>,
    expr: &ExpressionNode<'a>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    let node = match expr {
        ExpressionNode::Simple(node) => SimpleExpressionNode::from_node(node),
        ExpressionNode::Compound(node) => {
            SimpleExpressionNode::new(node.loc.span.slice(ctx.source), false, node.loc.clone())
        }
    };
    Box::new_in(node, &ctx.allocator)
}

pub(super) fn lower<'a>(
    ctx: &mut TransformContext<'a>,
    child: &TemplateChildNode<'a>,
) -> Option<IRSlot<'a>> {
    if let Some(element) = carrier(child) {
        return template(ctx, element);
    }
    match child {
        TemplateChildNode::If(node) => {
            let previous = ctx.structural_slot_spans;
            ctx.structural_slot_spans = true;
            let slot = conditional(ctx, &node.branches);
            ctx.structural_slot_spans = previous;
            slot
        }
        TemplateChildNode::For(node) => {
            let [child] = node.children.as_slice() else {
                return None;
            };
            let previous = ctx.structural_slot_spans;
            ctx.structural_slot_spans = true;
            let slot = lower(ctx, child);
            ctx.structural_slot_spans = previous;
            let mut slot = slot?;
            let value = expression(ctx, node.value_alias.as_ref()?);
            let key_prop = carrier(child)?.props.iter().find_map(|prop| match prop {
                PropNode::Directive(dir)
                    if dir.name == "bind"
                        && matches!(&dir.arg, Some(ExpressionNode::Simple(arg))
                        if arg.is_static && arg.content == "key") =>
                {
                    dir.exp.as_ref().map(|value| expression(ctx, value))
                }
                _ => None,
            });
            slot.control = Some(IRSlotControl::For(IRSlotLoop {
                source: expression(ctx, &node.source),
                value,
                key: node.key_alias.as_ref().map(|key| expression(ctx, key)),
                index: node
                    .object_index_alias
                    .as_ref()
                    .map(|index| expression(ctx, index)),
                key_prop,
            }));
            Some(slot)
        }
        _ => None,
    }
}

fn conditional<'a>(
    ctx: &mut TransformContext<'a>,
    branches: &[IfBranchNode<'a>],
) -> Option<IRSlot<'a>> {
    let (branch, rest) = branches.split_first()?;
    let [child] = branch.children.as_slice() else {
        return None;
    };
    let mut slot = lower(ctx, child)?;
    if let Some(condition) = &branch.condition {
        slot.control = Some(IRSlotControl::If {
            condition: expression(ctx, condition),
            negative: conditional(ctx, rest).map(|slot| Box::new_in(slot, &ctx.allocator)),
        });
    }
    Some(slot)
}

fn template<'a>(ctx: &mut TransformContext<'a>, element: &ElementNode<'a>) -> Option<IRSlot<'a>> {
    let dir = element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "slot" => Some(dir.as_ref()),
        _ => None,
    })?;
    let (name, is_static) = resolve_named_slot(dir);
    let name_loc = dir
        .arg
        .as_ref()
        .map_or(SourceLocation::STUB, |arg| arg.loc().clone());
    let name = SimpleExpressionNode::new(ctx.allocator.alloc_str(&name), is_static, name_loc);
    let fn_exp = dir.exp.as_ref().map(|params| expression(ctx, params));
    let block = transform_children(ctx, &element.children);
    ctx.next_id();
    Some(IRSlot {
        name: Box::new_in(name, &ctx.allocator),
        fn_exp,
        block,
        control: None,
    })
}
