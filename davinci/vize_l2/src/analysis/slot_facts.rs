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

/// Whitespace-only text (`&nbsp;` spellings included) that never counts as
/// implicit default-slot content.
#[must_use]
pub fn legacy_slot_filler_text(text: &str) -> bool {
    if text.trim().is_empty() {
        return true;
    }
    if !text.contains('&') {
        return false;
    }

    let mut tail = text;
    while let Some(ch) = tail.chars().next() {
        let consumed = if ch.is_whitespace() {
            ch.len_utf8()
        } else if let Some(consumed) = nbsp_entity_len(tail) {
            consumed
        } else {
            return false;
        };
        // Both lengths end on a char boundary (a whole char, or an ASCII
        // entity), so the tail always advances.
        tail = tail.get(consumed..).unwrap_or_default();
    }
    true
}

/// Filler text that still makes an empty slot render a props placeholder.
#[must_use]
pub fn legacy_slot_filler_needs_props_placeholder(text: &str) -> bool {
    legacy_slot_filler_text(text)
        && text
            .chars()
            .any(|ch| ch == '&' || !ch.is_ascii_whitespace())
}

fn nbsp_entity_len(text: &str) -> Option<usize> {
    if text.starts_with("&nbsp;") {
        return Some("&nbsp;".len());
    }
    if let Some(rest) = text.strip_prefix("&nbsp")
        && (rest.is_empty()
            || rest.starts_with('&')
            || rest.chars().next().is_some_and(char::is_whitespace))
    {
        return Some("&nbsp".len());
    }
    numeric_nbsp_entity_len(text)
}

fn numeric_nbsp_entity_len(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    if !bytes.starts_with(b"&#") {
        return None;
    }

    let mut index = 2usize;
    let radix = if matches!(bytes.get(index), Some(b'x' | b'X')) {
        index += 1;
        16
    } else {
        10
    };
    let start = index;
    while let Some(byte) = bytes.get(index) {
        let digit = if radix == 16 {
            byte.is_ascii_hexdigit()
        } else {
            byte.is_ascii_digit()
        };
        if !digit {
            break;
        }
        index += 1;
    }
    if index == start {
        return None;
    }
    let digits = core::str::from_utf8(bytes.get(start..index)?).ok()?;
    if u32::from_str_radix(digits, radix).ok()? != 0xa0 {
        return None;
    }
    if bytes.get(index) == Some(&b';') {
        index += 1;
    }
    Some(index)
}
