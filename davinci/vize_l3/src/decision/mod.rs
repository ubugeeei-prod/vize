//! Shared backend decisions keyed by the owning L2 artifact's node ids.
//!
//! This is the native decision boundary for issue #6839. It is separate
//! from the flat [`crate::op::Program`]: DOM and SSR consume these facts
//! beside L2, while Vapor requests the program separately. The borrowed
//! producer computes conservative static/binding/control facts;
//! placement analysis and production selection remain unfinished.

use alloc::vec::Vec;

use vize_l0::{id::NodeId, side_table::SideTable};
use vize_l2::artifact::Artifact;

use crate::placement::Placement;

mod build;
pub mod policy;

pub use build::{DecisionBuildError, build_decisions};

use policy::TargetPolicy;

/// Complete native decisions bound to their actual immutable L2 owner.
///
/// Only [`build_decisions`] can construct this result. A consumer takes this
/// sole input and derives the artifact and read-only tables from it; equal
/// local node indices in another artifact cannot establish that association.
/// Keeping the borrow also prevents unsealing the owner while this result lives.
///
/// Arbitrary complete tables cannot claim an owner's identity:
/// ```compile_fail
/// use vize_l2::artifact::Artifact;
/// use vize_l3::decision::{DecisionTables, NativeAnalysis};
/// fn forge<'o, 'a>(artifact: &'o Artifact<'a>, tables: DecisionTables) {
///     let _ = NativeAnalysis { artifact, tables };
/// }
/// ```
/// Its tables cannot be modified through the analysis:
/// ```compile_fail
/// use vize_l3::decision::NativeAnalysis;
/// fn edit(analysis: &mut NativeAnalysis<'_, '_>) {
///     analysis.tables().nodes.clear();
/// }
/// ```
/// Unsealing the owner cannot invalidate live decisions:
/// ```compile_fail
/// use vize_l2::artifact::Artifact;
/// use vize_l3::decision::{build_decisions, policy::TargetPolicy};
/// fn unseal(artifact: Artifact<'_>) {
///     let analysis = build_decisions(&artifact, TargetPolicy::Ssr).unwrap();
///     let _parts = artifact.into_parts();
///     let _ = analysis.tables();
/// }
/// ```
pub struct NativeAnalysis<'owner, 'arena> {
    artifact: &'owner Artifact<'arena>,
    tables: DecisionTables,
}

impl<'owner, 'arena> NativeAnalysis<'owner, 'arena> {
    /// The exact sealed artifact used by the native producer.
    #[must_use]
    pub fn artifact(&self) -> &'owner Artifact<'arena> {
        self.artifact
    }

    /// The target policy selected for these decisions.
    #[must_use]
    pub fn policy(&self) -> TargetPolicy {
        self.tables.policy
    }

    /// Complete decisions, borrowed read-only with their owner retained.
    #[must_use]
    pub fn tables(&self) -> &DecisionTables {
        &self.tables
    }
}

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

/// Mutable scratch records for native L3 decisions.
///
/// Completed consumer input is [`NativeAnalysis`], which retains the actual
/// owner and exposes these tables read-only. An empty or independently built
/// table cannot establish complete analysis or association with any artifact.
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
