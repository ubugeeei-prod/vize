//! Static child vnode hoist gates for native element emission.

use vize_davinci::id::NodeId;
use vize_s2::op::{ElementOp, Namespace, Op};

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
        if has_direct_component_child(element) {
            return true;
        }
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

fn has_direct_component_child(element: &ElementOp<'_>) -> bool {
    element
        .children
        .ops
        .iter()
        .any(|op| matches!(op, Op::Component(_)))
}

pub(super) fn can_whole_hoist_static_element(element: &ElementOp<'_>) -> bool {
    if element.namespace != Namespace::Html
        && !element.bindings.is_empty()
        && element
            .children
            .ops
            .iter()
            .any(|op| matches!(op, Op::Element(_)))
    {
        return false;
    }
    super::props_static::static_vnode_surface_can_hoist(&element.attributes, &element.bindings)
        && element
            .children
            .ops
            .iter()
            .all(can_whole_hoist_static_child)
}

fn can_whole_hoist_static_child(op: &Op<'_>) -> bool {
    match op {
        Op::Text(_) => true,
        Op::Element(element) => can_whole_hoist_static_element(element),
        _ => false,
    }
}
