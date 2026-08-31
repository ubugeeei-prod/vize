//! Transform-analogue helper pre-registration for the DOM emitter.

use vize_davinci::side_table::SideTable;
use vize_s2::op::{BindingOp, Op, Region};

use crate::lower::ForWrapper;
use crate::pass::S2Facts;
use crate::pass::walk::PageWalk;

use super::buf::Buf;
use super::create_slots_walk::first_slot_template;
use super::helper::Helper;
use super::{builtin, directive, sfc_style, slots};

pub(super) fn prefer_helpers(
    buf: &mut Buf,
    facts: &S2Facts,
    for_wrappers: &SideTable<ForWrapper>,
    walk: &mut PageWalk,
    region: &Region<'_>,
) {
    prefer_region_helpers(buf, facts, for_wrappers, walk, region, false, false);
}

fn prefer_region_helpers(
    buf: &mut Buf,
    facts: &S2Facts,
    for_wrappers: &SideTable<ForWrapper>,
    walk: &mut PageWalk,
    region: &Region<'_>,
    template_slot_context: bool,
    suppress_first_runtime_directives: bool,
) {
    if (template_slot_context && has_slot_outlet(region))
        || direct_slot_carrier_precedes_slot_outlet(region)
    {
        buf.prefer(Helper::RenderSlot);
    }
    let mut slot_context = template_slot_context;
    for (index, op) in region.ops.iter().enumerate() {
        prefer_op_helpers(
            buf,
            facts,
            for_wrappers,
            walk,
            op,
            slot_context,
            suppress_first_runtime_directives && index == 0,
        );
        slot_context |= op_is_direct_slot_carrier(op);
    }
}

fn prefer_op_helpers(
    buf: &mut Buf,
    facts: &S2Facts,
    for_wrappers: &SideTable<ForWrapper>,
    walk: &mut PageWalk,
    op: &Op<'_>,
    slot_context: bool,
    suppress_runtime_directives: bool,
) {
    let id = walk.mint();
    match op {
        Op::Element(element) if sfc_style::is_carrier_element(element) => {
            walk.skip(element.bindings.len())
        }
        Op::Element(element) => {
            let bindings = &element.bindings;
            let slot_template = slots::is_slot_template(element);
            let early_element_vnode = !slot_template && !slot_context;
            if !suppress_runtime_directives {
                directive::prefer_helpers(buf, bindings);
            }
            if early_element_vnode {
                buf.prefer(Helper::CreateElementVNode);
            }
            walk.skip(bindings.len());
            let child_slot_context = slot_context || slot_template;
            prefer_region_helpers(
                buf,
                facts,
                for_wrappers,
                walk,
                &element.children,
                child_slot_context,
                false,
            );
            if slot_context && !slot_template && !early_element_vnode {
                buf.prefer(Helper::CreateElementVNode);
            }
        }
        Op::Component(component) => {
            let bindings = &component.bindings;
            if bindings.iter().any(component_slot_content) {
                buf.prefer(Helper::RenderSlot);
            }
            directive::prefer_helpers(buf, bindings);
            if !builtin::is_dynamic_component(component) {
                buf.prefer(Helper::ResolveComponent);
            }
            walk.skip(bindings.len());
            if id.and_then(|id| facts.slot_facts.get(id)).is_some() {
                prefer_slot_helpers(buf, &component.children);
            }
            prefer_region_helpers(
                buf,
                facts,
                for_wrappers,
                walk,
                &component.children,
                slot_context,
                false,
            );
        }
        Op::Slot(slot) => {
            buf.prefer(Helper::RenderSlot);
            walk.skip(slot.bindings.len());
            prefer_region_helpers(
                buf,
                facts,
                for_wrappers,
                walk,
                &slot.fallback,
                slot_context,
                false,
            );
        }
        Op::If(if_op) => {
            buf.prefer(Helper::OpenBlock);
            buf.prefer(Helper::CreateBlock);
            buf.prefer(Helper::CreateElementBlock);
            buf.prefer(Helper::Fragment);
            buf.prefer(Helper::CreateComment);
            for branch in if_op.branches.iter() {
                prefer_if_branch_helpers(
                    buf,
                    facts,
                    for_wrappers,
                    walk,
                    &branch.region,
                    slot_context,
                );
            }
        }
        Op::For(for_op) => {
            buf.prefer(Helper::RenderList);
            buf.prefer(Helper::OpenBlock);
            buf.prefer(Helper::CreateBlock);
            buf.prefer(Helper::Fragment);
            let suppress_root_directives = id
                .and_then(|id| for_wrappers.get(id))
                .is_some_and(|_| super::tpl::should_unwrap_for(&for_op.region.ops));
            prefer_region_helpers(
                buf,
                facts,
                for_wrappers,
                walk,
                &for_op.region,
                slot_context,
                suppress_root_directives,
            );
        }
        Op::Text(_) => buf.prefer(Helper::CreateText),
        Op::Interpolation(_) => {
            buf.prefer(Helper::ToDisplayString);
            if id
                .and_then(|id| facts.text_facts.get(id))
                .is_some_and(|text| text.parts.iter().any(|part| !part.dynamic))
            {
                buf.prefer(Helper::CreateText);
            }
        }
    }
}

fn prefer_slot_helpers(buf: &mut Buf, children: &Region<'_>) {
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

fn prefer_if_branch_helpers(
    buf: &mut Buf,
    facts: &S2Facts,
    for_wrappers: &SideTable<ForWrapper>,
    walk: &mut PageWalk,
    region: &Region<'_>,
    slot_context: bool,
) {
    for op in region.ops.iter() {
        prefer_op_helpers(buf, facts, for_wrappers, walk, op, slot_context, false);
    }
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
        | Op::Interpolation(_) => false,
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
        Op::Text(_) | Op::Interpolation(_) => false,
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
        Op::Text(_) | Op::Interpolation(_) => false,
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
        Op::Element(_) | Op::Component(_) | Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) => {
            false
        }
    })
}

fn direct_slot_carrier_precedes_slot_outlet(region: &Region<'_>) -> bool {
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
        Op::Text(_) | Op::Interpolation(_) => None,
    }
}

fn op_is_direct_slot_carrier(op: &Op<'_>) -> bool {
    match op {
        Op::Element(element) => slots::is_slot_template(element),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| direct_slot_carrier_precedes_slot_outlet(&branch.region)),
        Op::For(for_op) => direct_slot_carrier_precedes_slot_outlet(&for_op.region),
        Op::Component(_) | Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) => false,
    }
}

fn op_precedes_slot_outlet_as_carrier(op: &Op<'_>) -> bool {
    op_has_component_slot_template_carrier(op) && !op_has_slot_outlet(op)
}

fn has_slot_template_carrier(region: &Region<'_>) -> bool {
    region.ops.iter().any(|op| match op {
        Op::Element(element) => slots::is_slot_template(element),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| has_slot_template_carrier(&branch.region)),
        Op::For(for_op) => has_slot_template_carrier(&for_op.region),
        Op::Component(_) | Op::Text(_) | Op::Interpolation(_) | Op::Slot(_) => false,
    })
}

fn op_has_component_slot_template_carrier(op: &Op<'_>) -> bool {
    match op {
        Op::Element(element) => {
            slots::is_slot_template(element)
                || element
                    .children
                    .ops
                    .iter()
                    .any(op_has_component_slot_template_carrier)
        }
        Op::Component(component) => {
            component.bindings.iter().any(component_slot_content)
                || has_slot_template_carrier(&component.children)
                || component
                    .children
                    .ops
                    .iter()
                    .any(op_has_component_slot_template_carrier)
        }
        Op::If(if_op) => if_op.branches.iter().any(|branch| {
            branch
                .region
                .ops
                .iter()
                .any(op_has_component_slot_template_carrier)
        }),
        Op::For(for_op) => for_op
            .region
            .ops
            .iter()
            .any(op_has_component_slot_template_carrier),
        Op::Slot(_) | Op::Text(_) | Op::Interpolation(_) => false,
    }
}

fn component_slot_content(binding: &BindingOp<'_>) -> bool {
    matches!(binding, BindingOp::SlotContent(_))
}

fn has_slot_outlet(region: &Region<'_>) -> bool {
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
        Op::Text(_) | Op::Interpolation(_) => false,
    }
}
