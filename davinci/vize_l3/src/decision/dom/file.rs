//! Sole-file analysis retains genuine scopes without assigning runtime access.

use super::{DomFacts, LiteralExpressions, vue::policy::NoReads};
use crate::decision::{DecisionBuildError, DecisionTables, NativeAnalysis, TargetPolicy, build};
use vize_l2::{
    artifact::Artifact,
    file::{FileArtifact, FileResolution, ScopeId},
};

/// The exact producer resolution admitted by the existing canonical node walk.
/// Construction is private; a caller cannot associate a table with a node.
pub struct DomFileExpression<'owner, 'arena> {
    pub(in crate::decision::dom) resolution: FileResolution<'owner, 'arena>,
}

impl core::fmt::Debug for DomFileExpression<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DomFileExpression")
            .field("node", &self.resolution.node())
            .field("scope", &self.resolution.scope())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> DomFileExpression<'owner, 'arena> {
    #[must_use]
    pub const fn resolution(&self) -> FileResolution<'owner, 'arena> {
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
/// use vize_l3::decision::{NativeAnalysis, NativeFileAnalysis};
/// fn discard_file<'f, 'a>(result: NativeFileAnalysis<'f, 'a>) -> NativeAnalysis<'f, 'a> {
///     result.analysis
/// }
/// ```
/// The live result prevents discarding its actual file owner:
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_l3::decision::build_dom_file_decisions;
/// fn discard(file: FileArtifact<'_>) {
///     let result = build_dom_file_decisions(&file).unwrap();
///     drop(file);
///     let _ = result.file();
/// }
/// ```
/// A bare artifact consumer cannot silently infer framework access:
/// ```compile_fail
/// use vize_l3::decision::{NativeAnalysis, NativeFileAnalysis};
/// fn bare(_: &NativeAnalysis<'_, '_>) {}
/// fn emit(result: &NativeFileAnalysis<'_, '_>) { bare(result); }
/// ```
pub struct NativeFileAnalysis<'owner, 'arena> {
    file: &'owner FileArtifact<'arena>,
    analysis: NativeAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> NativeFileAnalysis<'owner, 'arena> {
    #[must_use]
    pub const fn file(&self) -> &'owner FileArtifact<'arena> {
        self.file
    }

    #[must_use]
    pub fn artifact(&self) -> &'owner Artifact<'arena> {
        self.file.artifact()
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
}

/// Derive the canonical owner from the sole file and use the existing walk.
///
/// Missing or foreign factory rows are explicit DOM refusals. An incomplete
/// file is rejected before traversal. No caller-supplied scope, table, binding
/// list or free-name fallback participates in admission. If/For semantics are
/// still held until their actual file factories record complete owned facts.
pub fn build_dom_file_decisions<'owner, 'arena>(
    file: &'owner FileArtifact<'arena>,
) -> Result<NativeFileAnalysis<'owner, 'arena>, DecisionBuildError> {
    if !file.is_complete() {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build::build_with(
        file.artifact(),
        TargetPolicy::Dom,
        &LiteralExpressions,
        Some(file),
        &NoReads,
    )?;
    Ok(NativeFileAnalysis { file, analysis })
}

// Numeric query keys are meaningful only in this file. The entry above derives
// every queried NodeId from this owner's canonical events; equal foreign IDs
// are never accepted as independent owner authentication.
#[must_use]
pub(super) fn matches_expression(
    resolution: FileResolution<'_, '_>,
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
