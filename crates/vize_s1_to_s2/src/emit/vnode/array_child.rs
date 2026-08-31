use vize_s0::ensure_sufficient_stack;
use vize_s2::op::Op;

use super::super::{EmitCx, EmitError, UnsupportedReason as Reason};

pub(in crate::emit) fn emit_array_child(
    cx: &mut EmitCx<'_>,
    op: &Op<'_>,
    hoist_static_children: bool,
    cache_static_children: bool,
) -> Result<(), EmitError> {
    let hoist_static_children = hoist_static_children || cx.hoist_static_vnodes;
    if hoist_static_children
        && let Op::Element(element) = op
        && super::super::hoist::is_hoistable(element)
    {
        return super::super::hoist::emit_hoisted_element(cx, element);
    }
    if cache_static_children
        && let Op::Element(element) = op
        && super::super::hoist::is_hoistable(element)
    {
        return super::super::hoist::emit_cached_element(cx, element);
    }
    let id = cx.walk.mint();
    cx.with_static_vnode_hoist(hoist_static_children, |cx| {
        ensure_sufficient_stack(|| match op {
            Op::Element(element) if super::super::slots::is_slot_template(element) => {
                cx.walk.skip(element.bindings.len());
                super::super::tpl::emit_inline(cx, &element.children.ops)
            }
            Op::Element(element) => {
                cx.walk.skip(element.bindings.len());
                if super::super::once::emit_hoisted_child(cx, element)? {
                    return Ok(());
                }
                super::emit_nested(cx, element, id)
            }
            Op::Component(component) => {
                cx.walk.skip(component.bindings.len());
                super::super::component::emit_nested(cx, component, id)
            }
            Op::If(if_op) => super::super::emit_if_op(cx, if_op, id),
            Op::For(for_op) => super::super::emit_for_op(cx, for_op, id, None),
            Op::Slot(slot) => {
                cx.walk.skip(slot.bindings.len());
                super::super::outlet::emit_outlet(cx, slot, None, false)
            }
            Op::Text(_) | Op::Interpolation(_) => {
                Err(EmitError::unsupported_op(Reason::ArrayChildTextRun, op))
            }
        })
    })
}
