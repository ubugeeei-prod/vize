//! Transform-analogue helper pre-registration for the DOM emitter.

use vize_s2::op::{Op, Region};

use crate::pass::S2Facts;
use crate::pass::walk::PageWalk;

use super::buf::Buf;
use super::helper::Helper;
use super::{directive, sfc_style, slots};

pub(super) fn prefer_helpers(
    buf: &mut Buf,
    facts: &S2Facts,
    walk: &mut PageWalk,
    region: &Region<'_>,
) {
    if component_slot_template_carrier_precedes_slot_outlet(region) {
        buf.prefer(Helper::RenderSlot);
    }
    prefer_region_helpers(buf, facts, walk, region, false);
}

fn prefer_region_helpers(
    buf: &mut Buf,
    facts: &S2Facts,
    walk: &mut PageWalk,
    region: &Region<'_>,
    template_slot_context: bool,
) {
    for op in region.ops.iter() {
        let id = walk.mint();
        match op {
            Op::Element(element) if sfc_style::is_carrier_element(element) => {
                walk.skip(element.bindings.len())
            }
            Op::Element(element) => {
                let bindings = &element.bindings;
                directive::prefer_helpers(buf, bindings);
                if !template_slot_context {
                    buf.prefer(Helper::CreateElementVNode);
                }
                walk.skip(bindings.len());
                prefer_region_helpers(buf, facts, walk, &element.children, template_slot_context);
                if template_slot_context {
                    buf.prefer(Helper::CreateElementVNode);
                }
            }
            Op::Component(component) => {
                let bindings = &component.bindings;
                directive::prefer_helpers(buf, bindings);
                buf.prefer(Helper::ResolveComponent);
                walk.skip(bindings.len());
                let has_template_slot_carrier = has_slot_template_carrier(&component.children);
                let template_slot_context = template_slot_context || has_template_slot_carrier;
                if (has_template_slot_carrier && has_slot_outlet(&component.children))
                    || component_slot_template_carrier_precedes_slot_outlet(&component.children)
                {
                    buf.prefer(Helper::RenderSlot);
                }
                prefer_region_helpers(buf, facts, walk, &component.children, template_slot_context);
            }
            Op::Slot(slot) => {
                buf.prefer(Helper::RenderSlot);
                walk.skip(slot.bindings.len());
                prefer_region_helpers(buf, facts, walk, &slot.fallback, template_slot_context);
            }
            Op::If(if_op) => {
                buf.prefer(Helper::OpenBlock);
                buf.prefer(Helper::CreateBlock);
                buf.prefer(Helper::CreateElementBlock);
                buf.prefer(Helper::Fragment);
                buf.prefer(Helper::CreateComment);
                for branch in if_op.branches.iter() {
                    prefer_region_helpers(buf, facts, walk, &branch.region, template_slot_context);
                }
            }
            Op::For(for_op) => {
                buf.prefer(Helper::RenderList);
                buf.prefer(Helper::OpenBlock);
                buf.prefer(Helper::CreateBlock);
                buf.prefer(Helper::Fragment);
                prefer_region_helpers(buf, facts, walk, &for_op.region, template_slot_context);
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

fn has_slot_outlet(region: &Region<'_>) -> bool {
    region.ops.iter().any(|op| match op {
        Op::Element(element) => has_slot_outlet(&element.children),
        Op::Component(component) => has_slot_outlet(&component.children),
        Op::If(if_op) => if_op
            .branches
            .iter()
            .any(|branch| has_slot_outlet(&branch.region)),
        Op::For(for_op) => has_slot_outlet(&for_op.region),
        Op::Slot(_) => true,
        Op::Text(_) | Op::Interpolation(_) => false,
    })
}

fn component_slot_template_carrier_precedes_slot_outlet(region: &Region<'_>) -> bool {
    let mut saw_carrier = false;
    for op in region.ops.iter() {
        if saw_carrier && op_has_slot_outlet(op) {
            return true;
        }
        saw_carrier |= op_has_component_slot_template_carrier(op);
    }
    false
}

fn op_has_component_slot_template_carrier(op: &Op<'_>) -> bool {
    match op {
        Op::Element(element) => element
            .children
            .ops
            .iter()
            .any(op_has_component_slot_template_carrier),
        Op::Component(component) => {
            has_slot_template_carrier(&component.children)
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
