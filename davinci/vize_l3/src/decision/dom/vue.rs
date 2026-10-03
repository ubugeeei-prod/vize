//! Genuine setup membership and reads, without claiming template custody.

use super::{DomFacts, LiteralExpressions, ValueKind};
use crate::decision::{DecisionBuildError, DecisionTables, NativeAnalysis, TargetPolicy, build};
use alloc::vec::Vec;
use vize_l0::id::NodeId;
use vize_l2::{
    artifact::Artifact,
    file::{BindingRef, FileArtifact, FileResolution, vue::VueExposure},
    resolution::Occurrence,
};

pub(in crate::decision) mod policy;

/// Vue's external-function read of an authenticated setup declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueReadKind {
    /// An immutable binding with an actual primitive-literal initializer.
    SetupConst,
    /// Mutable let/var bindings retain dynamic value semantics.
    SetupLet,
}

/// One read retains the exact original resolver occurrence, including duplicates.
pub struct VueRenderRead<'owner, 'arena> {
    pub(in crate::decision::dom) occurrence: &'owner Occurrence<'arena>,
    pub(in crate::decision::dom) binding: BindingRef<'owner, 'arena>,
    pub(in crate::decision::dom) kind: VueReadKind,
}

impl<'owner, 'arena> VueRenderRead<'owner, 'arena> {
    #[must_use]
    pub fn occurrence(&self) -> &'owner Occurrence<'arena> {
        self.occurrence
    }

    #[must_use]
    pub fn binding(&self) -> BindingRef<'owner, 'arena> {
        self.binding
    }

    #[must_use]
    pub fn kind(&self) -> VueReadKind {
        self.kind
    }
}

/// Construction publishes the complete ordered read list or no row.
pub struct VueRenderExpression<'owner, 'arena> {
    pub(in crate::decision::dom) resolution: FileResolution<'owner, 'arena>,
    pub(in crate::decision::dom) reads: Vec<VueRenderRead<'owner, 'arena>>,
    pub(in crate::decision::dom) value: ValueKind,
}

impl core::fmt::Debug for VueRenderExpression<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("VueRenderExpression")
            .field("node", &self.resolution.node())
            .field("read_count", &self.reads.len())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> VueRenderExpression<'owner, 'arena> {
    #[must_use]
    pub fn resolution(&self) -> FileResolution<'owner, 'arena> {
        self.resolution
    }

    #[must_use]
    pub fn reads(&self) -> &[VueRenderRead<'owner, 'arena>] {
        &self.reads
    }

    #[must_use]
    pub fn value(&self) -> ValueKind {
        self.value
    }
}

/// Sole authenticated declaration owner and its immutable decisions.
///
/// This proves read classification only. Native template custody, complete
/// source spelling, script emission, If/For and product routing remain separate.
/// There is no conversion into a bare artifact or neutral file analysis.
///
/// ```compile_fail
/// use vize_l3::decision::dom::vue::NativeVueRenderAnalysis;
/// fn clone(view: NativeVueRenderAnalysis<'_, '_, '_, '_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use vize_l3::decision::dom::vue::VueRenderRead;
/// fn forge() { let _ = VueRenderRead { kind: Default::default() }; }
/// ```
/// A live result keeps the authentic membership owner borrowed:
/// ```compile_fail
/// use vize_l2::file::vue::VueExposure;
/// use vize_l3::decision::dom::vue::build_vue_render_decisions;
/// fn discard(exposure: VueExposure<'_, '_, '_, '_>) {
///     let result = build_vue_render_decisions(&exposure).unwrap();
///     drop(exposure);
///     let _ = result.file();
/// }
/// ```
/// The checked access rows have no mutable or neutral conversion:
/// ```compile_fail
/// use vize_l3::decision::{NativeFileAnalysis, dom::vue::NativeVueRenderAnalysis};
/// fn discard<'v,'f,'d,'p,'a>(result: NativeVueRenderAnalysis<'v,'f,'d,'p,'a>)
///     -> NativeFileAnalysis<'f,'a> { result.analysis }
/// ```
pub struct NativeVueRenderAnalysis<'view, 'owner, 'descriptor, 'program, 'arena> {
    exposure: &'view VueExposure<'owner, 'descriptor, 'program, 'arena>,
    analysis: NativeAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> NativeVueRenderAnalysis<'_, 'owner, '_, '_, 'arena> {
    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
        self.exposure.file()
    }

    #[must_use]
    pub fn artifact(&self) -> &'owner Artifact<'arena> {
        self.file().artifact()
    }

    #[must_use]
    pub fn policy(&self) -> TargetPolicy {
        self.analysis.policy()
    }

    #[must_use]
    pub fn tables(&self) -> &DecisionTables {
        self.analysis.tables()
    }

    #[must_use]
    pub fn dom(&self) -> Option<&DomFacts<'owner, 'arena>> {
        self.analysis.dom()
    }

    #[must_use]
    pub fn expression(&self, node: NodeId) -> Option<&VueRenderExpression<'owner, 'arena>> {
        self.analysis.dom()?.vue_expressions.get(node)
    }
}

/// Classify actual setup reads during the sole existing canonical event walk.
/// A raw File, role, binding list or caller access policy cannot select this path.
pub fn build_vue_render_decisions<'view, 'owner, 'descriptor, 'program, 'arena>(
    exposure: &'view VueExposure<'owner, 'descriptor, 'program, 'arena>,
) -> Result<NativeVueRenderAnalysis<'view, 'owner, 'descriptor, 'program, 'arena>, DecisionBuildError>
{
    let file = exposure.file();
    if !file.is_complete() {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build::build_with(
        file.artifact(),
        TargetPolicy::Dom,
        &LiteralExpressions,
        Some(file),
        &policy::ExposureReads(exposure),
    )?;
    Ok(NativeVueRenderAnalysis { exposure, analysis })
}
