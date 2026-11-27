//! Genuine original template completion retains its lower owner through L3.

use super::{VaporAnalysis, VaporL2IrAnalysis};
use crate::decision::{DecisionBuildError, DecisionTables};
use vize_l2::{
    ir::L2Ir,
    lang::js::{NativeTemplateL2Ir, NativeTemplateView},
    template::L2Template,
};

/// Only a moved original template completion can authorize native Vapor output.
///
/// Complete neutral File analysis cannot construct this capability:
/// ```compile_fail
/// use vize_l3::decision::vapor::{VaporL2IrAnalysis, TemplateVaporAnalysis};
/// fn promote(analysis: VaporL2IrAnalysis<'_, '_>) {
///     let _ = TemplateVaporAnalysis { analysis };
/// }
/// ```
/// Its original lower owner remains live until the analysis is released:
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateL2Ir;
/// use vize_l3::decision::vapor::build_template_vapor_decisions;
/// fn discard(owner: NativeTemplateL2Ir<'_>) {
///     let analysis = build_template_vapor_decisions(owner.view().unwrap()).unwrap();
///     drop(owner);
///     let _ = analysis.owner();
/// }
/// ```
/// A complete manually populated File is not original template completion:
/// ```compile_fail
/// use vize_l2::ir::L2Ir;
/// use vize_l3::decision::vapor::build_template_vapor_decisions;
/// fn forge(l2ir: &L2Ir<'_>) { let _ = build_template_vapor_decisions(l2ir); }
/// ```
pub struct TemplateVaporAnalysis<'owner, 'arena> {
    view: NativeTemplateView<'owner, 'arena>,
    analysis: VaporL2IrAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> TemplateVaporAnalysis<'owner, 'arena> {
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateL2Ir<'arena> {
        self.view.owner()
    }
    #[must_use]
    pub fn l2ir(&self) -> &'owner L2Ir<'arena> {
        self.analysis.l2ir()
    }
    #[must_use]
    pub fn original_attributes(
        &self,
    ) -> Option<&crate::decision::OriginalAttributeAnalysis<'owner, 'arena>> {
        self.analysis.original_attributes()
    }
    #[must_use]
    pub fn artifact(&self) -> &'owner L2Template<'arena> {
        self.analysis.artifact()
    }
    #[must_use]
    pub fn tables(&self) -> &DecisionTables {
        self.analysis.tables()
    }
    #[must_use]
    pub fn vapor(&self) -> Option<&VaporAnalysis<'owner, 'arena>> {
        self.analysis.vapor()
    }
}

/// Derive the File internally from the genuine non-cloneable lower completion.
/// It runs the existing producer once; no second AST/tree walk is introduced.
pub fn build_template_vapor_decisions<'owner, 'arena>(
    view: NativeTemplateView<'owner, 'arena>,
) -> Result<TemplateVaporAnalysis<'owner, 'arena>, DecisionBuildError> {
    let file = view.file().ok_or(DecisionBuildError::IncompleteFile)?;
    let analysis = super::l2ir::build_original_file_decisions(file)?;
    Ok(TemplateVaporAnalysis { view, analysis })
}
