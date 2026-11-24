//! Transform-analogue helper pre-registration for the DOM emitter.

mod slot_order;
#[cfg(test)]
mod tests;
mod unref;

use alloc::vec::Vec as StdVec;

use vize_l0::ensure_sufficient_stack;
use vize_l0::side_table::SideTable;
use vize_l2::expr::{ExprRef, OpaqueReason};
use vize_l2::op::{ComponentOp, Op, Region};

use vize_l2::analysis::fragment_facts::LoopFragment;
use vize_l2::analysis::pass_facts::L2Facts;
use vize_l2::analysis::text_facts::TextParts;
use vize_l2::walk::PageWalk;

use super::buf::Buf;
use super::helper::Helper;
use super::options::BindingTable;
use super::{builtin, component, directive, slots, style_carrier};
use slot_order::{
    component_slot_content, direct_slot_carrier_precedes_slot_outlet,
    explicit_transition_slot_with_implicit_transition_group, has_slot_outlet,
    op_is_direct_slot_carrier, prefer_slot_helpers,
};

/// What the preference walk reads besides the region it is walking.
pub(super) struct PreferCx<'a> {
    pub(super) source: &'a str,
    pub(super) facts: &'a L2Facts,
    pub(super) texts: &'a SideTable<TextParts>,
    pub(super) for_wrappers: &'a SideTable<LoopFragment>,
    /// The script bindings, when the emit carries them: a component tag
    /// named there resolves to `$setup` and never marks `resolveComponent`
    /// during the transform (`lane::element`, which matches the tag
    /// verbatim — the camelize/PascalCase widening is codegen's alone).
    pub(super) bindings: Option<&'a BindingTable>,
    /// Inline render closures wrap setup `let` reads in `_unref`.
    pub(super) inline: bool,
    /// First authored model or runtime-directive read that registers `_unref`
    /// before children are emitted.
    pub(super) authored_eager_unref: core::cell::Cell<u32>,
    /// First authored `v-for`, which a slot helper may pre-register at
    /// the owning component before the transform would visit it.
    pub(super) authored_for: core::cell::Cell<u32>,
    /// Whether the first loop source itself, rather than its callback, reads
    /// through `_unref` at the loop visit.
    pub(super) first_for_source_unref: core::cell::Cell<bool>,
    /// Transition slot structure whose `renderList` is pre-registered at
    /// the parent component, ahead of its authored loop.
    pub(super) early_transition_slot_for: core::cell::Cell<bool>,
}

impl PreferCx<'_> {
    fn tag_is_binding(&self, tag: &str) -> bool {
        self.bindings.is_some_and(|table| table.contains(tag))
    }
}

/// The render preamble's asset names, collected by the preference walk.
///
/// `generate_assets` resolves every component and custom directive the
/// template uses before the render body. Each list used to cost a walk of
/// the whole template. Collecting both here saves those two walks per
/// compile (the Cranelift ægraph lesson in
/// `docs/davinci/prior-art-backends.md`: passes over the same nodes compose
/// into one visit). Each list keeps the order its own walk produced:
/// directives in pre-order (an owner's bindings before its children),
/// components in post-order (children before the component). The replaced
/// walks live on in the tests as oracles.
#[derive(Debug, Default)]
pub(super) struct AssetNames<'a> {
    pub(super) components: StdVec<&'a str>,
    pub(super) directives: StdVec<&'a str>,
    /// Open `Op::And` regions. Component assets never come from inside one:
    /// a JSX `{c && <A/>}` child resolves at its own call.
    and_depth: u32,
}

impl<'a> AssetNames<'a> {
    fn note_component(&mut self, owner: &ComponentOp<'a>, source: &str) {
        if self.and_depth == 0 {
            component::note_name(owner, source, &mut self.components);
        }
    }
}

/// Pre-register the render helpers in transform order, and collect the
/// preamble's asset names in the same walk.
pub(super) fn prefer_helpers<'a>(
    buf: &mut Buf,
    cx: &PreferCx<'_>,
    walk: &mut PageWalk,
    region: &Region<'a>,
) -> AssetNames<'a> {
    let mut names = AssetNames::default();
    prefer_region_helpers(buf, cx, walk, &mut names, region, false, false);
    names
}

fn prefer_region_helpers<'a>(
    buf: &mut Buf,
    cx: &PreferCx<'_>,
    walk: &mut PageWalk,
    names: &mut AssetNames<'a>,
    region: &Region<'a>,
    template_slot_context: bool,
    suppress_first_runtime_directives: bool,
) {
    ensure_sufficient_stack(|| {
        prefer_region_helpers_guarded(
            buf,
            cx,
            walk,
            names,
            region,
            template_slot_context,
            suppress_first_runtime_directives,
        );
    });
}

fn prefer_region_helpers_guarded<'a>(
    buf: &mut Buf,
    cx: &PreferCx<'_>,
    walk: &mut PageWalk,
    names: &mut AssetNames<'a>,
    region: &Region<'a>,
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
            names,
            op,
            slot_context,
            suppress_first_runtime_directives && index == 0,
        );
        slot_context |= op_is_direct_slot_carrier(op);
    }
}

fn prefer_op_helpers<'a>(
    buf: &mut Buf,
    cx: &PreferCx<'_>,
    walk: &mut PageWalk,
    names: &mut AssetNames<'a>,
    op: &Op<'a>,
    slot_context: bool,
    suppress_runtime_directives: bool,
) {
    let id = walk.mint();
    let visit = walk.visits();
    unref::note_op(cx, op, visit);
    buf.set_prefer_visit(visit);
    match op {
        // A carrier holds only `v-bind()` CSS bindings and no children, so
        // it has no assets to collect either.
        Op::Element(element) if style_carrier::is_carrier_element(element) => {
            walk.skip(element.bindings.len())
        }
        Op::Element(element) => {
            let bindings = &element.bindings;
            let slot_template = slots::is_slot_template(element);
            let early_element_vnode = !slot_template && !slot_context;
            if !suppress_runtime_directives {
                directive::prefer_helpers(buf, bindings);
            }
            directive::note_names(bindings, &mut names.directives);
            if early_element_vnode {
                buf.prefer(Helper::CreateElementVNode);
            }
            walk.skip(bindings.len());
            let child_slot_context = slot_context || slot_template;
            prefer_region_helpers(
                buf,
                cx,
                walk,
                names,
                &element.children,
                child_slot_context,
                false,
            );
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
            directive::note_names(bindings, &mut names.directives);
            if !builtin::is_dynamic_component(component, cx.source)
                && !cx.tag_is_binding(component.name)
            {
                buf.prefer(Helper::ResolveComponent);
            }
            walk.skip(bindings.len());
            if id.and_then(|id| cx.facts.slot_facts.get(id)).is_some() {
                if explicit_transition_slot_with_implicit_transition_group(&component.children) {
                    cx.early_transition_slot_for.set(true);
                }
                prefer_slot_helpers(buf, &component.children);
            }
            prefer_region_helpers(
                buf,
                cx,
                walk,
                names,
                &component.children,
                slot_context,
                false,
            );
            names.note_component(component, cx.source);
        }
        Op::Slot(slot) => {
            buf.prefer(Helper::RenderSlot);
            walk.skip(slot.bindings.len());
            directive::note_names(&slot.bindings, &mut names.directives);
            prefer_region_helpers(buf, cx, walk, names, &slot.fallback, slot_context, false);
        }
        Op::If(if_op) => {
            buf.prefer(Helper::OpenBlock);
            buf.prefer(Helper::CreateBlock);
            buf.prefer(Helper::CreateElementBlock);
            buf.prefer(Helper::Fragment);
            buf.prefer(Helper::CreateComment);
            for branch in if_op.branches.iter() {
                prefer_if_branch_helpers(buf, cx, walk, names, &branch.region, slot_context);
            }
        }
        // A falsy `left` is the child itself: no comment placeholder.
        Op::And(and) => {
            names.and_depth += 1;
            prefer_if_branch_helpers(buf, cx, walk, names, &and.region, slot_context);
            names.and_depth -= 1;
        }
        Op::For(for_op) => {
            if cx.authored_for.get() == u32::MAX {
                cx.authored_for.set(visit);
            }
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
                names,
                &for_op.region,
                slot_context,
                suppress_root_directives,
            );
        }
        Op::OriginalFor(for_op) => {
            prefer_region_helpers(buf, cx, walk, names, &for_op.region, slot_context, false);
        }
        Op::ChildValue(_) => {}
        Op::Text(_) => buf.prefer(Helper::CreateText),
        Op::Comment(_) => buf.prefer(Helper::CreateComment),
        Op::Interpolation(interpolation) => {
            let compound = id.and_then(|id| cx.texts.get(id));
            let needs_display_string = if matches!(
                interpolation.expression,
                ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound
            ) {
                compound
                    .is_none_or(|text| text.parts.iter().any(|part| part.dynamic && !part.raw_html))
            } else {
                !interpolation.raw_html
            };
            if needs_display_string {
                buf.prefer(Helper::ToDisplayString);
            }
            if compound.is_some_and(|text| text.parts.iter().any(|part| !part.dynamic)) {
                buf.prefer(Helper::CreateText);
            }
        }
    }
}

fn prefer_if_branch_helpers<'a>(
    buf: &mut Buf,
    cx: &PreferCx<'_>,
    walk: &mut PageWalk,
    names: &mut AssetNames<'a>,
    region: &Region<'a>,
    slot_context: bool,
) {
    ensure_sufficient_stack(|| {
        for op in region.ops.iter() {
            prefer_op_helpers(buf, cx, walk, names, op, slot_context, false);
        }
    });
}
