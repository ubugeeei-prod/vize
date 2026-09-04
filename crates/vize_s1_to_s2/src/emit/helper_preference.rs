//! Transform-analogue helper pre-registration for the DOM emitter.

mod slot_order;

use vize_davinci::side_table::SideTable;
use vize_s2::op::{Op, Region};

use crate::lower::ForWrapper;
use crate::pass::S2Facts;
use crate::pass::walk::PageWalk;

use super::buf::Buf;
use super::helper::Helper;
use super::options::BindingTable;
use super::{builtin, directive, sfc_style, slots};
use slot_order::{
    component_slot_content, direct_slot_carrier_precedes_slot_outlet, has_slot_outlet,
    op_is_direct_slot_carrier, prefer_deferred_slot_helpers, prefer_slot_helpers,
};

/// What the preference walk reads besides the region it is walking.
pub(super) struct PreferCx<'a> {
    pub(super) facts: &'a S2Facts,
    pub(super) for_wrappers: &'a SideTable<ForWrapper>,
    /// The script bindings, when the emit carries them: a component tag
    /// named there resolves to `$setup` and never marks `resolveComponent`
    /// during the transform (`lane::element`, which matches the tag
    /// verbatim — the camelize/PascalCase widening is codegen's alone).
    pub(super) bindings: Option<&'a BindingTable>,
}

impl PreferCx<'_> {
    fn tag_is_binding(&self, tag: &str) -> bool {
        self.bindings.is_some_and(|table| table.contains(tag))
    }
}

pub(super) fn prefer_helpers(
    buf: &mut Buf,
    cx: &PreferCx<'_>,
    walk: &mut PageWalk,
    region: &Region<'_>,
) {
    prefer_region_helpers(buf, cx, walk, region, false, false);
}

fn prefer_region_helpers(
    buf: &mut Buf,
    cx: &PreferCx<'_>,
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
            cx,
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
    cx: &PreferCx<'_>,
    walk: &mut PageWalk,
    op: &Op<'_>,
    slot_context: bool,
    suppress_runtime_directives: bool,
) {
    let id = walk.mint();
    let visit = walk.visits();
    buf.set_prefer_visit(visit);
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
            prefer_region_helpers(buf, cx, walk, &element.children, child_slot_context, false);
            if slot_context && !slot_template && !early_element_vnode {
                buf.set_prefer_visit(visit);
                buf.prefer(Helper::CreateElementVNode);
            }
        }
        Op::Component(component) => {
            let bindings = &component.bindings;
            if bindings.iter().any(component_slot_content) {
                buf.prefer(Helper::RenderSlot);
            }
            directive::prefer_helpers(buf, bindings);
            if !builtin::is_dynamic_component(component) && !cx.tag_is_binding(component.name) {
                buf.prefer(Helper::ResolveComponent);
            }
            walk.skip(bindings.len());
            let slot_carrier = id.and_then(|id| cx.facts.slot_facts.get(id)).is_some();
            if slot_carrier {
                prefer_slot_helpers(buf, &component.children);
            }
            prefer_region_helpers(buf, cx, walk, &component.children, slot_context, false);
            if slot_carrier {
                buf.set_prefer_visit(walk.visits());
                prefer_deferred_slot_helpers(buf, &component.children);
            }
        }
        Op::Slot(slot) => {
            buf.prefer(Helper::RenderSlot);
            walk.skip(slot.bindings.len());
            prefer_region_helpers(buf, cx, walk, &slot.fallback, slot_context, false);
        }
        Op::If(if_op) => {
            buf.prefer(Helper::OpenBlock);
            buf.prefer(Helper::CreateBlock);
            buf.prefer(Helper::CreateElementBlock);
            buf.prefer(Helper::Fragment);
            buf.prefer(Helper::CreateComment);
            for branch in if_op.branches.iter() {
                prefer_if_branch_helpers(buf, cx, walk, &branch.region, slot_context);
            }
        }
        Op::For(for_op) => {
            buf.prefer(Helper::RenderList);
            buf.prefer(Helper::OpenBlock);
            buf.prefer(Helper::CreateBlock);
            buf.prefer(Helper::Fragment);
            let suppress_root_directives = id
                .and_then(|id| cx.for_wrappers.get(id))
                .is_some_and(|_| super::tpl::should_unwrap_for(&for_op.region.ops));
            prefer_region_helpers(
                buf,
                cx,
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
                .and_then(|id| cx.facts.text_facts.get(id))
                .is_some_and(|text| text.parts.iter().any(|part| !part.dynamic))
            {
                buf.prefer(Helper::CreateText);
            }
        }
    }
}

fn prefer_if_branch_helpers(
    buf: &mut Buf,
    cx: &PreferCx<'_>,
    walk: &mut PageWalk,
    region: &Region<'_>,
    slot_context: bool,
) {
    for op in region.ops.iter() {
        prefer_op_helpers(buf, cx, walk, op, slot_context, false);
    }
}
