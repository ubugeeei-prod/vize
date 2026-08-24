//! The `v-if` pass: the P2-9 port of
//! `crates/vize_atelier_core/src/transforms/v_if.rs` and the
//! sibling-mutation driver in `src/transform/structural.rs`, re-expressed
//! over `ui.if` owned regions.
//!
//! # What survived the port, and where the rest went
//!
//! The old transform did four things. Three are already absorbed by the
//! region design and the P2-8 lowering, which is the point of porting
//! this transform first:
//!
//! - **Chain grouping (the enter/exit sibling mutation)** — done at
//!   lowering: a region-owning `ui.if` is built with its branches from
//!   birth (`vize_ricalco::lower::structural`), so no pass revisits a
//!   sibling list.
//! - **Grammar diagnostics** (`v-if` missing expression, orphan
//!   `v-else`) — emitted at lowering with relief's exact message text.
//! - **Runtime-helper registration** — DOM-backend business; it moves
//!   with P2-11, not with the neutral S2 transform.
//!
//! What remains — and is this pass's body — is the branch-key half:
//! `extract_key_prop`'s lift of the authored `key` off the branch's
//! carrier into a per-branch fact, and the duplicate-key diagnostic
//! (vuejs/core #13881, `ErrorCode::VIfSameKey`). The element/binding
//! installment (series 5) completed the surface: a dynamic `:key` rides
//! `ui.bind` and is extracted beside the static arm ([`keys`]), slot
//! outlets carry an attribute surface of their own, and a
//! `<template v-if>` wrapper's key — captured at lowering into
//! [`WrapperKeys`](crate::lower::WrapperKeys), since the wrapper unwraps
//! before any pass runs — folds into the same fact. The collision check
//! is kind-blind text equality, exactly the legacy
//! `extract_key_value_str` under the default dialect (a bare `key` or a
//! valueless dynamic spelling never collides).
//!
//! # Classification (the review point)
//!
//! **`MandatoryLowering`, barrier** — see [`DESC`].
//!
//! - *Why mandatory:* skipping it changes meaning, not speed. The key
//!   fact is what keyed branch reuse compiles from, and the duplicate-key
//!   error is a user-visible diagnostic every optimization tier must
//!   emit (the old transform ran unconditionally).
//! - *Why `MandatoryLowering` and not `MandatoryDiagnostic`:* the pass
//!   **mutates the artifact** — the `key` attribute leaves the element's
//!   syntactic surface and becomes a semantic fact — and it is the pass
//!   that establishes the canonical `ui.if` form the verifier's
//!   canonical set (S2V004–S2V006) then holds; a diagnostic pass must
//!   preserve, and this one does not.
//! - *Why not `Optional`:* an optional pass may not change what the
//!   program means; dropping this one loses an error and a fact.
//! - *Why barrier:* forced by law 1 (mandatory passes never fuse), and
//!   independently true — the collision check compares facts **across**
//!   an op's branches, which is not the single-visit locality `Fusable`
//!   claims.
//!
//! # Facts and accounting
//!
//! Ids are positional (page order), so the pass re-derives them through
//! the shared [`super::walk`] — op line, attached bindings, then
//! children — and asserts its count against the lowering's minted
//! accounting on every run. [`IfFacts`] is keyed by the `ui.if` op's id;
//! entries exist only for ops with at least one extracted key
//! (sparse-table discipline). Every extraction and every diagnostic
//! leaves a provenance record (`pass.v-if.branch-key` /
//! `error.v-if-same-key`); `before` carries the folio-normalized
//! `key="value"` spelling because the arena attribute does not retain
//! its authored quoting.

use alloc::vec::Vec as StdVec;

use vize_carton::{Span, String, cstr};
use vize_davinci::diagnostic::{Diagnostic, Severity, Stage};
use vize_davinci::id::NodeId;
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};
use vize_davinci::side_table::SideTable;
use vize_disegno::op::{IfOp, Op};
use vize_disegno::provenance::ProvenanceRecord;

use super::walk::{PageWalk, assert_accounting, visit_ops};
use crate::lower::{Lowered, WrapperKey, WrapperKeys};

mod keys;

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "v-if";

/// The duplicate-key message, byte-identical to relief's
/// `ErrorCode::VIfSameKey` text so the two channels never drift on
/// wording (pinned against relief in the `vize_atelier_core`
/// differential suite, which owns the relief edge).
pub const SAME_KEY_MESSAGE: &str = "v-if/v-else-if branches must use unique keys.";

/// The pass description — classification reasoning in the module docs.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    // Attribute-to-fact movement invalidates no analysis: spans, scopes
    // and provenance keys are untouched, and no analysis reads the
    // attribute surface it trims.
    Preserved::ALL,
);

/// One branch's extracted `key`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchKey {
    /// Which spelling carried the key.
    pub kind: BranchKeyKind,
    /// The authored attribute's range.
    pub span: Span,
}

/// The two key spellings a branch carrier can author.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchKeyKind {
    /// A static `key` attribute; `None` for a bare `key` — which,
    /// exactly as the legacy `extract_key_value_str`, never collides.
    Static(Option<String>),
    /// A `:key` binding: the trimmed authored value text, plus the index
    /// of the carrier binding that carries it (`None` for a
    /// wrapper-captured key, which has no op).
    Dynamic {
        /// The trimmed value text (the parser's same-name expansion
        /// applied — a valueless `:key` reads `key`).
        source: String,
        /// The carrier's binding index, for the surface exclusion.
        bind_index: Option<usize>,
    },
}

impl BranchKey {
    /// The text the collision check compares — kind-blind, exactly the
    /// legacy `extract_key_value_str` under the default dialect. `None`
    /// never collides: a bare `key`, or a `:key` whose value position
    /// holds no expression (the authored-blank spelling, which the
    /// shipped parser leaves without one).
    #[must_use]
    pub fn collision_text(&self) -> Option<&str> {
        match &self.kind {
            BranchKeyKind::Static(value) => value.as_deref(),
            BranchKeyKind::Dynamic { source, .. } if source.is_empty() => None,
            BranchKeyKind::Dynamic { source, .. } => Some(source.as_str()),
        }
    }
}

/// Per-`ui.if` branch-key facts, one slot per branch in authored order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IfFacts {
    /// `branches[i]` is branch `i`'s extracted key, when it had one.
    pub branches: StdVec<Option<BranchKey>>,
}

/// Facts cross compile boundaries with their artifact (P1-11; the same
/// enforcement `Diagnostic` and `ProvenanceRecord` carry).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<BranchKey>();
    assert_owned::<BranchKeyKind>();
    assert_owned::<IfFacts>();
};

/// 64-bit footprints, guarded like every node-size assert (the wasm32
/// lane is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<BranchKey>() == 48);
    assert!(core::mem::size_of::<IfFacts>() == 24);
};

/// Run the pass over `lowered`, in place.
///
/// Walks the op tree once in page order (re-deriving positional ids),
/// lifts each branch's static `key` attribute into a [`BranchKey`] fact,
/// and reports duplicate authored keys on the later branch's attribute
/// span. Diagnostics and provenance append to `lowered`'s own channels.
///
/// # Panics
///
/// Panics when the re-derived page-order count disagrees with the
/// lowering's minted accounting — a compiler bug by the id law, never an
/// input property.
#[must_use]
pub fn run(lowered: &mut Lowered<'_>) -> SideTable<IfFacts> {
    let Lowered {
        root,
        op_count,
        diagnostics,
        provenance,
        wrappers,
        ..
    } = lowered;
    let mut channels = Channels {
        diagnostics,
        provenance,
        wrappers,
        facts: SideTable::new(),
    };
    let mut walk = PageWalk::new();
    visit_ops(&mut walk, &mut root.ops, &mut |id, op| {
        if let Op::If(if_op) = op {
            process_if(&mut channels, id, if_op);
        }
    });
    assert_accounting(&walk, *op_count, NAME);
    channels.facts
}

struct Channels<'l> {
    diagnostics: &'l mut StdVec<Diagnostic>,
    provenance: &'l mut StdVec<ProvenanceRecord>,
    /// The lowering's captured `<template v-if>` wrapper keys, read-only.
    wrappers: &'l SideTable<WrapperKeys>,
    facts: SideTable<IfFacts>,
}

/// Lift each branch's key (wrapper-captured or carrier-extracted) into a
/// fact, then diagnose duplicate key texts (later branch flagged, per the
/// legacy transform; kind-blind text equality).
fn process_if<'a>(walk: &mut Channels<'_>, id: Option<NodeId>, if_op: &mut IfOp<'a>) {
    let wrapper = id.and_then(|id| walk.wrappers.get(id));
    let mut keys: StdVec<Option<BranchKey>> = StdVec::with_capacity(if_op.branches.len());
    for (index, branch) in if_op.branches.iter_mut().enumerate() {
        let captured = wrapper
            .and_then(|keys| keys.branches.get(index))
            .and_then(|key| key.as_ref())
            .map(|key| match key {
                WrapperKey::Static { value, span } => BranchKey {
                    kind: BranchKeyKind::Static(value.clone()),
                    span: *span,
                },
                WrapperKey::Dynamic { source, span } => BranchKey {
                    kind: BranchKeyKind::Dynamic {
                        source: source.clone(),
                        bind_index: None,
                    },
                    span: *span,
                },
            });
        let key = captured.or_else(|| keys::take_carrier_key(branch.span, &mut branch.region.ops));
        if let Some(key) = &key {
            walk.provenance.push(ProvenanceRecord {
                rule: String::from("pass.v-if.branch-key"),
                node: id,
                before: keys::key_spelling(key),
                after: cstr!("fact key branch={index}"),
                span: key.span,
            });
        }
        keys.push(key);
    }

    for index in 1..keys.len() {
        let Some(later) = &keys[index] else {
            continue;
        };
        let Some(text) = later.collision_text() else {
            continue;
        };
        let collides = keys[..index].iter().any(|earlier| {
            earlier
                .as_ref()
                .and_then(BranchKey::collision_text)
                .is_some_and(|existing| existing == text)
        });
        if collides {
            walk.diagnostics.push(Diagnostic::new(
                Severity::Error,
                Stage::Semantic,
                later.span,
                String::from(SAME_KEY_MESSAGE),
            ));
            walk.provenance.push(ProvenanceRecord {
                rule: String::from("error.v-if-same-key"),
                node: id,
                before: keys::key_spelling(later),
                after: String::default(),
                span: later.span,
            });
        }
    }

    if keys.iter().any(Option::is_some)
        && let Some(id) = id
    {
        walk.facts.insert(id, IfFacts { branches: keys });
    }
}
