//! Collect single render roots without assigning sibling or nested fallthrough.

use crate::ir::{BlockIRNode, IfIRNode, NegativeBranch, OperationNode};
use vize_carton::{FxHashSet, ensure_sufficient_stack};

/// Collect the nodes that are a render's single root, as Vue's compiler decides
/// it: the block's only return and, through a returned `v-if` or keyed fragment,
/// the single return of each branch or of the keyed block.
pub(super) fn collect_single_root_returns(block: &BlockIRNode<'_>, roots: &mut FxHashSet<usize>) {
    ensure_sufficient_stack(|| {
        let [root_id] = block.returns.as_slice() else {
            return;
        };
        roots.insert(*root_id);
        // A block also holds the operations of its nested elements, so only the
        // operation that is itself the single return carries the root inward.
        for op in block.operation.iter() {
            match op {
                OperationNode::If(if_node) if if_node.id == *root_id => {
                    collect_if_branch_roots(if_node, roots);
                }
                OperationNode::Key(key) if key.id == *root_id => {
                    collect_single_root_returns(&key.render, roots);
                }
                _ => {}
            }
        }
    });
}

/// Collect the single root of every branch in a root `v-if` chain.
fn collect_if_branch_roots(if_node: &IfIRNode<'_>, roots: &mut FxHashSet<usize>) {
    ensure_sufficient_stack(|| {
        collect_single_root_returns(&if_node.positive, roots);
        match &if_node.negative {
            Some(NegativeBranch::Block(block)) => collect_single_root_returns(block, roots),
            Some(NegativeBranch::If(nested_if)) => collect_if_branch_roots(nested_if, roots),
            None => {}
        }
    });
}
