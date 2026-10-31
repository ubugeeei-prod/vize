//! Forwarded slot-outlet discovery with the existing recursive stack guard.

use crate::{ElementNode, ElementType, TemplateChildNode};
use vize_l0::ensure_sufficient_stack;

/// Check whether this component forwards an incoming slot to another component,
/// e.g. `<Inner><slot /></Inner>`.
pub fn has_forwarded_slot_outlet(el: &ElementNode<'_>) -> bool {
    el.children.iter().any(child_contains_slot_outlet)
}

fn child_contains_slot_outlet(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => {
            if el.tag_type == ElementType::Slot || el.tag == "slot" {
                return true;
            }
            ensure_sufficient_stack(|| el.children.iter().any(child_contains_slot_outlet))
        }
        TemplateChildNode::If(if_node) => if_node
            .branches
            .iter()
            .flat_map(|branch| branch.children.iter())
            .any(|child| ensure_sufficient_stack(|| child_contains_slot_outlet(child))),
        TemplateChildNode::For(for_node) => for_node
            .children
            .iter()
            .any(|child| ensure_sufficient_stack(|| child_contains_slot_outlet(child))),
        _ => false,
    }
}
