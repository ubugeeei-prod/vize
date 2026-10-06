//! Core transfers a conditional element's original key into its branch owner.

use vize_atelier_core::{
    ExpressionNode, IfBranchNode, PropNode, SimpleExpressionNode, TemplateChildNode,
};
use vize_carton::Box;

use super::{
    super::{context::TransformContext, transform_children},
    eligible, eligible_target,
};
use crate::ir::{BlockIRNode, KeyIRNode, OperationNode};

pub(in crate::lower) fn transform_branch<'a>(
    ctx: &mut TransformContext<'a>,
    branch: &IfBranchNode<'a>,
) -> BlockIRNode<'a> {
    let value = match (branch.children.as_slice(), branch.user_key.as_ref()) {
        ([TemplateChildNode::Element(el)], Some(PropNode::Directive(dir)))
            if !branch.is_template_if
                && eligible_target(el)
                && dir.name == "bind"
                && matches!(dir.arg.as_ref(), Some(ExpressionNode::Simple(arg))
                    if arg.is_static && arg.content == "key") =>
        {
            match dir.exp.as_ref() {
                Some(ExpressionNode::Simple(value))
                    if eligible(el, value, ctx.is_key_non_reactive()) =>
                {
                    Some(value.as_ref())
                }
                _ => None,
            }
        }
        _ => None,
    };
    let Some(value) = value else {
        return transform_children(ctx, &branch.children);
    };
    let id = ctx.next_id();
    let render = transform_children(ctx, &branch.children);
    let mut block = BlockIRNode::new(ctx.allocator);
    block.operation.push(OperationNode::Key(Box::new_in(
        KeyIRNode {
            id,
            value: Box::new_in(SimpleExpressionNode::from_node(value), &ctx.allocator),
            render,
            parent: None,
            anchor: None,
        },
        &ctx.allocator,
    )));
    block.returns.push(id);
    block
}
