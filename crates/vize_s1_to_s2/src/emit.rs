//! S2 → DOM render-function emission (P2-11).
//!
//! The unpublished home for the new DOM backend: `vize_atelier_dom` is
//! published and cannot name this crate in its release graph (the
//! installment-1 publish-gate measurement). Dual-run lives in
//! atelier_dom **test space** as a stripped-on-publish dev-dep, the
//! P2-9 carve-out. It writes JS directly from S2 ops, without relief codegen-nodes.
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
//! `v-text`, `v-bind` modifiers, dynamic `v-bind` keys / modifiers, and
//! Vue 2 pipe filters legalized by `legacy-sugar`.
//! The old lane stays the shipped compile path; [`super::DOM_LANE_FLAG`]
//! is named here and *read* in the atelier_dom witness.

mod buf;
mod builtin;
mod children;
mod component;
mod create_slots;
mod create_slots_walk;
mod cx;
mod directive;
mod entity;
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
mod outlet;
mod outlet_props;
mod props;
mod props_bind;
mod props_class;
mod props_object;
mod props_object_merge;
mod props_static;
mod props_value;
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
use vize_davinci::diagnostic::Severity;
use vize_davinci::id::NodeId;
use vize_davinci::pass::BudgetObserver;
use vize_davinci::side_table::SideTable;
use vize_s0::{Allocator, String};
use vize_s1::parse;
use vize_s2::op::{ElementOp, ForOp, IfOp, Namespace};
use vize_s2::scope::ScopeFacts;

use crate::lower::{ForWrapper, LegacyCaps, Lowered, WrapperKeys, lower_with_caps};
use crate::pass::walk::PageWalk;
use crate::pass::{S2Facts, run_transform};

use self::buf::Buf;
pub use self::error::{EmitError, UnsupportedReason, UnsupportedRefusal};
use self::fragment::emit_root;
use self::helper::Helper;

fn emit_if_op(cx: &mut EmitCx<'_>, if_op: &IfOp<'_>, id: Option<NodeId>) -> Result<(), EmitError> {
    vif::emit_if(cx, if_op, id)
}

fn emit_if_branch_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    key: &str,
) -> Result<(), EmitError> {
    vnode::emit_if_branch_element(cx, element, key)
}

fn emit_for_op(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    id: Option<NodeId>,
    fragment_key: Option<&str>,
) -> Result<(), EmitError> {
    vfor::emit_for(cx, for_op, id, fragment_key)
}

fn emit_for_item_call(
    cx: &mut EmitCx<'_>,
    element: &ElementOp<'_>,
    id: Option<NodeId>,
    stable: bool,
    key: Option<&str>,
) -> Result<(), EmitError> {
    vfor_item::emit_element(cx, element, id, stable, key)
}

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
}

/// One DOM render module, split the way the shipped codegen splits it
/// (`CodegenResult::{preamble, code}`) so a dual-run can compare each
/// half and the concatenated form the DOM snapshots use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomEmit {
    /// Helper destructure (`const { … } = Vue\n`).
    pub preamble: String,
    /// The `function render(…)` body, no trailing newline after `}`.
    pub code: String,
}

impl DomEmit {
    /// `preamble + "\\n" + code` — the same concatenation
    /// `vize_atelier_dom` snapshots pin.
    #[must_use]
    pub fn assembled(&self) -> String {
        let mut out = self.preamble.clone();
        out.push('\n');
        out.push_str(self.code.as_str());
        out
    }
}

/// Emit a DOM render function from an already-lowered (and typically
/// transformed) S2 artifact. `facts` is the transform product compounds
/// compile from.
pub fn emit_dom(lowered: &Lowered<'_>, facts: &S2Facts) -> Result<DomEmit, EmitError> {
    if lowered
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(EmitError::Diagnostics);
    }
    let static_cache = static_cache::enabled(&lowered.root, facts, &lowered.wrappers);
    let mut cx = EmitCx {
        buf: Buf::new(),
        source: lowered.source,
        facts,
        scopes: &lowered.scopes,
        wrappers: &lowered.wrappers,
        for_wrappers: &lowered.for_wrappers,
        walk: PageWalk::new(),
        scope_names: StdVec::new(),
        if_branch_key: 0,
        once_cache_index: 0,
        once_depth: 0,
        once_element_depth: 0,
        in_v_for: false,
        conditional_v_for_item: false,
        template_for_item_single_root: false,
        template_for_item_root_id: None,
        template_if_branch_root: false,
        template_if_for_branch_root: false,
        suppress_template_for_child_key: false,
        skip_memo: false,
        slot_param_depth: 0,
        hoist_static_vnodes: false,
        transition_slot_root: false,
        static_cache,
        parent_ns: Namespace::Html,
    };
    let filters = &facts.legacy.filters;
    if facts.legacy.filter_helper_precedes_components {
        cx.buf.prefer(Helper::ResolveFilter);
    }
    let mut helper_walk = PageWalk::new();
    helper_preference::prefer_helpers(
        &mut cx.buf,
        facts,
        &lowered.for_wrappers,
        &mut helper_walk,
        &lowered.root,
    );
    fragment::prefer_root_fragment(&mut cx.buf, &lowered.root);
    cx.buf
        .push("function render(_ctx, _cache, $props, $setup, $data, $options) {");
    cx.buf.indent();
    cx.buf.newline();
    let names = component::collect_names(&lowered.root);
    let dirs = directive::collect_names(&lowered.root);
    if !names.is_empty() {
        component::emit_resolves(&mut cx, &names);
    }
    if !dirs.is_empty() {
        directive::emit_resolves(&mut cx, &dirs);
    }
    if !filters.is_empty() {
        filter::emit_resolves(&mut cx, filters);
    }
    if !names.is_empty() || !dirs.is_empty() || !filters.is_empty() {
        cx.buf.newline();
    }
    cx.buf.push("return ");
    emit_root(&mut cx, &lowered.root)?;
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}");
    Ok(DomEmit {
        preamble: cx.buf.preamble(),
        code: cx.buf.code,
    })
}

/// Parse → lower → S2 transform → emit. The comparator's one-shot entry
/// so atelier_dom tests do not re-derive the pipeline.
pub fn emit_dom_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<DomEmit, EmitError> {
    emit_dom_source_with_caps(allocator, source, LegacyCaps::VUE3)
}

/// [`emit_dom_source`] under an explicit Vue dialect capability set.
pub fn emit_dom_source_with_caps<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    caps: LegacyCaps,
) -> Result<DomEmit, EmitError> {
    let (tree, errors) = parse(allocator, source);
    let mut lowered = lower_with_caps(allocator, &tree, &errors, caps);
    let mut budget = BudgetObserver::new();
    let facts = run_transform(&mut lowered, &mut budget);
    emit_dom(&lowered, &facts)
}
