//! Genuine original template completion retains its lower owner through L3.

use super::{VaporFacts, VaporFileAnalysis};
use crate::decision::{DecisionBuildError, DecisionTables};
use vize_l2::{
    artifact::Artifact,
    file::L2Ir,
    lang::js::{NativeTemplateFile, NativeTemplateView},
};

/// Only a moved original template completion can authorize native Vapor output.
///
/// Complete neutral File analysis cannot construct this capability:
/// ```compile_fail
/// use vize_l3::decision::vapor::{VaporFileAnalysis, TemplateVaporAnalysis};
/// fn promote(analysis: VaporFileAnalysis<'_, '_>) {
///     let _ = TemplateVaporAnalysis { analysis };
/// }
/// ```
/// Its original lower owner remains live until the analysis is released:
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateFile;
/// use vize_l3::decision::vapor::build_template_vapor_decisions;
/// fn discard(owner: NativeTemplateFile<'_>) {
///     let analysis = build_template_vapor_decisions(owner.view().unwrap()).unwrap();
///     drop(owner);
///     let _ = analysis.owner();
/// }
/// ```
/// A complete manually populated File is not original template completion:
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_l3::decision::vapor::build_template_vapor_decisions;
/// fn forge(file: &L2Ir<'_>) { let _ = build_template_vapor_decisions(file); }
/// ```
pub struct TemplateVaporAnalysis<'owner, 'arena> {
    view: NativeTemplateView<'owner, 'arena>,
    analysis: VaporFileAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> TemplateVaporAnalysis<'owner, 'arena> {
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateFile<'arena> {
        self.view.owner()
    }
    #[must_use]
    pub fn file(&self) -> &'owner L2Ir<'arena> {
        self.analysis.file()
    }
    #[must_use]
    pub fn original_attributes(
        &self,
    ) -> Option<&crate::decision::OriginalAttributeFacts<'owner, 'arena>> {
        self.analysis.original_attributes()
    }
    #[must_use]
    pub fn artifact(&self) -> &'owner Artifact<'arena> {
        self.analysis.artifact()
    }
    #[must_use]
    pub fn tables(&self) -> &DecisionTables {
        self.analysis.tables()
    }
    #[must_use]
    pub fn vapor(&self) -> Option<&VaporFacts<'owner, 'arena>> {
        self.analysis.vapor()
    }
}

/// Derive the File internally from the genuine non-cloneable lower completion.
/// It runs the existing producer once; no second AST/tree walk is introduced.
pub fn build_template_vapor_decisions<'owner, 'arena>(
    view: NativeTemplateView<'owner, 'arena>,
) -> Result<TemplateVaporAnalysis<'owner, 'arena>, DecisionBuildError> {
    let file = view.file().ok_or(DecisionBuildError::IncompleteFile)?;
    let analysis = super::file::build_original_file_decisions(file)?;
    Ok(TemplateVaporAnalysis { view, analysis })
}
