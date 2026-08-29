//! SFC style-block carriers are analysis facts, not DOM VNodes.

use vize_s2::op::{BindingOp, ElementOp, Op};

use super::EmitCx;

pub(super) fn is_carrier(op: &Op<'_>) -> bool {
    matches!(op, Op::Element(element) if is_carrier_element(element))
}

pub(super) fn is_carrier_element(element: &ElementOp<'_>) -> bool {
    element.tag == "style"
        && element.attributes.is_empty()
        && element.children.ops.is_empty()
        && !element.bindings.is_empty()
        && element
            .bindings
            .iter()
            .all(|binding| matches!(binding, BindingOp::VueCssBind(_)))
}

pub(super) fn skip_carrier(cx: &mut EmitCx<'_>, element: &ElementOp<'_>) {
    let _id = cx.walk.mint();
    cx.walk.skip(element.bindings.len());
}
