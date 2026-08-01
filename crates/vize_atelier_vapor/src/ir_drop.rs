//! Stack-safe teardown for the recursive Vapor IR graph.

use crate::ir::{BlockIRNode, IRDynamicInfo, NegativeBranch, OperationNode, RootIRNode};
use vize_carton::Box;

/// Drop a completed Vapor IR without following nested blocks on the machine stack.
///
/// Patterned templates lower to nested `If` operations. Ordinary derived drop
/// follows that graph recursively, so sufficiently deep valid input can abort a
/// small-stack compiler worker after code generation has already completed.
/// This consumes every recursive edge into an explicit heap worklist first;
/// the remaining values are shallow when their normal destructors run.
pub fn drop_ir_stack_safe(mut ir: RootIRNode<'_>) {
    let mut operations = std::vec::Vec::new();
    let mut dynamics = std::vec::Vec::new();
    drain_block(&mut ir.block, &mut operations, &mut dynamics);

    while let Some(operation) = operations.pop() {
        match operation {
            OperationNode::If(node) => {
                let mut node = Box::into_inner(node);
                drain_block(&mut node.positive, &mut operations, &mut dynamics);
                if let Some(negative) = node.negative.take() {
                    match negative {
                        NegativeBranch::Block(mut block) => {
                            drain_block(&mut block, &mut operations, &mut dynamics);
                        }
                        NegativeBranch::If(node) => operations.push(OperationNode::If(node)),
                    }
                }
            }
            OperationNode::For(node) => {
                let mut node = Box::into_inner(node);
                drain_block(&mut node.render, &mut operations, &mut dynamics);
            }
            OperationNode::CreateComponent(mut node) => {
                while let Some(mut slot) = node.slots.pop() {
                    drain_block(&mut slot.block, &mut operations, &mut dynamics);
                }
            }
            OperationNode::SlotOutlet(mut node) => {
                if let Some(mut fallback) = node.fallback.take() {
                    drain_block(&mut fallback, &mut operations, &mut dynamics);
                }
            }
            _ => {}
        }
    }

    while let Some(mut dynamic) = dynamics.pop() {
        dynamics.append(&mut dynamic.children);
    }
}

fn drain_block<'a>(
    block: &mut BlockIRNode<'a>,
    operations: &mut std::vec::Vec<OperationNode<'a>>,
    dynamics: &mut std::vec::Vec<IRDynamicInfo>,
) {
    while let Some(operation) = block.operation.pop() {
        operations.push(operation);
    }
    while let Some(mut effect) = block.effect.pop() {
        while let Some(operation) = effect.operations.pop() {
            operations.push(operation);
        }
    }
    dynamics.push(std::mem::take(&mut block.dynamic));
}
