//! Canonical slot-group facts consumed from L2 component owners.
//!
//! Dialect legalization chooses the groups, their names and scope origins.
//! This owned contract records those decisions without parsing a dialect,
//! walking a lowering result or selecting a backend. Existing Vue validation
//! and synthesis remain in the L1-to-L2 legalizer.

use crate::scope::{ScopeOrigin, ScopeTag};
use alloc::vec::Vec as StdVec;
use vize_l0::String;
use vize_l0::id::NodeId;

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
