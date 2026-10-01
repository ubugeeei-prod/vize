//! Shared backend decisions keyed by the owning L2 artifact's node ids.
//!
//! This is the native decision boundary for issue #6839. It is separate
//! from the flat [`crate::op::Program`]: DOM and SSR consume these facts
//! beside L2, while Vapor requests the program separately. The borrowed
//! conversion edge computes conservative static/binding/control facts;
//! placement analysis and production selection remain unfinished.

use alloc::vec::Vec;

use vize_l0::{id::NodeId, side_table::SideTable};

use crate::placement::Placement;

pub mod policy;

use policy::TargetPolicy;

/// Neutral static classification of one L2 subtree.
///
/// This vocabulary does not encode DOM patch flags or SSR/Vapor output.
/// The current DOM analysis remains the comparison oracle until the native
/// producer computes the same facts and passes byte and instruction gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticLevel {
    /// Runtime-dependent or conservatively ineligible surface/descendants.
    Dynamic,
    /// A static surface whose text children require runtime values.
    DynamicText,
    /// The whole subtree is independent of runtime values.
    Static,
}

/// Decisions owned by one L2 region op or attached binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDecision {
    /// Neutral static classification, before target encoding.
    pub static_level: StaticLevel,
    /// Static classification after this table's target policy filters
    /// attached bindings. L4 still owns the encoding of this fact.
    ///
    /// SSR omits native-element event handlers, so an otherwise static
    /// element can have dynamic neutral meaning and static output.
    pub output_level: StaticLevel,
    /// Dynamic attached binding ids, in authored binding order.
    ///
    /// These are L2 ids from the same artifact, never flat-program op ids.
    /// A node that owns no attached bindings has an empty list.
    pub dynamic_bindings: Vec<NodeId>,
    /// Chosen placement; L4 owns cache and hoist numbering.
    pub placement: Placement,
    /// Nearest containing control owner in [`DecisionTables::controls`].
    pub control: Option<NodeId>,
}

/// The authored construct that owns a control region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlKind {
    /// Conditional branches.
    Conditional,
    /// Iteration bodies.
    Loop,
    /// Slot outlet fallback. Grouped slot-content scopes are unfinished.
    Slot,
}

/// Control containment beside the L2 tree, keyed by its owning L2 node.
///
/// This record names conditional, loop and slot-outlet containment;
/// grouped slot-content scopes and branch analysis remain unfinished. It
/// does not invent a flat-program region id or reorder
/// any branch, slot, cache, or effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlRegion {
    /// The control construct's role.
    pub kind: ControlKind,
    /// Containing control owner, or none at the root.
    pub parent: Option<NodeId>,
}

/// L3 decisions consumed beside the L2 artifact that owns every key.
///
/// An empty table is scratch state, not evidence that analysis completed.
/// The native producer must account for all numbered L2 nodes and preserve
/// authored binding order before any production caller selects this path.
/// Storage is the existing sparse side table; no storage-cost reduction is
/// claimed by the native producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionTables {
    /// The policy under which these decisions were computed.
    pub policy: TargetPolicy,
    /// Per-node decisions, keyed by L2 region and binding node ids.
    pub nodes: SideTable<NodeDecision>,
    /// Control owners and their containment.
    pub controls: SideTable<ControlRegion>,
}
