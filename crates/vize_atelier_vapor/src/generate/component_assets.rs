//! Asset components resolve once at the top of the render, as upstream's
//! `genBlockContent` does: in the order their elements finish transforming
//! (children before the component that holds them). A resolution inside a
//! loop row, branch or slot would repeat per render of that block, and a
//! slot body runs under the child, which resolves against its own assets.
//! A component created once, directly in the root block, still resolves
//! where it is created (upstream's single-use `createAssetComponent`).

use crate::ir::{BlockIRNode, ComponentKind, NegativeBranch, OperationNode};
use vize_carton::{SmallVec, ensure_sufficient_stack};

/// Inline for the few asset components a template uses.
type Usages<'a> = SmallVec<[Usage<'a>; 4]>;

struct Usage<'a> {
    tag: &'a str,
    count: usize,
    root: bool,
}

/// The tags to resolve before the render's first operation.
pub(super) fn hoisted<'a>(block: &BlockIRNode<'a>) -> impl Iterator<Item = &'a str> {
    let mut usages = Usages::new();
    visit_block(block, true, &mut usages);
    usages
        .into_iter()
        .filter(|usage| usage.count > 1 || !usage.root)
        .map(|usage| usage.tag)
}

fn visit_block<'a>(block: &BlockIRNode<'a>, root: bool, usages: &mut Usages<'a>) {
    ensure_sufficient_stack(|| {
        for operation in block.operation.iter() {
            visit_operation(operation, root, usages);
        }
        for effect in block.effect.iter() {
            for operation in effect.operations.iter() {
                visit_operation(operation, false, usages);
            }
        }
    });
}

fn visit_negative<'a>(branch: &NegativeBranch<'a>, usages: &mut Usages<'a>) {
    match branch {
        NegativeBranch::Block(block) => visit_block(block, false, usages),
        NegativeBranch::If(node) => {
            visit_block(&node.positive, false, usages);
            if let Some(negative) = &node.negative {
                visit_negative(negative, usages);
            }
        }
    }
}

fn visit_operation<'a>(operation: &OperationNode<'a>, root: bool, usages: &mut Usages<'a>) {
    match operation {
        OperationNode::CreateComponent(component) => {
            for slot in component.slots.iter() {
                for block in slot.blocks() {
                    visit_block(block, false, usages);
                }
            }
            if component.kind != ComponentKind::Regular {
                return;
            }
            match usages.iter_mut().find(|usage| usage.tag == component.tag) {
                Some(usage) => {
                    usage.count += 1;
                    usage.root |= root;
                }
                None => usages.push(Usage {
                    tag: component.tag,
                    count: 1,
                    root,
                }),
            }
        }
        OperationNode::If(node) => {
            visit_block(&node.positive, false, usages);
            if let Some(negative) = &node.negative {
                visit_negative(negative, usages);
            }
        }
        OperationNode::For(node) => visit_block(&node.render, false, usages),
        OperationNode::Key(node) => visit_block(&node.render, false, usages),
        OperationNode::SlotOutlet(outlet) => {
            if let Some(fallback) = &outlet.fallback {
                visit_block(fallback, false, usages);
            }
        }
        _ => {}
    }
}
