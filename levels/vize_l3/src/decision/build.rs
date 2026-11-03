//! Borrowed, native L2-node decisions beside a sealed canonical artifact.
//!
//! One enter/leave walk owns every id; placement stays inline until analyzed.

use alloc::vec::Vec;

mod error;
mod walk;
pub use error::DecisionBuildError;
mod levels;
use super::attribute_value::OriginalAttributeCursor;
use super::dom::vue::policy::{FileReads, NoReads};
use super::dom::{DomExpressionFacts, LiteralExpressions, build::DomBuilder};
use super::ssr::build::SsrBuilder;
use super::vapor::build::VaporBuilder;
use levels::Levels;

use super::{
    ControlRegion, NativeAnalysis, NodeDecision,
    policy::{BindingOwner, TargetPolicy},
};
use vize_l0::{id::NodeId, side_table::SideTable};
use vize_l2::{artifact::Artifact, file::FileArtifact, lang::js::NativeSelectedSetup};

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
    build_with_context(
        artifact,
        policy,
        expressions,
        file,
        reads,
        BuildContext::default(),
    )
}

pub(in crate::decision) fn build_with_original<'owner, 'arena>(
    file: &'owner FileArtifact<'arena>,
    policy: TargetPolicy,
    expressions: &impl DomExpressionFacts,
    reads: &impl FileReads<'owner, 'arena>,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    build_with_context(
        file.artifact(),
        policy,
        expressions,
        Some(file),
        reads,
        BuildContext {
            ssr_setup: None,
            vapor_setup: None,
            original_file: Some(file),
        },
    )
}

pub(in crate::decision) fn build_with_ssr_setup<'owner, 'arena>(
    setup: &NativeSelectedSetup<'owner, 'arena>,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    let file = setup.file();
    if !file.is_complete() {
        return Err(DecisionBuildError::IncompleteFile);
    }
    build_with_context(
        file.artifact(),
        TargetPolicy::Ssr,
        &LiteralExpressions,
        Some(file),
        &NoReads,
        BuildContext {
            ssr_setup: Some(setup),
            vapor_setup: None,
            original_file: Some(file),
        },
    )
}

pub(in crate::decision) fn build_with_vapor_setup<'owner, 'arena>(
    setup: &NativeSelectedSetup<'owner, 'arena>,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    let file = setup.file();
    build_with_context(
        file.artifact(),
        TargetPolicy::Vapor,
        &LiteralExpressions,
        Some(file),
        &NoReads,
        BuildContext {
            ssr_setup: None,
            vapor_setup: Some(setup),
            original_file: Some(file),
        },
    )
}

fn build_with_context<'owner, 'arena>(
    artifact: &'owner Artifact<'arena>,
    policy: TargetPolicy,
    expressions: &impl DomExpressionFacts,
    file: Option<&'owner FileArtifact<'arena>>,
    reads: &impl FileReads<'owner, 'arena>,
    context: BuildContext<'_, 'owner, 'arena>,
) -> Result<NativeAnalysis<'owner, 'arena>, DecisionBuildError> {
    if context
        .original_file
        .is_some_and(|file| !file.native_text_values().is_empty())
    {
        return Err(DecisionBuildError::PreparedTextProfile);
    }
    let mut builder = Builder {
        policy,
        original_attributes: context
            .original_file
            .map(|file| OriginalAttributeCursor::new(file, artifact))
            .transpose()?,
        node_count: artifact.node_count(),
        frames: Vec::new(),
        nodes: SideTable::new(),
        controls: SideTable::new(),
        ssr: (policy == TargetPolicy::Ssr).then(|| match context.ssr_setup {
            Some(setup) => SsrBuilder::new_setup(setup),
            None => SsrBuilder::new(),
        }),
        vapor: (policy == TargetPolicy::Vapor).then(|| VaporBuilder::new(context.vapor_setup)),
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
    let original_attributes = builder
        .original_attributes
        .take()
        .map(OriginalAttributeCursor::finish)
        .transpose()?;
    let dom = builder.dom.take().map(DomBuilder::finish);
    let ssr = builder.ssr.take().map(SsrBuilder::finish);
    let vapor = builder.vapor.take().map(VaporBuilder::finish);
    Ok(NativeAnalysis {
        artifact,
        tables: builder.finish()?,
        original_attributes,
        dom,
        ssr,
        vapor,
    })
}

#[derive(Default)]
struct BuildContext<'facts, 'owner, 'arena> {
    ssr_setup: Option<&'facts NativeSelectedSetup<'owner, 'arena>>,
    vapor_setup: Option<&'facts NativeSelectedSetup<'owner, 'arena>>,
    original_file: Option<&'owner FileArtifact<'arena>>,
}

struct Builder<'facts, 'owner, 'arena, F, R> {
    policy: TargetPolicy,
    original_attributes: Option<OriginalAttributeCursor<'owner, 'arena>>,
    node_count: u32,
    frames: Vec<Frame>,
    nodes: SideTable<NodeDecision>,
    controls: SideTable<ControlRegion>,
    dom: Option<DomBuilder<'facts, 'owner, 'arena, F, R>>,
    ssr: Option<SsrBuilder<'facts, 'owner, 'arena>>,
    vapor: Option<VaporBuilder<'facts, 'owner, 'arena>>,
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

#[cfg(test)]
mod tests;
