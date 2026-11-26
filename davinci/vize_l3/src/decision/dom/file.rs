//! Sole-file analysis retains genuine scopes without assigning runtime access.

use super::{DomAnalysis, LiteralExpressions, vue::policy::NoReads};
use crate::decision::{DecisionAnalysis, DecisionBuildError, DecisionTables, TargetPolicy, build};
use vize_l2::{
    artifact::L2Template,
    file::{L2Ir, L2IrResolution, ScopeId},
};

/// The exact producer resolution admitted by the existing canonical node walk.
/// Construction is private; a caller cannot associate a table with a node.
pub struct DomL2IrExpression<'owner, 'arena> {
    pub(in crate::decision::dom) resolution: L2IrResolution<'owner, 'arena>,
}

impl core::fmt::Debug for DomL2IrExpression<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DomL2IrExpression")
            .field("node", &self.resolution.node())
            .field("scope", &self.resolution.scope())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> DomL2IrExpression<'owner, 'arena> {
    #[must_use]
    pub const fn resolution(&self) -> L2IrResolution<'owner, 'arena> {
        self.resolution
    }

    #[must_use]
    pub fn scope(&self) -> Option<ScopeId> {
        self.resolution.scope()
    }
}

/// An immutable result tied to its sole genuine file owner.
///
/// This deliberately cannot be passed to the bare-artifact `ContextOnly`
/// emitter. Script identity and lexical visibility do not establish Vue
/// runtime exposure. A target must consume an actual same-file access policy.
///
/// ```compile_fail
/// use vize_l3::decision::{DecisionAnalysis, DomL2IrAnalysis};
/// fn discard_file<'f, 'a>(result: DomL2IrAnalysis<'f, 'a>) -> DecisionAnalysis<'f, 'a> {
///     result.analysis
/// }
/// ```
/// The live result prevents discarding its actual file owner:
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_l3::decision::build_dom_file_decisions;
/// fn discard(l2ir: L2Ir<'_>) {
///     let result = build_dom_file_decisions(&l2ir).unwrap();
///     drop(l2ir);
///     let _ = result.l2ir();
/// }
/// ```
/// A bare artifact consumer cannot silently infer framework access:
/// ```compile_fail
/// use vize_l3::decision::{DecisionAnalysis, DomL2IrAnalysis};
/// fn bare(_: &DecisionAnalysis<'_, '_>) {}
/// fn emit(result: &DomL2IrAnalysis<'_, '_>) { bare(result); }
/// ```
pub struct DomL2IrAnalysis<'owner, 'arena> {
    l2ir: &'owner L2Ir<'arena>,
    analysis: DecisionAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> DomL2IrAnalysis<'owner, 'arena> {
    #[must_use]
    pub const fn l2ir(&self) -> &'owner L2Ir<'arena> {
        self.l2ir
    }

    #[must_use]
    pub fn original_attributes(
        &self,
    ) -> Option<&crate::decision::OriginalAttributeAnalysis<'owner, 'arena>> {
        self.analysis.original_attributes()
    }

    #[must_use]
    pub fn artifact(&self) -> &'owner L2Template<'arena> {
        self.l2ir.artifact()
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
    pub fn dom(&self) -> Option<&DomAnalysis<'owner, 'arena>> {
        self.analysis.dom()
    }
}

/// Derive the canonical owner from the sole file and use the existing walk.
///
/// Missing or foreign factory rows are explicit DOM refusals. An incomplete
/// file is rejected before traversal. No caller-supplied scope, table, binding
/// list or free-name fallback participates in admission. If/For semantics are
/// still held until their actual file factories record complete owned facts.
pub fn build_dom_file_decisions<'owner, 'arena>(
    l2ir: &'owner L2Ir<'arena>,
) -> Result<DomL2IrAnalysis<'owner, 'arena>, DecisionBuildError> {
    if !l2ir.is_complete() {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build::build_with(
        l2ir.artifact(),
        TargetPolicy::Dom,
        &LiteralExpressions,
        Some(l2ir),
        &NoReads,
    )?;
    Ok(DomL2IrAnalysis { l2ir, analysis })
}

// Numeric query keys are meaningful only in this file. The entry above derives
// every queried NodeId from this owner's canonical events; equal foreign IDs
// are never accepted as independent owner authentication.
#[must_use]
pub(super) fn matches_expression(
    resolution: L2IrResolution<'_, '_>,
    expression: &vize_l2::expr::JsExpr<'_>,
) -> bool {
    resolution.table().is_some_and(|table| {
        let retained = table.expression();
        core::ptr::eq(retained.ast, expression.ast)
            && core::ptr::eq(retained.source, expression.source)
            && retained.span == expression.span
            && match (retained.coordinates, expression.coordinates) {
                (None, None) => true,
                (Some(left), Some(right)) => core::ptr::eq(left, right),
                _ => false,
            }
    })
}

// Called only after the real selected template entry derives its own File.
pub(in crate::decision) fn build_original_file_decisions<'owner, 'arena>(
    l2ir: &'owner L2Ir<'arena>,
) -> Result<DomL2IrAnalysis<'owner, 'arena>, DecisionBuildError> {
    let analysis = crate::decision::build::build_with_original(
        l2ir,
        TargetPolicy::Dom,
        &crate::decision::dom::LiteralExpressions,
        &crate::decision::dom::vue::policy::NoReads,
    )?;
    Ok(DomL2IrAnalysis { l2ir, analysis })
}

#[cfg(feature = "client-server-decisions")]
impl<'owner, 'arena> DomL2IrAnalysis<'owner, 'arena> {
    pub(in crate::decision) fn from_pair(
        target: crate::decision::build::paired::CompletedTarget<'owner, 'arena>,
    ) -> (vize_l2::lang::js::NativeTemplateView<'owner, 'arena>, Self) {
        let (view, file, analysis) = target.into_parts();
        (
            view,
            Self {
                l2ir: file,
                analysis,
            },
        )
    }
}
