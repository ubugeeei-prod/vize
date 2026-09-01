use vize_davinci::id::NodeId;
use vize_s2::op::{BindingOp, ElementOp, Op};

use super::super::EmitCx;

pub(super) fn has_prop_bindings(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(|binding| {
        matches!(
            binding,
            BindingOp::Bind(_)
                | BindingOp::On(_)
                | BindingOp::Model(_)
                | BindingOp::VueHtml(_)
                | BindingOp::VueText(_)
        )
    })
}

pub(super) fn has_cloak(bindings: &[BindingOp<'_>]) -> bool {
    bindings
        .iter()
        .any(|binding| matches!(binding, BindingOp::VueCloak(_)))
}

pub(super) fn template_if_branch_root_has_direct_interpolation(
    cx: &EmitCx<'_>,
    element: &ElementOp<'_>,
    if_key: Option<&str>,
) -> bool {
    if if_key.is_none() || !cx.template_if_branch_root {
        return false;
    }
    has_direct_interpolation_child(element)
}

pub(super) fn has_direct_interpolation_child(element: &ElementOp<'_>) -> bool {
    element
        .children
        .ops
        .iter()
        .any(|op| matches!(op, Op::Interpolation(_)))
}

pub(super) fn has_interpolation_descendant(element: &ElementOp<'_>) -> bool {
    region_has_interpolation_descendant(&element.children.ops)
}

fn op_has_interpolation_descendant(op: &Op<'_>) -> bool {
    match op {
        Op::Interpolation(_) => true,
        Op::Element(element) => has_interpolation_descendant(element),
        Op::Component(component) => region_has_interpolation_descendant(&component.children.ops),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| region_has_interpolation_descendant(&branch.region.ops)),
        Op::For(for_op) => region_has_interpolation_descendant(&for_op.region.ops),
        Op::Slot(slot) => region_has_interpolation_descendant(&slot.fallback.ops),
        Op::Text(_) => false,
    }
}

fn region_has_interpolation_descendant(ops: &[Op<'_>]) -> bool {
    ops.iter().any(op_has_interpolation_descendant)
}

pub(super) fn has_dynamic_key_binding(element: &ElementOp<'_>) -> bool {
    element.bindings.iter().any(|binding| {
        matches!(
            binding,
            BindingOp::Bind(bind) if super::super::props_bind::is_key_bind_name(bind)
        )
    })
}

pub(super) fn direct_static_children_hoisted(
    cx: &EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
) -> bool {
    super::super::vnode_static::should_hoist_static_children(cx, element, id, true, false, false)
}
