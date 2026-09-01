//! Static child vnode hoist gates for native element emission.

use vize_davinci::id::NodeId;
use vize_s2::op::{ElementOp, Op};

use super::EmitCx;
use crate::pass::StaticLevel;

pub(super) fn should_hoist_static_children(
    cx: &EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
    allow_hoist: bool,
    branch_root: bool,
    for_item: bool,
) -> bool {
    if cx.conditional_v_for_item {
        return false;
    }
    if branch_root && cx.template_if_branch_root && has_direct_interpolation_child(element) {
        return false;
    }
    let requested =
        cx.hoist_static_vnodes || (allow_hoist && (branch_root || !element.bindings.is_empty()));
    if !requested {
        return false;
    }
    if branch_root || for_item {
        return true;
    }
    id.and_then(|id| cx.facts.static_facts.get(id))
        .is_some_and(|fact| fact.level == StaticLevel::NotStatic)
}

fn has_direct_interpolation_child(element: &ElementOp<'_>) -> bool {
    element
        .children
        .ops
        .iter()
        .any(|op| matches!(op, Op::Interpolation(_)))
}
