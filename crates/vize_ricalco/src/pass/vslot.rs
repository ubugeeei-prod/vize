//! The `v-slot` pass: the P2-9 port of
//! `crates/vize_atelier_core/src/transforms/v_slot.rs` (with its
//! `params.rs` and `validate.rs` halves), re-expressed over the
//! `ui.slot-content` binding surface the lowering now keeps.
//!
//! # What survived the port, and where the rest went
//!
//! The old step file splits three ways — thinner absorption than v-if
//! (¾) or v-for (whole), because most of the legacy slot machinery
//! never lived in the transform lane at all:
//!
//! - **Absorbed by lowering (P2-8 + this installment):** the outlet
//!   half of the file (`transform_slot_outlet`'s name + fallback — the
//!   `ui.slot` op, implicit name normalized to `default` at lowering)
//!   and the outlet's custom-directive/`v-model` diagnostics; the
//!   authored spelling of `v-slot` itself — name position, modifiers,
//!   params — now lowers to the attached [`SlotContentOp`] instead of
//!   the P2-8 `defer.v-slot`, and the slot-props scope facts (recorded
//!   since P2-8) key that binding op.
//! - **DOM realization (P2-11):** the entire `codegen/slots/` tree the
//!   step file feeds — `RenderSlot`/`CreateSlots` registration, dynamic
//!   slot objects, conditional carriers, the JSX `v-slots` spread —
//!   plus forwarded slot props on outlets (`defer.slot-props`, waiting
//!   for `ui.bind`).
//! - **The pass body (here):** what `collect_slots`, `get_slot_name` /
//!   `params.rs`, and `validate.rs` establish — the **canonical slot
//!   grouping** published as [`SlotFacts`] per `ui.component`, the
//!   default-name and implicit-default-slot **synthesis** (the series'
//!   first [`ScopeOrigin::Synthesized`] producer), the slot-props
//!   scope **consumption** (the vfor pattern extended to slot
//!   boundaries), and the four user diagnostics, wording pinned
//!   byte-identical to relief's.
//!
//! # The Synthesized producer
//!
//! Every name this pass invents is a recorded fact, never a naming
//! convention: a bare `v-slot`'s canonical name (`default`, rule
//! [`RULE_DEFAULT_NAME`]) and the implicit default group's name (rule
//! [`RULE_IMPLICIT_DEFAULT`]) carry
//! [`ScopeOrigin::Synthesized`]`{rule}`; an authored `v-slot:default`
//! spells the same text under `Authored` with its span. The two can
//! therefore never be confused downstream — pinned both directions by
//! the suite. The pass synthesizes no *binding* names: slot-prop names
//! are authored or pattern-pending (#4365), and the P2-8 hygiene law
//! (fresh [`ScopeTag`] per introduction site, identity `(name, tag)`)
//! is what the consumption validates across every slot boundary.
//!
//! # Classification (the review point)
//!
//! **`MandatoryLowering`, barrier** — see [`DESC`].
//!
//! - *Why mandatory:* the grouping is what slot compilation reads and
//!   the four diagnostics are user-visible at every tier (the legacy
//!   validation ran unconditionally); skipping the pass loses both.
//! - *Why `MandatoryLowering` and not `MandatoryDiagnostic`:* the pass
//!   establishes the invariant later stages assume — every component's
//!   canonical grouping, synthesized names as recorded facts — which is
//!   the lowering kind's literal definition ("the kind that
//!   canonicalizes"). It also diagnoses, which the diagnostic kind
//!   alone would cover; the choice follows what P2-11 consumes.
//!   Installment 2's recorded taxonomy tension applies here too, in a
//!   milder form: like v-for the pass **preserves the tree** (grouping
//!   is a published fact, not a mutation — a binding op cannot leave
//!   the surface without shifting every page-order id after it), so
//!   this is again a preserving mandatory pass, kept loud by the const
//!   pins.
//! - *Why barrier:* law 1 forces it, and slot gathering is literally
//!   the kind taxonomy's own example of what fusion breaks
//!   (`vize_davinci::pass::Fusability` — "sibling `v-else` collection,
//!   slot gathering"): a component's groups read across its whole
//!   child list, and tag freshness is a fact across the artifact.
//! - `Preserved::ALL`: nothing in the tree moves.
//!
//! # Facts and accounting
//!
//! The pass drives its own shaped recursion over the shared
//! [`super::walk::PageWalk`] (flat visitation cannot give a component
//! its children's carrier ids — the mint arithmetic still has one
//! home), count-asserted against the minted accounting on every run.
//! [`SlotFacts`] is keyed by the `ui.component`'s id, sparse: only
//! components with at least one group. Provenance: `pass.v-slot.scope`
//! per consumed params scope, `pass.v-slot.group` per grouped component
//! (static groups print quoted, dynamic in brackets, synthesized with a
//! leading `+`), and the two synthesis rules above; errors under
//! `error.v-slot-*`.

use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};
use vize_davinci::side_table::SideTable;
use vize_s0::String;
use vize_s2::scope::{ScopeOrigin, ScopeTag};

mod consume;
mod group;
mod spell;

use super::walk::{PageWalk, assert_accounting};
use crate::lower::Lowered;

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "v-slot";

/// The pass description — classification reasoning in the module docs.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    // The pass mutates nothing: grouping and synthesis are published
    // facts beside the tree.
    Preserved::ALL,
);

/// The four diagnostics, byte-identical to relief's `ErrorCode` texts
/// (pinned against relief in the `vize_atelier_core` differential
/// suite, which owns the relief edge — ricalco must not depend on the
/// legacy AST crate).
pub const MISPLACED_MESSAGE: &str = "v-slot can only be used on components or <template> tags.";
/// See [`MISPLACED_MESSAGE`].
pub const MIXED_MESSAGE: &str = "Mixed v-slot usage with named slots detected.";
/// See [`MISPLACED_MESSAGE`].
pub const DUPLICATE_MESSAGE: &str = "Duplicate slot names detected.";
/// See [`MISPLACED_MESSAGE`].
pub const EXTRANEOUS_MESSAGE: &str =
    "Extraneous children found when component already has an explicit default slot.";

/// The rule under which a bare `v-slot`'s canonical name is invented.
pub const RULE_DEFAULT_NAME: &str = "normalize.v-slot.default-name";
/// The rule under which the implicit default group is invented.
pub const RULE_IMPLICIT_DEFAULT: &str = "normalize.v-slot.implicit-default";

/// One group's canonical name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotName {
    /// A static canonical name (the dialect's modifier folding applied),
    /// with the hygiene origin: `Authored` at the spelling's span, or
    /// `Synthesized` under the rule that invented it.
    Static {
        /// The canonical text.
        text: String,
        /// Authored or synthesized — the recorded fact.
        origin: ScopeOrigin,
    },
    /// A dynamic name; the payload is the trimmed authored expression
    /// text (pessimal: consumers compare nothing on it).
    Dynamic {
        /// The trimmed authored source.
        text: String,
    },
}

impl SlotName {
    /// The canonical text, whichever kind carries it.
    #[must_use]
    pub fn text(&self) -> &str {
        match self {
            SlotName::Static { text, .. } | SlotName::Dynamic { text } => text.as_str(),
        }
    }
}

/// The consumed name view of an authored params position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotBound {
    /// The one enumerated simple-identifier name, validated against the
    /// lowering's `ScopeFacts` entry.
    Named(String),
    /// Authored but enumerating no name yet: a destructuring pattern
    /// (awaiting the #4365 enumeration seam), a foreign dialect, or the
    /// classified escape. Consumers pessimize.
    Pending,
}

/// One group's slot-props position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotParams {
    /// No params authored.
    Absent,
    /// Authored params under their introduction-site tag.
    Scoped {
        /// The trimmed authored source.
        text: String,
        /// The introduction-site tag the lowering minted (validated
        /// fresh).
        tag: ScopeTag,
        /// The consumed name view.
        name: SlotBound,
    },
}

/// Where a group's content lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotCarrier {
    /// The component's own `v-slot` — content is the component's
    /// children region.
    Component,
    /// A `<template v-slot>` child; the id is the template element's op
    /// (`None` only past id exhaustion, where no fact is keyed anyway).
    Template(Option<NodeId>),
    /// The synthesized implicit default — content is the non-template
    /// children.
    Implicit,
}

/// One canonical slot group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotGroup {
    /// The canonical name.
    pub name: SlotName,
    /// The slot-props position.
    pub params: SlotParams,
    /// Where the content lives.
    pub carrier: SlotCarrier,
}

/// One `ui.component`'s canonical slot grouping, groups in canonical
/// order: the component's own spellings, then template groups in child
/// order (duplicate static names dropped, exactly as the legacy
/// grouping), then the implicit default when synthesized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotFacts {
    /// The groups.
    pub groups: StdVec<SlotGroup>,
}

/// Facts cross compile boundaries with their artifact (P1-11; the same
/// enforcement `Diagnostic` and `ProvenanceRecord` carry).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<SlotName>();
    assert_owned::<SlotBound>();
    assert_owned::<SlotParams>();
    assert_owned::<SlotCarrier>();
    assert_owned::<SlotGroup>();
    assert_owned::<SlotFacts>();
};

/// 64-bit footprints, guarded like every node-size assert (the wasm32
/// lane is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<SlotName>() == 48);
    assert!(core::mem::size_of::<SlotBound>() == 24);
    assert!(core::mem::size_of::<SlotParams>() == 56);
    assert!(core::mem::size_of::<SlotCarrier>() == 8);
    assert!(core::mem::size_of::<SlotGroup>() == 112);
    assert!(core::mem::size_of::<SlotFacts>() == 24);
};

/// Run the pass over `lowered`.
///
/// One shaped page-order recursion: every `ui.slot-content`'s scope is
/// consumed where it stands, every `ui.component` is grouped from its
/// own spellings and its children's carriers, and the four validations
/// fire where the legacy lane fired them. The tree is not touched;
/// diagnostics and provenance append to `lowered`'s own channels.
///
/// # Panics
///
/// Panics on a broken accounting or hygiene law — a compiler bug, never
/// an input property: the re-derived count disagreeing with the minted
/// accounting, a params-bearing spelling without a scope entry (or a
/// paramless one with), a reused tag, or recorded bindings that
/// disagree with the params surface.
#[must_use]
pub fn run(lowered: &mut Lowered<'_>) -> SideTable<SlotFacts> {
    let Lowered {
        root,
        op_count,
        diagnostics,
        provenance,
        scopes,
        ..
    } = lowered;
    let mut channels = consume::Channels {
        scopes,
        diagnostics,
        provenance,
        seen_tags: StdVec::new(),
        facts: SideTable::new(),
    };
    let mut walk = PageWalk::new();
    consume::region(&mut walk, &mut channels, &root.ops);
    assert_accounting(&walk, *op_count, NAME);
    channels.facts
}
