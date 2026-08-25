//! The series-3 slot projection: the DOM-output-determining facts of
//! component slot grouping and slot outlets, projected from both lanes
//! into one shared shape and compared exactly.
//!
//! # The shared unit rule (applied to both trees)
//!
//! A **unit** is a node whose tag is not a native tag
//! (`vize_carton::is_native_tag`, computed from the authored tag on
//! both trees so the lanes' differing component classifiers never enter
//! the projection) that is **slot-active**: it carries a `v-slot`
//! itself or has a direct `<template v-slot>` child. Its groups follow
//! the legacy `collect_slots` + component-`v-slot` order: own
//! spellings, template groups in child order (duplicate static names
//! dropped silently), then the implicit default. Implicit-default
//! content uses the **shared predicate** — any non-slot-template child
//! that is not a comment and not whitespace-only text — on both trees;
//! where the legacy lane's raw predicate (comments and kept single
//! spaces count) would have synthesized a default this projection does
//! not, the class is **counted** (`units_filler_default`), never
//! compared. Conditional slot carriers (a `<template v-if v-slot>`
//! child, realized by legacy dynamic-slot codegen and dropped at S2
//! lowering under `drop.template-attribute`) and the JSX `v-slots`
//! spread are invisible to both projections by the same rule — counted
//! so the blind spot has a number. Components whose only group is the
//! implicit default of plain children are not slot-active and stay
//! uncompared (their grouping is definitional; the DOM bytes arrive
//! with P2-11).
//!
//! Slot **outlets** compare by name — the `renderSlot` name argument:
//! static value (a value-less `name` reads as `default`, both lanes) or
//! trimmed dynamic expression text. Forwarded outlet props stay
//! `defer.slot-props` (the `ui.bind` installment); fallback bytes are
//! P2-11's.

use vize_carton::String;
use vize_davinci::id::NodeId;
use vize_davinci::side_table::SideTable;
use vize_ricalco::pass::SlotFacts;
use vize_ricalco::pass::vslot::{SlotName, SlotParams};
use vize_s2::folio::{FolioBinding, FolioName, FolioOp};

/// The slot half of the comparator's accounting.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlotCounters {
    /// Slot-active units compared, groups matched pairwise.
    pub units: u64,
    /// Groups compared inside those units.
    pub groups: u64,
    /// Group params texts compared (authored on both sides).
    pub group_params: u64,
    /// Groups whose name both lanes invented (bare `v-slot` or the
    /// implicit default) — the cross-lane witness of the Synthesized
    /// producer.
    pub groups_invented: u64,
    /// Dynamic-name groups (trimmed texts compared).
    pub groups_dynamic: u64,
    /// Units holding a conditional/iterated slot carrier — modeled by
    /// neither projection (the recorded wrapper gap), counted so the
    /// blind spot has a number; the rest of the unit still compares.
    pub units_conditional: u64,
    /// Units also carrying the JSX `v-slots` spread (legacy codegen
    /// path; S2 carries it as `vue.directive`) — counted, unit still
    /// compared.
    pub units_forwarded: u64,
    /// Units where only the legacy raw predicate (comments / kept
    /// whitespace) would synthesize an implicit default — counted,
    /// never compared.
    pub units_filler_default: u64,
    /// Slot outlets compared by name.
    pub outlets: u64,
    /// Outlets whose compared name is a dynamic expression text.
    pub outlets_dynamic: u64,
}

/// A projected name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PName {
    /// The canonical static text (modifiers folded).
    Static(String),
    /// A dynamic expression's trimmed source text.
    Dynamic(String),
}

/// One projected group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PGroup {
    pub name: PName,
    /// Whether the lane invented the name (no authored argument /
    /// implicit default).
    pub invented: bool,
    /// The trimmed params text; `None` when unauthored or blank.
    pub params: Option<String>,
}

/// One projected slot-active unit.
#[derive(Debug, Clone, Default)]
pub struct PUnit {
    pub groups: Vec<PGroup>,
    /// Legacy-side counted classes (always `false` on the S2 side).
    pub conditional: bool,
    pub forwarded: bool,
    pub filler_default: bool,
}

/// One projected outlet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct POutlet {
    pub name: PName,
}

fn trimmed(text: &str) -> String {
    String::from(text.trim())
}

// ------------------------------------------------------------------- S2

pub(super) fn has_slot_content(bindings: &[FolioBinding]) -> bool {
    bindings
        .iter()
        .any(|binding| matches!(binding, FolioBinding::SlotContent(_)))
}

/// Whether a folio component is slot-active (the shared unit rule).
pub fn s2_slot_active(bindings: &[FolioBinding], children: &[FolioOp]) -> bool {
    has_slot_content(bindings)
        || children.iter().any(|child| {
            matches!(child, FolioOp::Element(el) if el.tag.as_str() == "template" && has_slot_content(&el.bindings))
        })
}

/// Project one S2 unit from the pass's published facts.
pub fn s2_unit(id: Option<NodeId>, facts: &SideTable<SlotFacts>) -> PUnit {
    let entry = id
        .and_then(|id| facts.get(id))
        .expect("a slot-active component has published slot facts");
    let groups = entry
        .groups
        .iter()
        .map(|group| PGroup {
            name: match &group.name {
                SlotName::Static { text, .. } => PName::Static(String::from(text.as_str())),
                SlotName::Dynamic { text } => PName::Dynamic(trimmed(text.as_str())),
            },
            invented: matches!(
                &group.name,
                SlotName::Static {
                    origin: vize_s2::scope::ScopeOrigin::Synthesized { .. },
                    ..
                }
            ),
            params: match &group.params {
                SlotParams::Scoped { text, .. } => Some(trimmed(text.as_str())),
                SlotParams::Absent => None,
            },
        })
        .collect();
    PUnit {
        groups,
        ..PUnit::default()
    }
}

/// Project one S2 outlet name.
pub fn s2_outlet(name: &FolioName) -> POutlet {
    POutlet {
        name: match name {
            FolioName::Static(text) => PName::Static(String::from(text.as_str())),
            FolioName::Dynamic(expr) => {
                let source = match expr {
                    vize_s2::folio::FolioExpr::Js { source, .. }
                    | vize_s2::folio::FolioExpr::Foreign { source, .. }
                    | vize_s2::folio::FolioExpr::Opaque { source, .. }
                    | vize_s2::folio::FolioExpr::Filter { source, .. } => source,
                };
                PName::Dynamic(trimmed(source.as_str()))
            }
        },
    }
}

// -------------------------------------------------------------- compare

fn diverged(
    name: &str,
    source: &str,
    old: &dyn core::fmt::Debug,
    s2: &dyn core::fmt::Debug,
    why: core::fmt::Arguments<'_>,
) -> ! {
    panic!(
        "TS-25 divergence [{name}]: {why}\ntemplate:\n{source}\nlegacy projection: {old:#?}\ns2 projection: {s2:#?}"
    )
}

/// Compare the two lanes' slot projections, counting every class.
pub fn check(
    name: &str,
    source: &str,
    old_units: &[PUnit],
    s2_units: &[PUnit],
    old_outlets: &[POutlet],
    s2_outlets: &[POutlet],
    counters: &mut SlotCounters,
) {
    if old_units.len() != s2_units.len() {
        diverged(
            name,
            source,
            &old_units,
            &s2_units,
            format_args!("unit count {} vs {}", old_units.len(), s2_units.len()),
        );
    }
    for (old, s2) in old_units.iter().zip(s2_units) {
        counters.units += 1;
        counters.units_conditional += u64::from(old.conditional);
        counters.units_forwarded += u64::from(old.forwarded);
        counters.units_filler_default += u64::from(old.filler_default);
        if old.groups.len() != s2.groups.len() {
            diverged(
                name,
                source,
                old,
                s2,
                format_args!("group count {} vs {}", old.groups.len(), s2.groups.len()),
            );
        }
        for (old_group, s2_group) in old.groups.iter().zip(&s2.groups) {
            if old_group != s2_group {
                diverged(
                    name,
                    source,
                    old,
                    s2,
                    format_args!("group {old_group:?} vs {s2_group:?}"),
                );
            }
            counters.groups += 1;
            counters.groups_invented += u64::from(old_group.invented);
            counters.groups_dynamic += u64::from(matches!(old_group.name, PName::Dynamic(_)));
            counters.group_params += u64::from(old_group.params.is_some());
        }
    }
    if old_outlets.len() != s2_outlets.len() {
        diverged(
            name,
            source,
            &old_outlets,
            &s2_outlets,
            format_args!("outlet count {} vs {}", old_outlets.len(), s2_outlets.len()),
        );
    }
    for (old, s2) in old_outlets.iter().zip(s2_outlets) {
        if old != s2 {
            diverged(
                name,
                source,
                old,
                s2,
                format_args!("outlet {old:?} vs {s2:?}"),
            );
        }
        counters.outlets += 1;
        counters.outlets_dynamic += u64::from(matches!(old.name, PName::Dynamic(_)));
    }
}
