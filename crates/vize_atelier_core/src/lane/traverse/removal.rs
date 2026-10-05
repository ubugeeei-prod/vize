//! Complete deferred branch-gap removal after the active child borrow ends.

use crate::TemplateChildNode;
use vize_l0::Vec;

pub(super) fn remove_branch_whitespace<'a>(
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    index: &mut usize,
    from: usize,
) {
    // Only a successfully joined branch requests earlier removal.
    // Comments and every sibling outside that branch gap keep their ownership.
    let mut gap = *index;
    while gap > from {
        gap -= 1;
        if matches!(children.get(gap), Some(TemplateChildNode::Text(t)) if t.content.trim().is_empty())
        {
            children.remove(gap);
            *index -= 1;
        }
    }
}
