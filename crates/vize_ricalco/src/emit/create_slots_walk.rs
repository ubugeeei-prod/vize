//! Walk helpers for `createSlots` entry collection.

use vize_s2::op::{BindingOp, ElementOp, ForOp, IfOp, Op, Region, SlotContentOp};

use super::EmitCx;

pub(super) fn slot_content<'a>(element: &'a ElementOp<'a>) -> Option<&'a SlotContentOp<'a>> {
    element.bindings.iter().find_map(|binding| match binding {
        BindingOp::SlotContent(content) => Some(&**content),
        _ => None,
    })
}

pub(super) fn slot_template_content<'a>(
    element: &'a ElementOp<'a>,
) -> Option<&'a SlotContentOp<'a>> {
    if element.tag == "template" {
        slot_content(element)
    } else {
        None
    }
}

pub(super) fn first_slot_template<'a>(
    region: &'a Region<'a>,
) -> Option<(usize, &'a ElementOp<'a>, &'a SlotContentOp<'a>)> {
    region.ops.iter().enumerate().find_map(|(i, op)| match op {
        Op::Element(element) => {
            slot_template_content(element).map(|content| (i, &**element, content))
        }
        _ => None,
    })
}

pub(super) fn is_slot_if(if_op: &IfOp<'_>) -> bool {
    if_op
        .branches
        .iter()
        .any(|branch| first_slot_template(&branch.region).is_some())
}

pub(super) fn is_slot_for(for_op: &ForOp<'_>) -> bool {
    first_slot_template(&for_op.region).is_some()
}

pub(super) fn skip_ops(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) {
    for op in ops {
        skip_op(cx, op);
    }
}

fn skip_op(cx: &mut EmitCx<'_>, op: &Op<'_>) {
    let _id = cx.walk.mint();
    match op {
        Op::Element(element) => {
            cx.walk.skip(element.bindings.len());
            skip_ops(cx, &element.children.ops);
        }
        Op::Component(component) => {
            cx.walk.skip(component.bindings.len());
            skip_ops(cx, &component.children.ops);
        }
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                skip_ops(cx, &branch.region.ops);
            }
        }
        Op::For(for_op) => skip_ops(cx, &for_op.region.ops),
        Op::Slot(slot) => {
            cx.walk.skip(slot.bindings.len());
            skip_ops(cx, &slot.fallback.ops);
        }
        Op::Text(_) | Op::Interpolation(_) => {}
    }
}
