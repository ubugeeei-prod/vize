use crate::ir::{BlockIRNode, OperationNode};
use vize_carton::{cstr, String};

use super::super::context::GenerateContext;

pub(super) fn emit_insertion_state(
    ctx: &mut GenerateContext,
    parent: Option<usize>,
    anchor: Option<usize>,
) {
    let Some(parent_id) = parent else {
        return;
    };
    ctx.use_helper("setInsertionState");
    let anchor_expr = anchor
        .map(|anchor_id| cstr!("n{}", anchor_id))
        .unwrap_or_else(|| String::from("null"));
    ctx.push_line(&cstr!(
        "_setInsertionState(n{}, {}, true)",
        parent_id,
        anchor_expr
    ));
}

pub(super) fn block_requires_parent_insertion_state(block: &BlockIRNode<'_>) -> bool {
    block.operation.iter().any(|op| match op {
        OperationNode::If(if_node) => if_node.parent.is_none(),
        OperationNode::For(for_node) => for_node.parent.is_none(),
        OperationNode::CreateComponent(component) => component.parent.is_none(),
        OperationNode::SlotOutlet(_) => true,
        _ => false,
    })
}
