use vize_s2::op::ComponentOp;

use crate::pass::SlotFacts;

use super::super::js::RawJs;
use super::super::{EmitCx, EmitError, builtin, create_slots, create_slots_walk, slots};

pub(super) struct Args<'a> {
    pub(super) array: bool,
    pub(super) has_array: bool,
    pub(super) create: bool,
    pub(super) facts: Option<&'a SlotFacts>,
    pub(super) spread: Option<&'a RawJs<'a>>,
    pub(super) emit_flag: bool,
    pub(super) keyed_branch: bool,
    pub(super) transition_slot_root: bool,
}

pub(super) fn emit(
    cx: &mut EmitCx<'_>,
    component: &ComponentOp<'_>,
    args: Args<'_>,
) -> Result<(), EmitError> {
    let emitted = if args.array {
        if args.has_array {
            cx.buf.push(", ");
            cx.with_static_vnode_hoist(true, |cx| {
                builtin::emit_array_children(cx, &component.children, args.keyed_branch)
            })?;
            true
        } else if args.emit_flag {
            cx.buf.push(", null");
            false
        } else {
            false
        }
    } else if args.create {
        cx.buf.push(", ");
        cx.with_static_vnode_hoist(true, |cx| {
            create_slots::emit_create_slots(cx, &component.children, args.spread)
        })?;
        true
    } else if let Some(facts) = args.facts {
        cx.buf.push(", ");
        cx.with_static_vnode_hoist(true, |cx| {
            let previous = cx.transition_slot_root;
            cx.transition_slot_root = previous || args.transition_slot_root;
            let result = slots::emit_slots(
                cx,
                &component.children,
                facts,
                args.spread,
                &component.bindings,
            );
            cx.transition_slot_root = previous;
            result
        })?;
        true
    } else if let Some(spread) = args.spread {
        cx.buf.push(", ");
        cx.buf.push(spread.as_str());
        false
    } else if args.emit_flag {
        cx.buf.push(", null");
        false
    } else {
        false
    };
    if !emitted {
        create_slots_walk::skip_ops(cx, &component.children.ops);
    }
    Ok(())
}
