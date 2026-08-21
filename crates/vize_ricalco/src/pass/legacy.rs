//! The legacy pass (P2-9 series 7, `_legacy` only): the filter-site
//! consumption and the asset-registration fact.
//!
//! # What survived the port, and where the rest went
//!
//! - **Absorbed by lowering** (`lower::legacy`): the three desugars
//!   (`.sync`, `slot-scope`/`scope`, the v-on event sugar) — the live
//!   lane's own pre-transform rewrites, mirrored — and the filter
//!   *split* itself, recorded per site beside the tree.
//! - **DOM realization**: the shipped rewrite's product — the
//!   `_filter_<name>(base,args)` call text, `toValidAssetId`, the
//!   `_resolveFilter` helper and the `const _filter_x = …` preamble
//!   (`codegen/root.rs`) — everything that *emits*.
//! - **The pass body (here)**: the transform-lane state the live lane
//!   keeps on its context — `ctx.add_filter`'s first-seen, deduplicated
//!   asset registration (`TransformContext::filters` →
//!   `RootNode::filters`) — published as [`LegacyFacts::assets`], plus
//!   the site laws: every recorded split keys a live filter op and
//!   every filter op keys a recorded split (count-matched both ways),
//!   and re-splitting the op's opaque source through the one mirrored
//!   splitter reproduces the recorded parts byte-equally (the #4365
//!   one-rule-two-sides discipline).
//!
//! # Classification (the review point)
//!
//! **`MandatoryLowering`, barrier, `Preserved::ALL`.**
//!
//! - *Mandatory:* under a filter dialect the asset list is meaning —
//!   the live lane registers unconditionally at every tier; skipping
//!   loses what realization resolves filters from.
//! - *Lowering, not Diagnostic:* the pass emits no user diagnostic
//!   (the live rewrite bails silently on malformed names — mirrored at
//!   the lowering's split); it establishes the canonical asset order
//!   later stages assume. The pass **preserves** — the recorded
//!   preserving-mandatory taxonomy tension, fourth occurrence,
//!   const-pinned like its three predecessors.
//! - *Barrier:* law 1 forces it, and independently first-seen asset
//!   order is a fact across every filter site of the artifact, not
//!   single-visit locality.
//!
//! The pass rides only in [`TRANSFORM_LEGACY`] — the per-dialect
//! pipeline superset — so the plain [`TRANSFORM`] pipeline (and every
//! walk-count pin over it) is byte-identical in both feature shapes.
//!
//! [`TRANSFORM_LEGACY`]: pipeline::TRANSFORM_LEGACY
//! [`TRANSFORM`]: super::TRANSFORM

use alloc::vec::Vec as StdVec;

use vize_carton::{String, cstr};
use vize_davinci::id::NodeId;
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};
use vize_davinci::side_table::SideTable;
use vize_disegno::expr::{ExprRef, OpaqueReason};
use vize_disegno::op::{BindingOp, Op};
use vize_disegno::provenance::ProvenanceRecord;

use super::walk::{PageWalk, assert_accounting};
use crate::lower::Lowered;
use crate::lower::legacy::{FilterParts, filter_split};

#[path = "legacy/pipeline.rs"]
mod pipeline;

pub use pipeline::{TRANSFORM_LEGACY, TRANSFORM_LEGACY_PASSES, run_transform_legacy};

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "legacy";

/// The pass description — classification reasoning in the module docs.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    Preserved::ALL,
);

/// The validated view of one filter site, keyed by its op's page-order
/// id: the base expression and the ordered filter names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterFacts {
    /// The base expression text, trimmed.
    pub base: String,
    /// The chain's filter names, outermost last.
    pub names: StdVec<String>,
}

/// The legacy pass's product: the artifact-level asset registration and
/// the per-site consumed views.
#[derive(Debug, Default)]
pub struct LegacyFacts {
    /// Filter asset names in first-seen page order, deduplicated — the
    /// live lane's `ctx.add_filter` registration, preserved as a fact.
    pub assets: StdVec<String>,
    /// Per-site consumed views, keyed by the `vue.filter` /
    /// `ui.bind` op's page-order id.
    pub sites: SideTable<FilterFacts>,
}

const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<FilterFacts>();
    assert_owned::<LegacyFacts>();
};

/// Run the legacy pass over one lowered artifact.
///
/// # Panics
///
/// Panics when a site law is broken — recorded splits and live filter
/// ops must match one-to-one, and the re-derived split must reproduce
/// the recorded parts — a compiler bug by construction, never an input
/// property (the id-accounting style).
pub fn run(lowered: &mut Lowered<'_>) -> LegacyFacts {
    let mut facts = LegacyFacts::default();
    let mut consumed: u32 = 0;
    let mut walk = PageWalk::new();
    visit_region(
        &mut walk,
        &lowered.root.ops,
        &lowered.filters,
        &mut lowered.provenance,
        &mut facts,
        &mut consumed,
    );
    assert_accounting(&walk, lowered.op_count, NAME);
    assert!(
        consumed as usize == lowered.filters.len(),
        "filter-site law broken: the lowering recorded {} splits but the tree holds {} filter ops",
        lowered.filters.len(),
        consumed,
    );
    facts
}

fn visit_region(
    walk: &mut PageWalk,
    ops: &[Op<'_>],
    recorded: &SideTable<FilterParts>,
    provenance: &mut StdVec<ProvenanceRecord>,
    facts: &mut LegacyFacts,
    consumed: &mut u32,
) {
    for op in ops {
        let id = walk.mint();
        match op {
            Op::Element(element) => {
                visit_bindings(
                    walk,
                    &element.bindings,
                    recorded,
                    provenance,
                    facts,
                    consumed,
                );
                visit_region(
                    walk,
                    &element.children.ops,
                    recorded,
                    provenance,
                    facts,
                    consumed,
                );
            }
            Op::Component(component) => {
                visit_bindings(
                    walk,
                    &component.bindings,
                    recorded,
                    provenance,
                    facts,
                    consumed,
                );
                visit_region(
                    walk,
                    &component.children.ops,
                    recorded,
                    provenance,
                    facts,
                    consumed,
                );
            }
            Op::Text(_) | Op::Interpolation(_) => {}
            Op::VueFilter(filter) => {
                let ExprRef::Opaque(opaque) = &filter.expression else {
                    panic!(
                        "filter-site law broken: a vue.filter op's expression is not the pessimal escape"
                    );
                };
                consume(id, opaque, recorded, provenance, facts, consumed);
            }
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    visit_region(
                        walk,
                        &branch.region.ops,
                        recorded,
                        provenance,
                        facts,
                        consumed,
                    );
                }
            }
            Op::For(for_op) => {
                visit_region(
                    walk,
                    &for_op.region.ops,
                    recorded,
                    provenance,
                    facts,
                    consumed,
                );
            }
            Op::Slot(slot) => {
                visit_bindings(walk, &slot.bindings, recorded, provenance, facts, consumed);
                visit_region(
                    walk,
                    &slot.fallback.ops,
                    recorded,
                    provenance,
                    facts,
                    consumed,
                );
            }
        }
    }
}

fn visit_bindings(
    walk: &mut PageWalk,
    bindings: &[BindingOp<'_>],
    recorded: &SideTable<FilterParts>,
    provenance: &mut StdVec<ProvenanceRecord>,
    facts: &mut LegacyFacts,
    consumed: &mut u32,
) {
    for binding in bindings {
        let id = walk.mint();
        if let BindingOp::Bind(bind) = binding
            && let Some(ExprRef::Opaque(opaque)) = &bind.value
            && opaque.reason == OpaqueReason::LegacyFilter
        {
            consume(id, opaque, recorded, provenance, facts, consumed);
        }
    }
}

/// Consume one filter site: the entry-present law, the re-split law,
/// and the asset registration.
fn consume(
    id: Option<NodeId>,
    opaque: &vize_disegno::expr::OpaqueExpr<'_>,
    recorded: &SideTable<FilterParts>,
    provenance: &mut StdVec<ProvenanceRecord>,
    facts: &mut LegacyFacts,
    consumed: &mut u32,
) {
    let Some(id) = id else {
        return;
    };
    assert!(
        opaque.reason == OpaqueReason::LegacyFilter,
        "filter-site law broken: a vue.filter op carries reason {:?}",
        opaque.reason,
    );
    let Some(parts) = recorded.get(id) else {
        panic!("filter-site law broken: op {id:?} has no recorded split");
    };
    // The one-rule-two-sides law: re-splitting the opaque source must
    // reproduce the recorded parts byte-equally.
    let derived = filter_split(opaque.source)
        .unwrap_or_else(|| panic!("filter-site law broken: op {id:?}'s source no longer splits"));
    assert!(
        derived == *parts,
        "filter-site law broken: op {id:?}'s re-derived split diverged from the recorded parts",
    );
    for segment in &parts.segments {
        if !facts.assets.contains(&segment.name) {
            facts.assets.push(segment.name.clone());
        }
    }
    provenance.push(ProvenanceRecord {
        rule: String::from("pass.legacy.filter"),
        node: Some(id),
        before: String::from(opaque.source),
        after: cstr!("base {} segments={}", parts.base, parts.segments.len()),
        span: opaque.span,
    });
    facts.sites.insert(
        id,
        FilterFacts {
            base: parts.base.clone(),
            names: parts.segments.iter().map(|s| s.name.clone()).collect(),
        },
    );
    *consumed += 1;
}
