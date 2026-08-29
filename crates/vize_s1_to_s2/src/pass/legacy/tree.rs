//! Shared walks over region ops and attached binding lists.

use vize_s2::op::{BindingOp, Op};

use vize_s0::Vec;

/// Visit every attached binding list in document order (element,
/// component, slot outlet), then the owned regions.
pub(super) fn map_binding_lists<'a>(
    ops: &mut [Op<'a>],
    visit: &mut impl FnMut(&mut Vec<'a, BindingOp<'a>>),
) {
    for op in ops.iter_mut() {
        match op {
            Op::Element(element) => {
                visit(&mut element.bindings);
                map_binding_lists(&mut element.children.ops, visit);
            }
            Op::Component(component) => {
                visit(&mut component.bindings);
                map_binding_lists(&mut component.children.ops, visit);
            }
            Op::Slot(slot) => {
                visit(&mut slot.bindings);
                map_binding_lists(&mut slot.fallback.ops, visit);
            }
            Op::If(if_op) => {
                for branch in if_op.branches.iter_mut() {
                    map_binding_lists(&mut branch.region.ops, visit);
                }
            }
            Op::For(for_op) => map_binding_lists(&mut for_op.region.ops, visit),
            Op::Text(_) | Op::Interpolation(_) => {}
        }
    }
}
