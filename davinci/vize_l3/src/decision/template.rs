//! Preserve the original selected template custody through the existing L3 walk.

mod setup;
pub use setup::{SelectedSetupDomAnalysis, build_selected_setup_dom_decisions};

use super::{
    DecisionBuildError, DecisionTables, DomL2IrAnalysis,
    dom::{DomAnalysis, DomL2IrExpression, DomL2IrForHead, DomL2IrHandler},
    policy::TargetPolicy,
};
use vize_l0::id::NodeId;
use vize_l2::{
    artifact::L2Template,
    file::L2Ir,
    lang::js::{NativeTemplateL2Ir, NativeTemplateView},
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
/// use vize_l3::decision::{DomL2IrAnalysis, template::TemplateDomAnalysis};
/// fn forge<'f, 'a>(analysis: DomL2IrAnalysis<'f, 'a>) {
///     let _ = TemplateDomAnalysis { analysis };
/// }
/// ```
/// The actual lower owner cannot be discarded while its analysis is live:
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateL2Ir;
/// use vize_l3::decision::template::build_template_dom_decisions;
/// fn discard(owner: NativeTemplateL2Ir<'_>) {
///     let analysis = build_template_dom_decisions(owner.view().unwrap()).unwrap();
///     drop(owner);
///     let _ = analysis.l2ir();
/// }
/// ```
/// The completion view cannot be extracted as a neutral analysis or cloned:
/// ```compile_fail
/// use vize_l3::decision::{DomL2IrAnalysis, template::TemplateDomAnalysis};
/// fn neutral<'f, 'a>(result: TemplateDomAnalysis<'f, 'a>)
///     -> DomL2IrAnalysis<'f, 'a> { result.analysis }
/// ```
/// ```compile_fail
/// use vize_l3::decision::template::TemplateDomAnalysis;
/// fn clone(result: TemplateDomAnalysis<'_, '_>) { let _ = result.clone(); }
/// ```
/// A complete neutral File is not a template completion capability:
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_l3::decision::template::build_template_dom_decisions;
/// fn promote(l2ir: &L2Ir<'_>) { let _ = build_template_dom_decisions(l2ir); }
/// ```
pub struct TemplateDomAnalysis<'owner, 'arena> {
    view: NativeTemplateView<'owner, 'arena>,
    analysis: DomL2IrAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> TemplateDomAnalysis<'owner, 'arena> {
    /// Original selected Component and File retained by the sole lower owner.
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateL2Ir<'arena> {
        self.view.owner()
    }

    #[must_use]
    pub fn l2ir(&self) -> &'owner L2Ir<'arena> {
        self.analysis.l2ir()
    }

    #[must_use]
    pub fn original_attributes(&self) -> Option<&super::OriginalAttributeAnalysis<'owner, 'arena>> {
        self.analysis.original_attributes()
    }

    #[must_use]
    pub fn artifact(&self) -> &'owner L2Template<'arena> {
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
    pub fn dom(&self) -> Option<&DomAnalysis<'owner, 'arena>> {
        self.analysis.dom()
    }

    #[must_use]
    pub fn expression(&self, node: NodeId) -> Option<&DomL2IrExpression<'owner, 'arena>> {
        self.dom()?.file_expression(node)
    }

    /// Whole original head joined during the sole DOM Enter walk.
    /// Bounded setup reads retain identity; collection runtime spelling and
    /// loop emission remain typed refusals.
    #[must_use]
    pub fn for_head(&self, node: NodeId) -> Option<&DomL2IrForHead<'owner, 'arena>> {
        self.dom()?.file_for_head(node)
    }

    /// Whole handler custody observed at this original File's actual On event.
    /// Runtime inline/reference classification remains a separate prerequisite.
    #[must_use]
    pub fn handler(&self, node: NodeId) -> Option<&DomL2IrHandler<'owner, 'arena>> {
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
pub fn build_template_dom_decisions<'owner, 'arena>(
    view: NativeTemplateView<'owner, 'arena>,
) -> Result<TemplateDomAnalysis<'owner, 'arena>, DecisionBuildError> {
    let file = view.file().ok_or(DecisionBuildError::IncompleteFile)?;
    let analysis = super::dom::file::build_original_file_decisions(file)?;
    Ok(TemplateDomAnalysis { view, analysis })
}

#[cfg(feature = "client-server-decisions")]
impl<'owner, 'arena> TemplateDomAnalysis<'owner, 'arena> {
    pub(in crate::decision) fn from_pair(
        target: crate::decision::build::paired::CompletedTarget<'owner, 'arena>,
    ) -> Self {
        let (view, analysis) = DomL2IrAnalysis::from_pair(target);
        Self { view, analysis }
    }
}
