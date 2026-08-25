//! v-if transform for Vapor mode.
//!
//! Transforms v-if, v-else-if, v-else directives into IfIRNode.

use vize_carton::{Allocator, Box};

use crate::ir::{BlockIRNode, IfIRNode, NegativeBranch, OperationNode};
use vize_atelier_core::{
    DirectiveNode, ElementNode, ExpressionNode, IfBranchNode, SimpleExpressionNode, SourceLocation,
};

/// Transform v-if directive to IR
pub fn transform_v_if<'a>(
    allocator: &'a Allocator,
    dir: &DirectiveNode<'a>,
    _el: &ElementNode<'a>,
    children_block: BlockIRNode<'a>,
    id: usize,
    source: &'a str,
) -> OperationNode<'a> {
    let condition = extract_condition(allocator, dir, source);

    let if_node = IfIRNode {
        id,
        condition,
        positive: children_block,
        negative: None,
        once: false,
        parent: None,
        anchor: None,
    };

    OperationNode::If(Box::new_in(if_node, &allocator))
}

/// Transform IfBranchNode to IfIRNode
pub fn transform_if_branches<'a>(
    allocator: &'a Allocator,
    branches: &[IfBranchNode<'a>],
    transform_children: impl Fn(
        &'a Allocator,
        &[vize_atelier_core::TemplateChildNode<'a>],
    ) -> BlockIRNode<'a>,
    source: &'a str,
    id_generator: &mut impl FnMut() -> usize,
) -> Option<OperationNode<'a>> {
    if branches.is_empty() {
        return None;
    }

    let first_branch = &branches[0];
    let condition = if let Some(ref cond) = first_branch.condition {
        extract_expression(allocator, cond, source)
    } else {
        Box::new_in(
            SimpleExpressionNode::new("true", false, SourceLocation::STUB),
            &allocator,
        )
    };

    let positive = transform_children(allocator, &first_branch.children);

    let negative = if branches.len() > 1 {
        Some(transform_remaining_branches(
            allocator,
            &branches[1..],
            &transform_children,
            source,
            id_generator,
        ))
    } else {
        None
    };

    let if_node = IfIRNode {
        id: id_generator(),
        condition,
        positive,
        negative,
        once: false,
        parent: None,
        anchor: None,
    };

    Some(OperationNode::If(Box::new_in(if_node, &allocator)))
}

/// Transform remaining branches (v-else-if, v-else)
fn transform_remaining_branches<'a>(
    allocator: &'a Allocator,
    branches: &[IfBranchNode<'a>],
    transform_children: &impl Fn(
        &'a Allocator,
        &[vize_atelier_core::TemplateChildNode<'a>],
    ) -> BlockIRNode<'a>,
    source: &'a str,
    id_generator: &mut impl FnMut() -> usize,
) -> NegativeBranch<'a> {
    if branches.is_empty() {
        return NegativeBranch::Block(BlockIRNode::new(allocator));
    }

    let branch = &branches[0];

    if let Some(ref cond) = branch.condition {
        // v-else-if
        let condition = extract_expression(allocator, cond, source);
        let positive = transform_children(allocator, &branch.children);

        let negative = if branches.len() > 1 {
            Some(transform_remaining_branches(
                allocator,
                &branches[1..],
                transform_children,
                source,
                id_generator,
            ))
        } else {
            None
        };

        let nested_if = IfIRNode {
            id: id_generator(),
            condition,
            positive,
            negative,
            once: false,
            parent: None,
            anchor: None,
        };

        NegativeBranch::If(Box::new_in(nested_if, &allocator))
    } else {
        // v-else
        NegativeBranch::Block(transform_children(allocator, &branch.children))
    }
}

/// Extract condition from directive
fn extract_condition<'a>(
    allocator: &'a Allocator,
    dir: &DirectiveNode<'a>,
    source: &'a str,
) -> Box<'a, SimpleExpressionNode<'a>> {
    if let Some(ref exp) = dir.exp {
        extract_expression(allocator, exp, source)
    } else {
        Box::new_in(
            SimpleExpressionNode::new("true", false, SourceLocation::STUB),
            &allocator,
        )
    }
}

/// Extract expression from ExpressionNode
fn extract_expression<'a>(
    allocator: &'a Allocator,
    exp: &ExpressionNode<'a>,
    source: &'a str,
) -> Box<'a, SimpleExpressionNode<'a>> {
    match exp {
        ExpressionNode::Simple(simple) => {
            let node =
                SimpleExpressionNode::new(simple.content, simple.is_static, simple.loc.clone());
            Box::new_in(node, &allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node = SimpleExpressionNode::new(
                compound.loc.span.slice(source),
                false,
                compound.loc.clone(),
            );
            Box::new_in(node, &allocator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::extract_condition;
    use vize_atelier_core::{DirectiveNode, SourceLocation};
    use vize_carton::Allocator;

    #[test]
    fn test_extract_condition() {
        let allocator = Allocator::new();
        let dir = DirectiveNode::new(&allocator, "if", SourceLocation::STUB);

        let condition = extract_condition(&allocator, &dir, "");
        assert_eq!(condition.content, "true");
    }
}
