//! The `v-for` pass: the P2-9 port of
//! `crates/vize_atelier_core/src/steps/v_for.rs` and its driver
//! halves (`src/lane/structural.rs::transform_v_for`, the
//! `enter_v_for_scope` calls in `src/lane/traverse.rs`),
//! re-expressed over `ui.for` owned regions.
//!
//! # What survived the port, and where the rest went
//!
//! Almost everything the old transform did is already absorbed — the
//! `v_for.rs` step file itself ports **whole** into the P2-8 lowering,
//! which is this installment's headline measurement:
//!
//! - **The value split** (`parse_for_expression`: separator find, alias
//!   split, strict grammar) — done at lowering (`lower/vfor.rs`,
//!   consuming the P2-5b splitter decision; `a in b in c` reads alias
//!   `a`, source `b in c`, and the undecomposable whole rides
//!   `Opaque(ForValue)` with pessimal semantics).
//! - **Grammar diagnostics** (`VForNoExpression`,
//!   `VForMalformedExpression`) — emitted at lowering with relief's
//!   exact text.
//! - **The node restructure** (take the element, wrap it in a
//!   `ForNode`) — a region-owning `ui.for` is built with its region from
//!   birth (`lower/forop.rs`), the `<template v-for>` unwrap included.
//! - **Scope recording** (`enter_v_for_scope`'s registration data) —
//!   the lowering mints a fresh [`ScopeTag`] per `ui.for` and records
//!   the authored simple-identifier bindings as `ScopeFacts` (P2-8).
//! - **Runtime-helper registration and the fragment/keying decisions**
//!   (`RenderList`/`OpenBlock`/..., keyed-vs-unkeyed fragments, the
//!   iterated element's `key` prop, `v-memo`'s placeholder params) —
//!   DOM-backend realization; they move with P2-11, and the iterated
//!   element's `key` deliberately **stays on its attribute surface**
//!   (unlike a v-if branch key, the legacy lane never lifts it — codegen
//!   reads it per vnode).
//! - **Source identifier prefixing** (`process_expression`) — the old
//!   lane's `transform_expression`, which stays put by the task's
//!   non-goal; P2-5b owns its future.
//!
//! What remains — and is this pass's body — is the **hygiene
//! consumption**: the lowering records the scope facts, and this pass is
//! where they become load-bearing. It re-derives the binding surface of
//! every `ui.for` with the same one-scanner rule the lowering used,
//! validates the recorded facts against it (entry present, fresh tag,
//! bindings byte-equal in position order, origins `Authored` at the
//! positions' exact spans), and publishes the consumed view as
//! [`ForFacts`] for the stages that read scopes next (the P2-5b rewriter
//! when S2 region structure starts feeding it; P2-11's params). The pass
//! **synthesizes no names**: an absent alias stays the zero-width escape
//! (inventing `_`/`__` placeholders is realization, P2-11's — and the
//! P2-8 record already scopes the first `ScopeOrigin::Synthesized`
//! producer to slot normalization).
//!
//! # Classification (the review point)
//!
//! **`MandatoryLowering`, barrier** — see [`DESC`].
//!
//! - *Why mandatory:* the scope-consumption contract is semantics, not
//!   speed. The old lane entered the v-for scope unconditionally at
//!   every tier because binding resolution inside the region depends on
//!   it; a tier that skipped this pass would hand later stages
//!   unvalidated facts and no consumed view — meaning, not polish.
//! - *Why `MandatoryLowering` and not `MandatoryDiagnostic`:* the pass
//!   emits **no user diagnostic at all** — both v-for user errors fire
//!   at lowering — so the diagnostic kind is simply false of it. What it
//!   does is establish an invariant later stages assume ("every
//!   `ui.for`'s scope facts agree with its binding surface, one fresh
//!   tag per site"), which is `MandatoryLowering`'s literal definition.
//!   The taxonomy tension is recorded rather than smoothed: this is a
//!   *preserving* mandatory pass (a "mandatory analysis" would be a
//!   fourth kind), the invariant is established by validation-and-fact
//!   rather than by rewrite, and the const pins keep the choice loud.
//! - *Why barrier:* forced by law 1 (mandatory passes never fuse), and
//!   independently true — tag freshness is a fact **across** every
//!   introduction site of the artifact, not the single-visit locality
//!   `Fusable` claims.
//! - `Preserved::ALL`: the pass mutates nothing — spans, attributes,
//!   scopes and provenance keys are all left exactly as lowered.
//!
//! # Facts and accounting
//!
//! Ids re-derive through the shared [`super::walk`], count-asserted
//! against the minted accounting on every run. [`ForFacts`] is keyed by
//! the `ui.for` op's id, one entry per op that has an id (dense over the
//! family, unlike [`super::vif`]'s sparse keys: every `ui.for` is an
//! introduction site). Every consumption leaves a provenance record
//! (`pass.v-for.scope`).

use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};
use vize_davinci::side_table::SideTable;
use vize_s0::{String, cstr};
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{ForOp, Op};
use vize_s2::provenance::ProvenanceRecord;
use vize_s2::scope::{ScopeBinding, ScopeFacts, ScopeOrigin, ScopeTag};

use super::walk::{PageWalk, assert_accounting, visit_ops};
use crate::lower::{Lowered, simple_identifier};

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "v-for";

/// The pass description — classification reasoning in the module docs.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    // The pass mutates nothing: it validates the recorded scopes and
    // publishes the consumed view beside the tree.
    Preserved::ALL,
);

/// One binding position's consumed name status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForName {
    /// The position's one enumerated simple-identifier name, validated
    /// against the lowering's `ScopeFacts` entry.
    Named(String),
    /// The position is authored but enumerates no name yet: a
    /// destructuring pattern (awaiting the #4365 enumeration seam), a
    /// foreign dialect, or the classified escape. Consumers pessimize —
    /// unknown names may bind here.
    Pending,
    /// The position was not authored (`key`/`index` grammar slots, or
    /// the zero-width value hole of an absent alias).
    Absent,
}

impl ForName {
    /// The provenance spelling: the name, `?` for pending, `-` for
    /// absent.
    fn spell(&self) -> &str {
        match self {
            ForName::Named(name) => name.as_str(),
            ForName::Pending => "?",
            ForName::Absent => "-",
        }
    }
}

/// One `ui.for`'s consumed scope view, positions in grammar order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForFacts {
    /// The introduction-site tag the lowering minted (validated fresh).
    pub tag: ScopeTag,
    /// The value position.
    pub value: ForName,
    /// The second position (object key).
    pub key: ForName,
    /// The third position (index).
    pub index: ForName,
}

/// Facts cross compile boundaries with their artifact (P1-11; the same
/// enforcement `Diagnostic` and `ProvenanceRecord` carry).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<ForName>();
    assert_owned::<ForFacts>();
};

/// 64-bit footprints, guarded like every node-size assert (the wasm32
/// lane is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<ForName>() == 24);
    assert!(core::mem::size_of::<ForFacts>() == 80);
};

/// Run the pass over `lowered`.
///
/// Walks the op tree once in page order (the shared re-derivation),
/// validates each `ui.for`'s recorded scope facts against its binding
/// surface, and publishes the consumed view. The tree is not touched;
/// provenance appends to `lowered`'s own channel.
///
/// # Panics
///
/// Panics on a broken accounting or hygiene law — a compiler bug, never
/// an input property: the re-derived page-order count disagreeing with
/// the minted accounting, a `ui.for` without a scope entry, a reused
/// tag, or recorded bindings that disagree with the binding surface.
#[must_use]
pub fn run(lowered: &mut Lowered<'_>) -> SideTable<ForFacts> {
    let Lowered {
        root,
        op_count,
        provenance,
        scopes,
        ..
    } = lowered;
    let mut facts = SideTable::new();
    let mut seen_tags: StdVec<ScopeTag> = StdVec::new();
    let mut walk = PageWalk::new();
    visit_ops(&mut walk, &mut root.ops, &mut |id, op| {
        if let Op::For(for_op) = op {
            consume_scope(scopes, provenance, &mut seen_tags, &mut facts, id, for_op);
        }
    });
    assert_accounting(&walk, *op_count, NAME);
    facts
}

/// Validate one `ui.for`'s recorded scope against its binding surface,
/// then publish the consumed view.
fn consume_scope(
    scopes: &SideTable<ScopeFacts>,
    provenance: &mut StdVec<ProvenanceRecord>,
    seen_tags: &mut StdVec<ScopeTag>,
    facts: &mut SideTable<ForFacts>,
    id: Option<NodeId>,
    for_op: &ForOp<'_>,
) {
    // Past id exhaustion the lowering attached no facts to attach to;
    // the tree stays total and only the keying stops (`Cx::attach_scope`).
    let Some(id) = id else {
        return;
    };
    let recorded = scopes.get(id).unwrap_or_else(|| {
        panic!("hygiene law broken: ui.for {id} has no scope entry — every ui.for is a binding-introduction site")
    });
    assert!(
        !seen_tags.contains(&recorded.tag),
        "hygiene law broken: ui.for {id} reuses scope tag {} — introduction sites mint fresh tags",
        recorded.tag,
    );
    seen_tags.push(recorded.tag);

    // An undecomposable value (`source` is the `ForValue` escape) puts
    // the whole binding under the pessimal opaque laws: the value
    // position's zero-width hole then means "unknowable", never
    // "absent" — unlike the *valid* absent alias (`v-for=" in xs"`),
    // whose source split cleanly and whose value truly binds nothing.
    let binding = &for_op.binding;
    let undecomposable = matches!(
        &binding.source,
        ExprRef::Opaque(opaque) if matches!(opaque.reason, OpaqueReason::ForValue)
    );
    let value = if undecomposable {
        ForName::Pending
    } else {
        position(Some(&binding.value))
    };
    let key = position(binding.key.as_ref());
    let index = position(binding.index.as_ref());

    // The re-derivation of what the lowering recorded: one entry per
    // named position, in position order, authored at the exact expression
    // span — the same one-scanner rule (`simple_identifier`), so the two
    // derivations can only disagree when one of them changed.
    let expected: StdVec<ScopeBinding> = [
        (&value, Some(&binding.value)),
        (&key, binding.key.as_ref()),
        (&index, binding.index.as_ref()),
    ]
    .into_iter()
    .filter_map(|(name, expr)| match (name, expr) {
        (ForName::Named(name), Some(expr)) => Some(ScopeBinding {
            name: name.clone(),
            origin: ScopeOrigin::Authored { span: expr.span() },
        }),
        _ => None,
    })
    .collect();
    assert!(
        recorded.bindings == expected,
        "hygiene law broken: ui.for {id} recorded bindings {:?} but its binding surface derives {:?}",
        recorded.bindings,
        expected,
    );

    provenance.push(ProvenanceRecord {
        rule: String::from("pass.v-for.scope"),
        node: Some(id),
        before: cstr!(
            "scope {} bindings={}",
            recorded.tag,
            recorded.bindings.len()
        ),
        after: cstr!(
            "fact value={} key={} index={}",
            value.spell(),
            key.spell(),
            index.spell()
        ),
        span: for_op.span,
    });
    facts.insert(
        id,
        ForFacts {
            tag: recorded.tag,
            value,
            key,
            index,
        },
    );
}

/// Classify one binding position.
///
/// A `Js` identifier is the enumerated name; any other authored text is
/// [`ForName::Pending`] (pattern, foreign, or escape — the zero-width
/// escape of an absent alias counts as absent, since no text was
/// authored at all).
fn position(expr: Option<&ExprRef<'_>>) -> ForName {
    let Some(expr) = expr else {
        return ForName::Absent;
    };
    if let Some(name) = simple_identifier(expr) {
        return ForName::Named(String::from(name));
    }
    if expr.source().is_empty() {
        return ForName::Absent;
    }
    ForName::Pending
}
