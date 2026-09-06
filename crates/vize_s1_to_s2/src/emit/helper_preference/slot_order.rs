use vize_s2::op::{BindingOp, Op, Region};

use super::super::buf::Buf;
use super::super::create_slots_walk::first_slot_template;
use super::super::helper::Helper;
use super::super::slots;

pub(super) fn prefer_slot_helpers(buf: &mut Buf, children: &Region<'_>) {
    if direct_slot_carrier_precedes_slot_outlet(children) {
        buf.prefer(Helper::RenderSlot);
    }
    if slot_outlet_precedes_vnode(children) {
        buf.prefer(Helper::RenderSlot);
    }
    if dynamic_slot_carrier_has_slot_outlet(children) {
        buf.prefer(Helper::RenderSlot);
    }
    if has_dynamic_slot_for(children) {
        buf.prefer(Helper::RenderList);
    }
    if conditional_slot_template_has_direct_v_for(children) {
        buf.prefer(Helper::RenderList);
    }
    if explicit_transition_slot_with_implicit_transition_group(children) {
        buf.prefer(Helper::RenderList);
        buf.prefer(Helper::WithCtx);
        buf.prefer(Helper::Transition);
        buf.prefer(Helper::VShow);
    }
}

pub(super) fn component_slot_content(binding: &BindingOp<'_>) -> bool {
    matches!(binding, BindingOp::SlotContent(_))
}

fn has_dynamic_slot_for(region: &Region<'_>) -> bool {
    region
        .ops
        .iter()
        .any(|op| matches!(op, Op::For(for_op) if first_slot_template(&for_op.region).is_some()))
}

fn conditional_slot_template_has_direct_v_for(region: &Region<'_>) -> bool {
    region.ops.iter().any(|op| match op {
        Op::If(if_op) => if_op.branches.iter().any(|branch| {
            first_slot_template(&branch.region).is_some_and(|(_, element, _)| {
                element
                    .children
                    .ops
                    .iter()
                    .any(|op| matches!(op, Op::For(_)))
            })
        }),
        Op::Element(_)
        | Op::Component(_)
        | Op::Slot(_)
        | Op::For(_)
        | Op::Text(_)
        | Op::Interpolation(_)
        | Op::Comment(_) => false,
    })
}

fn explicit_transition_slot_with_implicit_transition_group(region: &Region<'_>) -> bool {
    let mut explicit_transition = false;
    let mut implicit_transition_group = false;
    for op in region.ops.iter() {
        if let Op::Element(element) = op
            && slots::is_slot_template(element)
        {
            explicit_transition |= region_has_transition(&element.children);
            continue;
        }
        implicit_transition_group |= op_has_transition_group(op);
    }
    explicit_transition && implicit_transition_group
}

fn region_has_transition(region: &Region<'_>) -> bool {
    region.ops.iter().any(op_has_transition)
}

fn op_has_transition(op: &Op<'_>) -> bool {
    match op {
        Op::Component(component) => {
            matches!(component.name, "Transition" | "transition")
                || region_has_transition(&component.children)
        }
        Op::Element(element) => region_has_transition(&element.children),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| region_has_transition(&branch.region)),
        Op::For(for_op) => region_has_transition(&for_op.region),
        Op::Slot(slot) => region_has_transition(&slot.fallback),
        Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => false,
    }
}

fn op_has_transition_group(op: &Op<'_>) -> bool {
    match op {
        Op::Component(component) => {
            matches!(component.name, "TransitionGroup" | "transition-group")
                || region_has_transition_group(&component.children)
        }
        Op::Element(element) => region_has_transition_group(&element.children),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| region_has_transition_group(&branch.region)),
        Op::For(for_op) => region_has_transition_group(&for_op.region),
        Op::Slot(slot) => region_has_transition_group(&slot.fallback),
        Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => false,
    }
}

fn region_has_transition_group(region: &Region<'_>) -> bool {
    region.ops.iter().any(op_has_transition_group)
}

fn dynamic_slot_carrier_has_slot_outlet(region: &Region<'_>) -> bool {
    region.ops.iter().any(|op| match op {
        Op::If(if_op) => if_op.branches.iter().any(|branch| {
            first_slot_template(&branch.region).is_some() && has_slot_outlet(&branch.region)
        }),
        Op::For(for_op) => {
            first_slot_template(&for_op.region).is_some() && has_slot_outlet(&for_op.region)
        }
        Op::Element(_)
        | Op::Component(_)
        | Op::Slot(_)
        | Op::Text(_)
        | Op::Interpolation(_)
        | Op::Comment(_) => false,
    })
}

pub(super) fn direct_slot_carrier_precedes_slot_outlet(region: &Region<'_>) -> bool {
    let mut saw_carrier = false;
    for op in region.ops.iter() {
        if saw_carrier && op_has_slot_outlet(op) {
            return true;
        }
        saw_carrier |= op_precedes_slot_outlet_as_carrier(op);
    }
    false
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SlotOrderMarker {
    SlotOutlet,
    VNode,
}

fn slot_outlet_precedes_vnode(region: &Region<'_>) -> bool {
    first_slot_order_marker(region) == Some(SlotOrderMarker::SlotOutlet)
}

fn first_slot_order_marker(region: &Region<'_>) -> Option<SlotOrderMarker> {
    region.ops.iter().find_map(first_op_slot_order_marker)
}

fn first_op_slot_order_marker(op: &Op<'_>) -> Option<SlotOrderMarker> {
    match op {
        Op::Slot(_) => Some(SlotOrderMarker::SlotOutlet),
        Op::Element(element) if slots::is_slot_template(element) => {
            first_slot_order_marker(&element.children)
        }
        Op::Element(_) | Op::Component(_) => Some(SlotOrderMarker::VNode),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .find_map(|branch| first_slot_order_marker(&branch.region)),
        Op::For(for_op) => first_slot_order_marker(&for_op.region),
        Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => None,
    }
}

pub(super) fn op_is_direct_slot_carrier(op: &Op<'_>) -> bool {
    match op {
        Op::Element(element) => slots::is_slot_template(element),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| direct_slot_carrier_precedes_slot_outlet(&branch.region)),
        Op::For(for_op) => direct_slot_carrier_precedes_slot_outlet(&for_op.region),
        Op::Component(_) | Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => {
            false
        }
    }
}

fn op_precedes_slot_outlet_as_carrier(op: &Op<'_>) -> bool {
    op_has_direct_slot_carrier(op) && !op_has_slot_outlet(op)
}

fn region_has_direct_slot_carrier(region: &Region<'_>) -> bool {
    region.ops.iter().any(op_has_direct_slot_carrier)
}

fn op_has_direct_slot_carrier(op: &Op<'_>) -> bool {
    match op {
        Op::Element(element) => slots::is_slot_template(element),
        Op::Component(component) => {
            component.bindings.iter().any(component_slot_content)
                || component_tree_has_slot_carrier(&component.children)
        }
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| region_has_direct_slot_carrier(&branch.region)),
        Op::For(for_op) => region_has_direct_slot_carrier(&for_op.region),
        Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => false,
    }
}

fn component_tree_has_slot_carrier(region: &Region<'_>) -> bool {
    region.ops.iter().any(|op| match op {
        Op::Element(element) => slots::is_slot_template(element),
        Op::Component(component) => {
            component.bindings.iter().any(component_slot_content)
                || component_tree_has_slot_carrier(&component.children)
        }
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| component_tree_has_slot_carrier(&branch.region)),
        Op::For(for_op) => component_tree_has_slot_carrier(&for_op.region),
        Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => false,
    })
}

pub(super) fn has_slot_outlet(region: &Region<'_>) -> bool {
    region.ops.iter().any(op_has_slot_outlet)
}

fn op_has_slot_outlet(op: &Op<'_>) -> bool {
    match op {
        Op::Element(element) => has_slot_outlet(&element.children),
        Op::Component(component) => has_slot_outlet(&component.children),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| has_slot_outlet(&branch.region)),
        Op::For(for_op) => has_slot_outlet(&for_op.region),
        Op::Slot(_) => true,
        Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => false,
    }
}
