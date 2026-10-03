//! Preserve the original selected template custody through the existing L3 walk.

use super::{
    DecisionBuildError, DecisionTables, NativeFileAnalysis, build_dom_file_decisions,
    dom::{DomFacts, DomFileExpression, DomFileForHead, DomFileHandler},
    policy::TargetPolicy,
};
use vize_l0::id::NodeId;
use vize_l2::{
    artifact::Artifact,
    file::FileArtifact,
    lang::js::{NativeTemplateFile, NativeTemplateView},
};

/// Decisions retain the moved, non-cloneable lower completion view.
///
/// The lower route authenticates selected Text/Comment/empty roots and
/// ordinary static HTML headers/bodies, with original HTML whitespace and
/// unsupported directive operands refused before completion. Structural completion does
/// not grant Vue tag roles: target holes retain the actual node and source.
/// Original For heads retain same-File custody, while runtime reads, loop
/// targets and whole-SFC migration remain separate gates.
///
/// A neutral analysis or an externally paired File cannot construct it:
/// ```compile_fail
/// use vize_l3::decision::{NativeFileAnalysis, native::NativeTemplateDomAnalysis};
/// fn forge<'f, 'a>(analysis: NativeFileAnalysis<'f, 'a>) {
///     let _ = NativeTemplateDomAnalysis { analysis };
/// }
/// ```
/// The actual lower owner cannot be discarded while its analysis is live:
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateFile;
/// use vize_l3::decision::native::build_native_dom_file_decisions;
/// fn discard(owner: NativeTemplateFile<'_>) {
///     let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
///     drop(owner);
///     let _ = analysis.file();
/// }
/// ```
/// The completion view cannot be extracted as a neutral analysis or cloned:
/// ```compile_fail
/// use vize_l3::decision::{NativeFileAnalysis, native::NativeTemplateDomAnalysis};
/// fn neutral<'f, 'a>(result: NativeTemplateDomAnalysis<'f, 'a>)
///     -> NativeFileAnalysis<'f, 'a> { result.analysis }
/// ```
/// ```compile_fail
/// use vize_l3::decision::native::NativeTemplateDomAnalysis;
/// fn clone(result: NativeTemplateDomAnalysis<'_, '_>) { let _ = result.clone(); }
/// ```
/// A complete neutral File is not a template completion capability:
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_l3::decision::native::build_native_dom_file_decisions;
/// fn promote(file: &FileArtifact<'_>) { let _ = build_native_dom_file_decisions(file); }
/// ```
pub struct NativeTemplateDomAnalysis<'owner, 'arena> {
    view: NativeTemplateView<'owner, 'arena>,
    analysis: NativeFileAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> NativeTemplateDomAnalysis<'owner, 'arena> {
    /// Original selected Component and File retained by the sole lower owner.
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateFile<'arena> {
        self.view.owner()
    }

    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
        self.analysis.file()
    }

    #[must_use]
    pub fn artifact(&self) -> &'owner Artifact<'arena> {
        self.analysis.artifact()
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
    pub fn expression(&self, node: NodeId) -> Option<&DomFileExpression<'owner, 'arena>> {
        self.dom()?.file_expression(node)
    }

    /// Whole original head joined during the sole DOM Enter walk.
    /// Collection access and loop emission remain typed refusals.
    #[must_use]
    pub fn for_head(&self, node: NodeId) -> Option<&DomFileForHead<'owner, 'arena>> {
        self.dom()?.file_for_head(node)
    }

    /// Whole handler custody observed at this original File's actual On event.
    /// Runtime inline/reference classification remains a separate prerequisite.
    #[must_use]
    pub fn handler(&self, node: NodeId) -> Option<&DomFileHandler<'owner, 'arena>> {
        self.dom()?.file_handler(node)
    }
}

/// Derive File internally from the moved completion view and traverse once.
///
/// There is no caller File, role, scope, binding list or completeness flag.
/// Missing File remains an explicit invariant refusal. The existing sole-file
/// producer supplies all decisions; this wrapper adds no AST/tree walk, stage
/// or runtime access classification.
///
/// Genuine L1 root text receipts permit selected-root condensation at the
/// original cursor. Direct-child and nested-body whitespace/entity refusals
/// remain unchanged. Omitted source events confer no node or target decision.
pub fn build_native_dom_file_decisions<'owner, 'arena>(
    view: NativeTemplateView<'owner, 'arena>,
) -> Result<NativeTemplateDomAnalysis<'owner, 'arena>, DecisionBuildError> {
    let file = view.file().ok_or(DecisionBuildError::IncompleteFile)?;
    let analysis = build_dom_file_decisions(file)?;
    Ok(NativeTemplateDomAnalysis { view, analysis })
}
