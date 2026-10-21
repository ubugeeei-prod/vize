//! The hoist-static pass: the P2-9 installment-6 port of
//! `crates/vize_atelier_core/src/steps/hoist_static.rs` (+
//! `hoist_static/props.rs`, `static_type.rs`) — the contract's
//! designated L2 **analysis** pass: a fact, not a mutation.
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
//!   children (they must survive as vnodes); L2 carries no comment ops
//!   (the L1 v1 scope's recorded face), so the fact is computed over a
//!   comment-free tree and the differential lane counts the class
//!   (`comments_elements`) instead of comparing it.
//! - **Vue dialect builtins** (`v-once`, `v-memo`, `v-show`, `v-html`,
//!   `v-text`, `v-cloak`) fail `hoistable_binding` like any non-`ui.bind`.
//!   The still-deferred builtin (`v-pre`) leaves the element looking
//!   cleaner to L2 than to the legacy lane; counted (`builtins_subtrees`).
//! - **The const rule** admits locals the expression binds and is still
//!   weaker than the shipped classifier on free names and `this`
//!   ([`vize_l2::analysis::constant_expression`]).
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

use vize_l0::pass::{NoRemarks, RemarkSink};
use vize_l0::side_table::SideTable;
use vize_l2::op::Namespace;

use super::walk::{PageWalk, assert_accounting};
use crate::lower::Lowered;

mod consts;
mod lattice;
mod remarks;

pub use remarks::{STATIC_PROPS, STATIC_SUBTREE};
pub use vize_l2::analysis::constant_expression::is_self_bound_constant as constant_for_hoist;
pub use vize_l2::analysis::static_facts::{DESC, NAME, StaticFacts, StaticLevel};

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
    run_remarked(lowered, &mut NoRemarks)
}

/// [`run`], explaining each owner's hoist eligibility through `remarks`
/// (the P3-13 vocabulary in `remarks.rs`). Under a detached sink
/// ([`NoRemarks`], or a pass-manager channel whose observer consumes no
/// remarks) this is [`run`] exactly: the remark path is compiled out.
///
/// # Panics
///
/// As [`run`].
#[must_use]
pub fn run_remarked<R: RemarkSink>(
    lowered: &mut Lowered<'_>,
    remarks: &mut R,
) -> SideTable<StaticFacts> {
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
        remarks,
    );
    assert_accounting(&walk, *op_count, NAME);
    facts
}
