//! The hoist-static pass: the P2-9 installment-6 port of
//! `crates/vize_atelier_core/src/transforms/hoist_static.rs` (+
//! `hoist_static/props.rs`, `static_type.rs`) — the contract's
//! designated S2 **analysis** pass: a fact, not a mutation.
//!
//! # What the port measured (the absorbed-vs-pass-body split)
//!
//! Unlike installments 4–5's dead step files, `hoist_static` is **live
//! in the shipped lane**: `lane.rs:295` runs it after traversal
//! (`atelier.transform.hoist_static`) and `codegen/children.rs` reads
//! `is_static_node` besides. The living code splits three ways:
//!
//! - **The analysis** — `static_type.rs`'s whole-subtree lattice
//!   (`StaticType`) and `props.rs`'s per-prop hoistability — is this
//!   pass's body, published per owner as [`StaticFacts`].
//! - **The decisions** — which node whole-hoists, which props-hoist —
//!   are *position- and option-dependent* (`is_root`, the inherited
//!   `hoist_static_vnodes` flag, `hoist_static`/`inline`/`scope_id`
//!   options), so they belong to the stage that holds position and
//!   options: DOM realization (P2-11). The differential lane replays
//!   the shipped decision procedure over these facts to prove they
//!   *determine* the shipped decisions.
//! - **The realization** — `VNodeCall` construction, scoped-CSS
//!   attribute injection, `_hoisted_N` emission, `should_use_block` /
//!   `count_dynamic_children` — moves whole to P2-11.
//!
//! # Classification (the review point): the series' first `Optional`
//!
//! **`Optional`, `Fusable`, `Preserved::ALL`** — see [`DESC`], pinned
//! in `pass.rs`.
//!
//! - *Why optional:* skipping loses only optimization facts. The
//!   shipped lane proves it about itself: `TransformOptions::default()`
//!   ships `hoist_static: false` (the DOM layer opts in), the transform
//!   emits **no diagnostic** from this step, and un-hoisted output is
//!   correct output. `PassKind::Optional`'s definition — "may not
//!   change what the program means, only how well it is expressed" —
//!   is literally this pass.
//! - *Why fusable:* the lattice is a synthesized attribute — one
//!   post-order visit, each node's fact computed from its own surface
//!   and its children's already-computed facts, no sibling lookahead,
//!   no fixpoint — which is `Fusability::Fusable`'s own definition
//!   ("single-visit, local, synthesized-attribute style").
//! - What the fusion machinery then actually does is pinned in
//!   `pass.rs`: the pass forms the pipeline's **first non-barrier
//!   group** — a singleton, because every neighbour is a barrier — so
//!   `group_count()` rises to 6 and fusion still buys nothing until a
//!   second fusable pass lands beside it.
//!
//! # What the facts cannot see (recorded, counted)
//!
//! - **Comments**: the legacy lattice refuses to hoist across comment
//!   children (they must survive as vnodes); S2 carries no comment ops
//!   (the S1 v1 scope's recorded face), so the fact is computed over a
//!   comment-free tree and the differential lane counts the class
//!   (`comments_elements`) instead of comparing it.
//! - **Deferred builtins** (`v-html`, `v-show`, `v-cloak`, `v-pre`):
//!   still `defer.*` at lowering, so an element carrying one looks
//!   cleaner to S2 than to the legacy lane; counted
//!   (`builtins_subtrees`). `v-once` / `v-memo` now lower as
//!   `vue.once` / `vue.memo` and fail `hoistable_binding` like any
//!   non-`ui.bind`.
//! - **The const rule** is deliberately weaker than the shipped
//!   classifier's ([`consts::constant_for_hoist`]'s module docs — the
//!   pessimal law's first real consumer lives there).
//!
//! # Facts and accounting
//!
//! Dense over `ui.element` + `ui.component` ops ([`StaticFacts`], keyed
//! by page-order id); outlets need no entry (`has_static_props` is
//! false for them by rule, their level `NotStatic` by kind). The pass
//! drives its own shaped recursion over the shared
//! [`super::walk::PageWalk`] (post-order: a parent's lattice value is
//! computed from its children's), mints through the one arithmetic,
//! and ends at the same accounting assertion. Every fact leaves a
//! provenance record (`pass.hoist-static.fact`).

use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};
use vize_davinci::side_table::SideTable;
use vize_disegno::op::Namespace;

use super::walk::{PageWalk, assert_accounting};
use crate::lower::Lowered;

mod consts;
mod lattice;

pub use consts::constant_for_hoist;

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "hoist-static";

/// The pass description — classification reasoning in the module docs;
/// the series' first `Optional` and first `Fusable`.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::Optional,
    Fusability::Fusable,
    // Pure analysis: the tree, spans, scopes and every other fact are
    // left exactly as found.
    Preserved::ALL,
);

/// The per-node constant level — the old `static_type.rs` ladder
/// (`StaticType`), re-expressed as a published fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticLevel {
    /// Dynamic surface or dynamic descendants; never hoistable whole.
    NotStatic,
    /// Static but for interpolation children — root prop-hoisting still
    /// applies, whole-hoisting never does.
    HasDynamicText,
    /// The whole subtree is static: whole-hoistable where position
    /// allows.
    FullyStatic,
}

/// One owner's static-analysis facts: the lattice value plus the three
/// subtree predicates the shipped decision procedure reads
/// (`has_static_props`, `has_only_static_nested_children`,
/// `has_only_native_element_descendants` — the position/option gates
/// stay with the reader).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticFacts {
    /// The constant level ([`StaticLevel`]).
    pub level: StaticLevel,
    /// `has_static_props`: a non-empty, fully hoistable props surface
    /// (never true for outlets).
    pub props_hoistable: bool,
    /// `has_only_static_nested_children`: non-empty children, all in
    /// the static-nested class (interpolations count as static here —
    /// the shipped rule).
    pub nested_static: bool,
    /// `has_only_native_element_descendants`: every descendant is a
    /// native element or text-family node (the `inline` root arm's
    /// gate).
    pub native_descendants: bool,
    /// The owner stands in a foreign markup namespace — its own for a
    /// `ui.element`, the *inherited context* for a `ui.component`
    /// (which deliberately carries no namespace of its own): the
    /// shipped `ns != Html` props-hoist arm's input, first seen live on
    /// a `<TransitionGroup>` inside `<svg>` (directus `arrows.vue`,
    /// caught by the differential lane's first corpus run).
    pub foreign: bool,
}

/// Facts cross compile boundaries with their artifact (P1-11).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<StaticFacts>();
};

/// Five bytes on every width: one tag byte plus four bools.
const _: () = assert!(core::mem::size_of::<StaticFacts>() == 5);

/// Run the pass over `lowered`.
///
/// One post-order walk in page order; the tree is not touched, facts
/// publish beside it, provenance appends to `lowered`'s own channel.
///
/// # Panics
///
/// Panics only on broken id accounting — a compiler bug by the id law,
/// never an input property.
#[must_use]
pub fn run(lowered: &mut Lowered<'_>) -> SideTable<StaticFacts> {
    let Lowered {
        root,
        op_count,
        provenance,
        ..
    } = lowered;
    let mut facts = SideTable::new();
    let mut walk = PageWalk::new();
    lattice::visit_region(
        &mut walk,
        &root.ops,
        Namespace::Html,
        provenance,
        &mut facts,
    );
    assert_accounting(&walk, *op_count, NAME);
    facts
}
