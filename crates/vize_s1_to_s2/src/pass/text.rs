//! The text pass: the P2-9 installment-4 port of
//! `crates/vize_atelier_core/src/steps/text.rs`,
//! re-expressed as the consumption of the lowering's condense-and-merge
//! work (`crate::lower::text`).
//!
//! # What survived the port, and where the rest went
//!
//! The old step file is **dead in the shipped lane** — exported from
//! `steps.rs`/`lib.rs`, called by nothing (measured: zero call sites
//! outside the re-exports). The behaviour it describes ships from two
//! other places, and the S2 port follows the living code, not the dead
//! file:
//!
//! - **Whitespace condensing** — parse time in the legacy lane
//!   (`vize_armature`'s `condense_whitespace`, the shipped `is_pre_tag`);
//!   absorbed into the P2-8 lowering here, because the remove-vs-condense
//!   rule reads *comments* as neighbours and only the lowering still
//!   sees them (`lower::text`'s module docs carry the full argument and
//!   the decision record).
//! - **Text/interpolation merging** — codegen time in the legacy lane
//!   (`codegen/children.rs`'s consecutive-run grouping); absorbed into
//!   the lowering for the same comment-boundary reason, and because
//!   P2-5b assigns the [`OpaqueReason::Compound`] class "only [to] the
//!   S1→S2 lowering". The pessimal laws
//!   (`vize_s2::expr::opaque`) bind every compound from its first
//!   byte: never constant, equal to nothing, byte-verbatim-or-refusal —
//!   so nothing downstream may compile from the rebuilt source text;
//!   [`TextFacts`] is the structured view realization compiles from.
//! - **Runtime-helper registration** (`ToDisplayString`,
//!   `CreateText`), the single-space `createTextVNode()` convention,
//!   and the array-forcing rules — DOM-backend business, they move with
//!   P2-11.
//!
//! What remains — and is this pass's body — is the **compound
//! consumption**: the lowering records each merged run's parts beside
//! the tree ([`crate::lower::Lowered::texts`]), and this pass is where
//! the record becomes load-bearing. Per compound op it validates
//! entry-present, part shape (≥ 2 parts, ≥ 1 dynamic — a lone node
//! never merges), span contiguity (parts tile the op's span exactly —
//! which is also the proof no comment or dropped byte was merged
//! across), and the rebuilt source byte-equal through the **same one
//! rebuild rule** ([`rebuild_source`], the #4365 one-scanner
//! discipline applied to spellings); then publishes the consumed view
//! as [`TextFacts`]. It also holds the region law the merge
//! establishes: adjacent text-family siblings are always separated by
//! dropped source bytes — mergeable adjacency does not survive the
//! lowering. A broken law panics as a compiler bug (the id-accounting
//! style), never a user diagnostic.
//!
//! # Classification (the review point)
//!
//! **`MandatoryLowering`, barrier** — see [`DESC`].
//!
//! - *Why mandatory:* the merged units are what DOM text emission
//!   compiles from — `createTextVNode` boundaries and their
//!   concatenated payloads are meaning, not polish — and the legacy
//!   lane computed both halves unconditionally at every tier (parse
//!   time and codegen time have no optimization levels). A tier that
//!   skipped this pass would hand realization unvalidated compounds.
//! - *Why `MandatoryLowering` and not `MandatoryDiagnostic`:* the pass
//!   emits **no user diagnostic** — condensing and merging produce
//!   none in the legacy lane either — so the diagnostic kind is simply
//!   false of it; it establishes an invariant later stages assume
//!   ("every compound's parts tile its span and rebuild to its
//!   source"), `MandatoryLowering`'s literal definition. The
//!   installment-2 taxonomy tension applies verbatim: this is a
//!   *preserving* mandatory pass, and the const pins keep the choice
//!   loud.
//! - *Why barrier:* forced by law 1, and independently true — the
//!   adjacency law reads **across** sibling ops of every region, not
//!   the single-visit locality `Fusable` claims.
//! - `Preserved::ALL`: the pass mutates nothing — the tree, spans,
//!   scopes and provenance keys are left exactly as lowered.
//!
//! # Facts and accounting
//!
//! The pass drives its own shaped recursion over the shared
//! [`super::walk::PageWalk`] (flat visitation cannot hand a region its
//! sibling pairs for the adjacency law); the mint arithmetic keeps its
//! one home in `pass/walk.rs`, and the recursion ends at the same
//! accounting assertion. [`TextFacts`] is keyed by the compound op's
//! id — dense over the compound family. Every consumption leaves a
//! provenance record (`pass.text.compound`).
//!
//! [`OpaqueReason::Compound`]: vize_s2::expr::OpaqueReason::Compound

use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};
use vize_davinci::side_table::SideTable;
use vize_s0::{String, cstr, ensure_sufficient_stack};
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::Op;
use vize_s2::provenance::ProvenanceRecord;

use super::walk::{PageWalk, assert_accounting};
use crate::lower::{Lowered, TextPart, TextParts, rebuild_source};

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "text";

/// The pass description — classification reasoning in the module docs.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    // The pass mutates nothing: it validates the recorded runs and
    // publishes the consumed view beside the tree.
    Preserved::ALL,
);

/// One compound op's consumed view: the merged run's parts, validated
/// against the op they key (span tiling, the rebuild law). What DOM
/// realization (P2-11) compiles text units from — never the opaque
/// source, which the pessimal laws confine to display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFacts {
    /// The parts, in document order ([`TextPart`]'s field docs).
    pub parts: StdVec<TextPart>,
}

/// Facts cross compile boundaries with their artifact (P1-11).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<TextFacts>();
};

/// 64-bit footprint, guarded like every node-size assert (the wasm32
/// lane is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<TextFacts>() == 24);

/// Run the pass over `lowered`.
///
/// Walks the op tree once in page order (the shared re-derivation,
/// shaped so every region hands its sibling pairs to the adjacency
/// law), validates each compound op's recorded parts, and publishes the
/// consumed view. The tree is not touched; provenance appends to
/// `lowered`'s own channel.
///
/// # Panics
///
/// Panics on a broken accounting or compound law — a compiler bug,
/// never an input property: the re-derived count disagreeing with the
/// minted accounting, a compound op without a parts entry (or a parts
/// entry keying no compound op), parts that fail the shape/tiling/
/// rebuild laws, or mergeable adjacency surviving the lowering.
#[must_use]
pub fn run(lowered: &mut Lowered<'_>) -> SideTable<TextFacts> {
    let Lowered {
        root,
        op_count,
        provenance,
        texts,
        ..
    } = lowered;
    let mut facts = SideTable::new();
    let mut walk = PageWalk::new();
    visit_region(&mut walk, &root.ops, texts, provenance, &mut facts);
    assert_accounting(&walk, *op_count, NAME);
    assert!(
        facts.len() == texts.len(),
        "compound law broken: the lowering recorded {} text runs but the tree holds {} compound ops",
        texts.len(),
        facts.len(),
    );
    facts
}

/// The shaped recursion: one region at a time, so the adjacency law
/// sees sibling pairs; ids mint through the one shared arithmetic.
fn visit_region(
    walk: &mut PageWalk,
    ops: &[Op<'_>],
    texts: &SideTable<TextParts>,
    provenance: &mut StdVec<ProvenanceRecord>,
    facts: &mut SideTable<TextFacts>,
) {
    ensure_sufficient_stack(|| visit_region_guarded(walk, ops, texts, provenance, facts));
}

fn visit_region_guarded(
    walk: &mut PageWalk,
    ops: &[Op<'_>],
    texts: &SideTable<TextParts>,
    provenance: &mut StdVec<ProvenanceRecord>,
    facts: &mut SideTable<TextFacts>,
) {
    check_adjacency(ops);
    for op in ops {
        let id = walk.mint();
        match op {
            Op::Element(element) => {
                for _ in element.bindings.iter() {
                    let _ = walk.mint();
                }
                visit_region(walk, &element.children.ops, texts, provenance, facts);
            }
            Op::Component(component) => {
                for _ in component.bindings.iter() {
                    let _ = walk.mint();
                }
                visit_region(walk, &component.children.ops, texts, provenance, facts);
            }
            Op::Text(_) => {}
            Op::Interpolation(interpolation) => {
                consume_compound(
                    texts,
                    provenance,
                    facts,
                    id,
                    &interpolation.expression,
                    interpolation.span,
                );
            }
            Op::If(if_op) => {
                for branch in if_op.branches.iter() {
                    visit_region(walk, &branch.region.ops, texts, provenance, facts);
                }
            }
            Op::For(for_op) => visit_region(walk, &for_op.region.ops, texts, provenance, facts),
            Op::Slot(slot) => {
                for _ in slot.bindings.iter() {
                    let _ = walk.mint();
                }
                visit_region(walk, &slot.fallback.ops, texts, provenance, facts);
            }
        }
    }
}

/// The adjacency law: two text-family siblings always have dropped
/// source bytes between them (a comment, a removed run, recovered
/// junk) — zero-gap adjacency is exactly what the lowering merges, so
/// its survival is a compiler bug.
fn check_adjacency(ops: &[Op<'_>]) {
    for pair in ops.windows(2) {
        let (prev, next) = match pair {
            [prev, next] => (prev, next),
            _ => continue,
        };
        let prev_end = match prev {
            Op::Text(text) => text.span.end,
            Op::Interpolation(interpolation) => interpolation.span.end,
            _ => continue,
        };
        let next_start = match next {
            Op::Text(text) => text.span.start,
            Op::Interpolation(interpolation) => interpolation.span.start,
            _ => continue,
        };
        assert!(
            prev_end < next_start,
            "compound law broken: adjacent text-family ops touch at offset {prev_end} — \
             mergeable adjacency must not survive the lowering",
        );
    }
}

/// Validate one compound op's recorded parts and publish the fact.
fn consume_compound(
    texts: &SideTable<TextParts>,
    provenance: &mut StdVec<ProvenanceRecord>,
    facts: &mut SideTable<TextFacts>,
    id: Option<NodeId>,
    expression: &ExprRef<'_>,
    span: vize_s0::Span,
) {
    let ExprRef::Opaque(opaque) = expression else {
        return;
    };
    if opaque.reason != OpaqueReason::Compound {
        return;
    }
    // Past id exhaustion the lowering had no key to attach the parts to;
    // the tree stays total and only the keying stops.
    let Some(id) = id else {
        return;
    };
    let recorded = texts.get(id).unwrap_or_else(|| {
        panic!("compound law broken: compound op {id} has no recorded parts entry")
    });
    let parts = &recorded.parts;
    let dynamic = parts.iter().filter(|part| part.dynamic).count();
    assert!(
        parts.len() >= 2 && dynamic >= 1,
        "compound law broken: op {id} records {} parts ({dynamic} dynamic) — a lone node never merges",
        parts.len(),
    );
    assert!(
        parts
            .windows(2)
            .all(|pair| pair[0].dynamic || pair[1].dynamic),
        "compound law broken: op {id} holds adjacent static parts — the lowering fuses them",
    );
    let mut cursor = span.start;
    for part in parts {
        assert!(
            part.span.start == cursor,
            "compound law broken: op {id}'s parts do not tile its span (gap at {cursor})",
        );
        cursor = part.span.end;
    }
    assert!(
        cursor == span.end,
        "compound law broken: op {id}'s parts end at {cursor}, its span at {}",
        span.end,
    );
    let rebuilt = rebuild_source(parts);
    assert!(
        rebuilt.as_str() == opaque.source,
        "compound law broken: op {id}'s source {:?} does not rebuild from its parts {:?}",
        opaque.source,
        rebuilt,
    );

    provenance.push(ProvenanceRecord {
        rule: String::from("pass.text.compound"),
        node: Some(id),
        before: cstr!("parts={}", parts.len()),
        after: cstr!("fact static={} dynamic={dynamic}", parts.len() - dynamic),
        span,
    });
    facts.insert(
        id,
        TextFacts {
            parts: parts.clone(),
        },
    );
}
