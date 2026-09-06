//! S2 → DOM render-function emission (P2-11).
//!
//! The published home for the DOM backend. `vize_atelier_dom` calls this
//! module for normal DOM emission; it writes JS directly from S2 ops, without
//! relief codegen-nodes.
//!
//! This installment emits **static native HTML / SVG / MathML**, interpolations,
//! mixed text siblings, static-name `ui.bind`, static-name `ui.on`
//! (including event/key/option modifiers), native `ui.if`, **native
//! `ui.for`**, **object-spread `v-bind`** (`normalizeProps` /
//! `mergeProps`), **static-name components** (`resolveComponent` /
//! `createVNode` / `createBlock`), **object `v-on`** (`toHandlers`),
//! and **implicit default slots** (`withCtx` / `_: 1|2`, including text,
//! static-vnode hoists, static `ui.for` item blocks, named / scoped
//! `<template>` slots, `createSlots` for `v-if` / `v-for` slot
//! templates, **slot outlets** (`renderSlot` / `_: 3 FORWARDED`), and
//! Vue builtins (`Teleport` / `KeepAlive` / `Transition` / `Suspense`),
//! `<component :is>` (`resolveDynamicComponent`), and **template
//! fragments** (empty → `null`, multi-root / compound-root
//! `_Fragment` + `STABLE_FRAGMENT`), **`<template v-if>` /
//! `<template v-for>` fragments** (`STABLE_FRAGMENT` / unwrap after
//! hoist), **slot outlet same-name `:name` / `v-bind:name`**,
//! **object `v-on`** (`toHandlers(..., true)`), **`v-model`**
//! (native `withDirectives` + `vModelText`-family helpers; component
//! `modelValue` / `onUpdate:` product props), and **custom directives**
//! (`resolveDirective` + `_withDirectives`, merged with native
//! `v-model`, including dynamic component model arguments), **colon /
//! vnode-hook events** (`@update:…`,
//! `@vue:mounted`) including merged duplicate handlers, and
//! **destructured `v-for` aliases** (`({ id })`, `[a, b]`, defaults),
//! **`createSlots` + `v-slots`** (`...expr` on the `{ _: 2 }` base), and
//! **dynamic `v-if` keys** (`:key="expr"`), foreign namespace boundaries,
//! template refs, Vue 2 `.native` event sugar, static+dynamic `style`,
//! dynamic `v-on` keys, native-element `v-once` / `v-memo`, `v-html` /
//! `v-text`, ordinary template comments, `v-bind` modifiers, dynamic
//! `v-bind` keys / modifiers, and Vue 2 pipe filters legalized by
//! `legacy-sugar`, and **module mode**
//! ([`DomEmitOptions`]: `import { … } from "vue"` + `export function
//! render(_ctx, _cache)`, custom runtime module / global names), and
//! **`cache_handlers`** (`_cache[n] || (_cache[n] = …)` around a `v-on`
//! handler, guarded-and-forwarded when it is a reference, dropping the
//! handler from the patch flag; suppressed under `v-for` / slot params),
//! and **`scope_id`** (`<style scoped>`'s `"data-v-abc123": ""` pair on
//! every props object — inline, hoisted and component alike, and once as a
//! trailing `mergeProps` argument rather than per spread segment), and
//! experimental in-tag comments. `atelier_dom` selects this lane for supported
//! DOM compiles, including source-map requests whose maps are verified against
//! compatibility codegen. Compatibility codegen remains available only for
//! unsupported option surfaces.

mod budget;
mod buf;
mod builtin;
mod cache_slots;
mod children;
mod component;
mod constant_expr;
mod create_slots;
mod create_slots_walk;
mod custom_element;
mod cx;
mod directive;
mod dispatch;
mod entity;
mod entry;
mod error;
mod filter;
mod flag;
mod fragment;
mod helper;
mod helper_preference;
mod hoist;
mod html;
pub(crate) mod js;
mod js_comment;
mod memo;
mod merge;
mod model;
mod model_key;
mod namespace;
mod on;
mod on_body;
mod on_dynamic;
mod on_typed;
mod once;
mod options;
mod outlet;
mod outlet_props;
mod prefix;
mod props;
mod props_bind;
mod props_class;
mod props_dynamic;
mod props_object;
mod props_object_merge;
mod props_static;
mod props_value;
mod run;
mod sfc_style;
mod slot_root;
mod slots;
mod static_cache;
mod style;
mod tpl;
mod vfor;
mod vfor_item;
mod vif;
mod vnode;
mod vnode_children;
mod vnode_static;
mod vtext;

use alloc::vec::Vec as StdVec;
use vize_davinci::id::NodeId;
use vize_davinci::side_table::SideTable;
use vize_s0::String;
use vize_s2::op::Namespace;
use vize_s2::scope::ScopeFacts;

use crate::lower::{ForWrapper, WrapperKeys};
use crate::pass::S2Facts;
use crate::pass::walk::PageWalk;

pub use self::budget::{
    DomEmitBudget, ObservedDomEmit, emit_dom_source_observed,
    emit_dom_source_observed_with_options, emit_dom_source_with_caps_observed,
};
use self::buf::Buf;
pub(crate) use self::custom_element::tag_pattern_matches;
use self::dispatch::{emit_for_item_call, emit_for_op, emit_if_branch_call, emit_if_op};
pub use self::entry::{
    DomEmit, DomEmitSections, emit_dom_source, emit_dom_source_with_caps,
    emit_dom_source_with_options,
};
pub use self::error::{EmitError, UnsupportedReason, UnsupportedRefusal};
pub use self::options::{BindingKind, BindingTable, DomEmitMode, DomEmitOptions};
use self::run::emit_dom_with_emit_budget;
pub use self::run::{emit_dom, emit_dom_with_options};

/// Per-emit numbering + helper buffer. Page-order ids re-derive the
/// same arithmetic the S2 passes use so compound text facts resolve.
struct EmitCx<'facts> {
    buf: Buf,
    source: &'facts str,
    facts: &'facts S2Facts,
    scopes: &'facts SideTable<ScopeFacts>,
    wrappers: &'facts SideTable<WrapperKeys>,
    for_wrappers: &'facts SideTable<ForWrapper>,
    walk: PageWalk,
    scope_names: StdVec<String>,
    /// Sibling `v-if` chains share one counter; nested chains reset.
    if_branch_key: u32,
    /// `v-once` / `v-memo` cache slots are numbered in render-order,
    /// sharing the function-level `_cache` array like the shipped lane.
    once_cache_index: u32,
    /// Static children nested inside `v-once` follow shipped one-shot hoists.
    once_depth: u32,
    /// Element child-list depth inside the current `v-once` owner.
    once_element_depth: u32,
    /// Slot objects inside `v-for` carry `_: 2 /* DYNAMIC */`.
    in_v_for: bool,
    /// Emission is inside a `v-for` render-list item that was lowered from
    /// a `v-if` branch, where descendant hoists stay inline like the shipped lane.
    conditional_v_for_item: bool,
    /// The next array child is the single component child of a lowered
    /// `<template v-for>` item; its root props inherit the old item-root hoist.
    template_for_item_single_root: bool,
    /// Page-order id for the component currently claiming
    /// `template_for_item_single_root`.
    template_for_item_root_id: Option<NodeId>,
    /// The current branch root was unwrapped from `<template v-if>`.
    template_if_branch_root: bool,
    /// The current `v-for` branch root came from an authored `<template v-if>`.
    template_if_for_branch_root: bool,
    /// A native root unwrapped from `<template v-for>` drops an authored
    /// child key unless the key was authored on the template wrapper.
    suppress_template_for_child_key: bool,
    /// `v-for + v-memo` uses Vue's special `_cached` callback shape, so
    /// the item emitter must skip the ordinary `_withMemo` wrapper.
    skip_memo: bool,
    /// Nested components inside a scoped `withCtx` treat forwarded
    /// outlets as `_: 2` + `DYNAMIC_SLOTS` (Vue `has_slot_params`).
    slot_param_depth: u32,
    /// Legacy `hoist_static_vnodes` recursion state. Directive/component/branch
    /// roots stay inline, but descendants may still become hoisted static VNodes.
    hoist_static_vnodes: bool,
    /// Transition/BaseTransition slots keep keyed native roots as block VNodes.
    transition_slot_root: bool,
    /// The shipped lane caches static child vnodes only after transform
    /// produced at least one root hoist.
    static_cache: bool,
    /// Current native parent namespace. DOM runtime namespace inference
    /// depends on SVG/MathML boundaries staying block-local while same-namespace
    /// descendants remain inline VNodes.
    parent_ns: Namespace,
    /// `prefix_identifiers`: expressions go through [`prefix`] on their
    /// way out instead of being pushed verbatim.
    prefix_identifiers: bool,
    /// `hoist_static`: static props and static VNode declarations are emitted
    /// only when the public transform option enabled them.
    hoist_static: bool,
    /// The shipped lane's `is_ts`: expressions are type-erased first.
    is_ts: bool,
    /// The shipped lane's `cache_handlers`.
    cache_handlers: bool,
    /// Scoped-style attr that only module-level static VNode hoists bake into
    /// props; runtime VNodes rely on Vue's current scope id.
    hoisted_scope_id: Option<&'facts str>,
    /// The shipped lane's `scope_id`, emitted as the trailing props pair.
    scope_id: Option<&'facts str>,
    /// `CodegenContext::skip_scope_id`: inside a `mergeProps` call the scope
    /// pair is emitted once as a trailing argument, never per segment.
    skip_scope_id: bool,
    /// `(digit offset in `buf.code`, ordering key, slot number)` for
    /// every `_cache` index written so far. The shipped codegen takes a
    /// slot when it *reaches* the construct, so the ordering key is where
    /// that construct starts — the same place for every shape but
    /// `withMemo`, whose slot number prints after the body it wraps. Slot
    /// bodies here are rendered in source order, which is what the
    /// `_hoisted_N` numbering needs (the shipped lane assigns *those* in
    /// the transform), so the sites are recorded and the numbering is
    /// re-derived once the printed order is known.
    cache_sites: StdVec<(usize, usize, u32)>,
    /// The op-visit count at which prefixing first needed `_unref`
    /// (`u32::MAX`: never) — where the transform would register it.
    used_unref: core::cell::Cell<u32>,
    /// The shipped lane's `component_name`, for the self-reference flag
    /// on `resolveComponent`.
    component_name: Option<&'facts str>,
    /// The transform scope and codegen slot params the prefixer consults.
    scope: prefix::PrefixScope<'facts>,
}
