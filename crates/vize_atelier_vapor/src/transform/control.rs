//! Control flow transformation (v-if, v-for).
//!
//! Handles `IfNode` and `ForNode` from the template AST.

use vize_carton::Box;

use crate::ir::{BlockIRNode, ForIRNode, IfIRNode, NegativeBranch, OperationNode};
use vize_atelier_core::{
    ExpressionNode, ForNode, IfNode, PropNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode,
};

use super::{context::TransformContext, transform_children};

/// Transform IfNode (from compiler-core v-if transform)
pub(crate) fn transform_if_node<'a>(
    ctx: &mut TransformContext<'a>,
    if_node: &IfNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    transform_if_node_with_options(ctx, if_node, block, None, None, true);
}

pub(crate) fn transform_if_node_into_parent<'a>(
    ctx: &mut TransformContext<'a>,
    if_node: &IfNode<'a>,
    block: &mut BlockIRNode<'a>,
    parent: usize,
) {
    transform_if_node_with_options(ctx, if_node, block, Some(parent), None, false);
}

fn transform_if_node_with_options<'a>(
    ctx: &mut TransformContext<'a>,
    if_node: &IfNode<'a>,
    block: &mut BlockIRNode<'a>,
    parent: Option<usize>,
    anchor: Option<usize>,
    add_return: bool,
) {
    if if_node.branches.is_empty() {
        return;
    }

    // Allocate ID for the if node itself
    let if_id = ctx.next_id();

    // First branch is the v-if condition
    let first_branch = &if_node.branches[0];

    // Get condition from first branch
    let condition = if let Some(ref cond) = first_branch.condition {
        match cond {
            ExpressionNode::Simple(simple) => {
                let cond_node = SimpleExpressionNode::new(
                    simple.content.clone(),
                    simple.is_static,
                    simple.loc.clone(),
                );
                Box::new_in(cond_node, ctx.allocator)
            }
            ExpressionNode::Compound(compound) => {
                let cond_node = SimpleExpressionNode::new(
                    compound.loc.source.clone(),
                    false,
                    compound.loc.clone(),
                );
                Box::new_in(cond_node, ctx.allocator)
            }
        }
    } else {
        // No condition means v-else, which shouldn't be the first branch
        let cond_node = SimpleExpressionNode::new("true", false, SourceLocation::STUB);
        Box::new_in(cond_node, ctx.allocator)
    };

    // Consume an ID for the positive branch block
    let _positive_branch_id = ctx.next_id();

    // Transform first branch children
    let positive = transform_children(ctx, &first_branch.children);

    // Handle remaining branches (v-else-if, v-else)
    let negative = if if_node.branches.len() > 1 {
        Some(transform_remaining_branches(
            ctx,
            &if_node.branches[1..],
            parent,
            anchor,
        ))
    } else {
        None
    };

    let ir_if = IfIRNode {
        id: if_id,
        condition,
        positive,
        negative,
        once: false,
        parent,
        anchor,
    };

    block
        .operation
        .push(OperationNode::If(Box::new_in(ir_if, ctx.allocator)));
    if add_return {
        block.returns.push(if_id);
    }
}

/// Transform remaining if branches (v-else-if, v-else)
pub(crate) fn transform_remaining_branches<'a>(
    ctx: &mut TransformContext<'a>,
    branches: &[vize_atelier_core::IfBranchNode<'a>],
    parent: Option<usize>,
    anchor: Option<usize>,
) -> NegativeBranch<'a> {
    if branches.is_empty() {
        // This shouldn't happen, but return an empty block just in case
        return NegativeBranch::Block(BlockIRNode::new(ctx.allocator));
    }

    let branch = &branches[0];

    if let Some(ref cond) = branch.condition {
        // v-else-if: create nested IfIRNode
        // Note: v-else-if is inline, so it doesn't consume its own ID

        let condition = match cond {
            ExpressionNode::Simple(simple) => {
                let cond_node = SimpleExpressionNode::new(
                    simple.content.clone(),
                    simple.is_static,
                    simple.loc.clone(),
                );
                Box::new_in(cond_node, ctx.allocator)
            }
            ExpressionNode::Compound(compound) => {
                let cond_node = SimpleExpressionNode::new(
                    compound.loc.source.clone(),
                    false,
                    compound.loc.clone(),
                );
                Box::new_in(cond_node, ctx.allocator)
            }
        };

        // Consume ID for positive branch block
        let _positive_branch_id = ctx.next_id();

        let positive = transform_children(ctx, &branch.children);

        let negative = if branches.len() > 1 {
            // Consume ID for negative branch callback block
            let _negative_block_id = ctx.next_id();
            Some(transform_remaining_branches(
                ctx,
                &branches[1..],
                parent,
                anchor,
            ))
        } else {
            None
        };

        let nested_if = IfIRNode {
            id: 0, // Not used for inline v-else-if
            condition,
            positive,
            negative,
            once: false,
            parent,
            anchor,
        };

        NegativeBranch::If(Box::new_in(nested_if, ctx.allocator))
    } else {
        // v-else: consume ID for the else branch block
        let _else_branch_id = ctx.next_id();
        NegativeBranch::Block(transform_children(ctx, &branch.children))
    }
}

/// Transform ForNode (from compiler-core v-for transform)
pub(crate) fn transform_for_node<'a>(
    ctx: &mut TransformContext<'a>,
    for_node: &ForNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    transform_for_node_with_options(ctx, for_node, block, None, None, true);
}

pub(crate) fn transform_for_node_into_parent<'a>(
    ctx: &mut TransformContext<'a>,
    for_node: &ForNode<'a>,
    block: &mut BlockIRNode<'a>,
    parent: usize,
) {
    transform_for_node_with_options(ctx, for_node, block, Some(parent), None, false);
}

fn transform_for_node_with_options<'a>(
    ctx: &mut TransformContext<'a>,
    for_node: &ForNode<'a>,
    block: &mut BlockIRNode<'a>,
    parent: Option<usize>,
    anchor: Option<usize>,
    add_return: bool,
) {
    // Allocate for-node ID first (before children consume IDs)
    let for_id = ctx.next_id();

    // Get source expression
    let source = clone_simple_expr(ctx, &for_node.source);

    // Get value alias
    let value = for_node
        .value_alias
        .as_ref()
        .map(|v| clone_simple_expr(ctx, v));

    // Get key alias
    let key = for_node
        .key_alias
        .as_ref()
        .map(|k| clone_simple_expr(ctx, k));

    // Get index alias
    let index = for_node
        .object_index_alias
        .as_ref()
        .map(|i| clone_simple_expr(ctx, i));

    // Extract :key from the first child element's props
    let key_prop = extract_key_prop(ctx, for_node);

    // Consume ID for the render block
    let _render_block_id = ctx.next_id();

    // Transform children as render block
    let render = transform_children(ctx, &for_node.children);

    let ir_for = ForIRNode {
        id: for_id,
        source,
        value,
        key,
        index,
        key_prop,
        render,
        once: false,
        component: false,
        only_child: for_node.children.len() == 1,
        parent,
        anchor,
    };

    block
        .operation
        .push(OperationNode::For(Box::new_in(ir_for, ctx.allocator)));
    if add_return {
        block.returns.push(for_id);
    }
}

/// Clone an ExpressionNode into a SimpleExpressionNode
fn clone_simple_expr<'a>(
    ctx: &mut TransformContext<'a>,
    expr: &ExpressionNode<'a>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    match expr {
        ExpressionNode::Simple(simple) => {
            let node = SimpleExpressionNode::new(
                simple.content.clone(),
                simple.is_static,
                simple.loc.clone(),
            );
            Box::new_in(node, ctx.allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node =
                SimpleExpressionNode::new(compound.loc.source.clone(), false, compound.loc.clone());
            Box::new_in(node, ctx.allocator)
        }
    }
}

/// Extract :key prop from the first child element of a v-for node
fn extract_key_prop<'a>(
    ctx: &mut TransformContext<'a>,
    for_node: &ForNode<'a>,
) -> Option<Box<'a, SimpleExpressionNode<'a>>> {
    // Look at the first child element for a :key directive
    for child in for_node.children.iter() {
        if let TemplateChildNode::Element(el) = child {
            for prop in el.props.iter() {
                if let PropNode::Directive(dir) = prop {
                    if dir.name.as_str() == "bind" {
                        if let Some(ref arg) = dir.arg {
                            if let ExpressionNode::Simple(key_arg) = arg {
                                if key_arg.content.as_str() == "key" {
                                    if let Some(ref exp) = dir.exp {
                                        if let ExpressionNode::Simple(s) = exp {
                                            let node = SimpleExpressionNode::new(
                                                s.content.clone(),
                                                s.is_static,
                                                s.loc.clone(),
                                            );
                                            return Some(Box::new_in(node, ctx.allocator));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
