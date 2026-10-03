//! Borrowed, native L2-node decisions beside a sealed canonical artifact.
//!
//! One enter/leave walk owns every id; placement stays inline until analyzed.

use alloc::vec::Vec;

mod error;
mod walk;
pub use error::DecisionBuildError;
mod levels;
use super::dom::vue::policy::{FileReads, NoReads};
use super::dom::{DomExpressionFacts, LiteralExpressions, build::DomBuilder};
use super::ssr::build::SsrBuilder;
use super::vapor::build::VaporBuilder;
use levels::Levels;

use super::{
    ControlKind, ControlRegion, DecisionTables, NativeAnalysis, NodeDecision, StaticLevel,
    policy::{BindingOwner, BindingRole, TargetPolicy},
};
use crate::placement::Placement;
use vize_l0::{Span, id::NodeId, side_table::SideTable};
use vize_l2::{
    artifact::Artifact,
    file::FileArtifact,
    op::{BindingOp, Op},
    walk::{NodeEvent, NodeRef},
};

/// Compute conservative shared facts for every canonical L2 node.
///
/// Neutral meaning never changes with the selected target. Output meaning
/// filters compile-time cloak markers and SSR native-element events only.
/// Authored comments, components and control ops stay dynamic. Slot-content
/// grouping, hoist/cache eligibility and production selection are unfinished.
pub fn build_decisions<'owner, 'arena>(
    artifact: &'owner Artifact<'arena>,
    policy: TargetPolicy,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    build_with(artifact, policy, &LiteralExpressions, None, &NoReads)
}

/// Apply explicit native expression-binding semantics in the same DOM walk.
pub fn build_dom_decisions<'owner, 'arena>(
    artifact: &'owner Artifact<'arena>,
    expressions: &impl DomExpressionFacts,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    build_with(artifact, TargetPolicy::Dom, expressions, None, &NoReads)
}

pub(in crate::decision) fn build_with<'owner, 'arena>(
    artifact: &'owner Artifact<'arena>,
    policy: TargetPolicy,
    expressions: &impl DomExpressionFacts,
    file: Option<&'owner FileArtifact<'arena>>,
    reads: &impl FileReads<'owner, 'arena>,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    let mut builder = Builder {
        policy,
        node_count: artifact.node_count(),
        frames: Vec::new(),
        nodes: SideTable::new(),
        controls: SideTable::new(),
        ssr: (policy == TargetPolicy::Ssr).then(SsrBuilder::new),
        vapor: (policy == TargetPolicy::Vapor).then(VaporBuilder::new),
        dom: (policy == TargetPolicy::Dom)
            .then(|| DomBuilder::new(expressions, artifact.root().ops.len(), file, reads)),
    };
    let mut failure = None;
    artifact
        .visit_events(&mut |event| {
            if failure.is_none() {
                failure = builder.visit(event).err();
            }
        })
        .map_err(|_| DecisionBuildError::NodeLimit)?;
    if let Some(error) = failure {
        return Err(error);
    }
    let dom = builder.dom.take().map(DomBuilder::finish);
    let ssr = builder.ssr.take().map(SsrBuilder::finish);
    let vapor = builder.vapor.take().map(VaporBuilder::finish);
    Ok(NativeAnalysis {
        artifact,
        tables: builder.finish()?,
        dom,
        ssr,
        vapor,
    })
}

struct Builder<'facts, 'owner, 'arena, F, R> {
    policy: TargetPolicy,
    node_count: u32,
    frames: Vec<Frame>,
    nodes: SideTable<NodeDecision>,
    controls: SideTable<ControlRegion>,
    dom: Option<DomBuilder<'facts, 'owner, 'arena, F, R>>,
    ssr: Option<SsrBuilder<'owner, 'arena>>,
    vapor: Option<VaporBuilder<'owner, 'arena>>,
}

/// One open region op, released at its matching leave event.
struct Frame {
    id: NodeId,
    levels: Levels,
    owner: Option<BindingOwner>,
    control: Option<NodeId>,
    child_control: Option<NodeId>,
    dynamic_bindings: Vec<NodeId>,
}

