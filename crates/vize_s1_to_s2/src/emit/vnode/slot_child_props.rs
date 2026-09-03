//! The scoped-slot child-props exception the vnode hoist gate consults.
//! Split out of `vnode.rs` to keep it inside the source budget.

use vize_s2::op::{BindingOp, ElementOp, Op};

use super::super::EmitCx;
use super::super::props_static::PropHoistPosition;

pub(super) fn scoped_for_slot_component_slot_child_props(
    cx: &EmitCx<'_>,
    element: &ElementOp<'_>,
    prop_hoist: PropHoistPosition,
) -> bool {
    matches!(prop_hoist, PropHoistPosition::Nested)
        && cx.slot_param_depth > 0
        && element.bindings.is_empty()
        && matches!(
            element.children.ops.as_slice(),
            [Op::Component(component)]
                if component
                    .bindings
                    .iter()
                    .any(|binding| matches!(binding, BindingOp::SlotContent(_)))
        )
}
