//! Genuine original template completion retains its lower owner through L3.

use super::{NativeVaporFileAnalysis, VaporFacts};
use crate::decision::{DecisionBuildError, DecisionTables};
use vize_l2::{
    artifact::Artifact,
    file::FileArtifact,
    lang::js::{NativeTemplateFile, NativeTemplateView},
};

/// Only a moved original template completion can authorize native Vapor output.
///
/// Complete neutral File analysis cannot construct this capability:
/// ```compile_fail
/// use vize_l3::decision::vapor::{NativeVaporFileAnalysis, NativeTemplateVaporAnalysis};
/// fn promote(analysis: NativeVaporFileAnalysis<'_, '_>) {
///     let _ = NativeTemplateVaporAnalysis { analysis };
/// }
/// ```
/// Its original lower owner remains live until the analysis is released:
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateFile;
/// use vize_l3::decision::vapor::build_native_vapor_file_decisions;
/// fn discard(owner: NativeTemplateFile<'_>) {
///     let analysis = build_native_vapor_file_decisions(owner.view().unwrap()).unwrap();
///     drop(owner);
///     let _ = analysis.owner();
/// }
/// ```
/// A complete manually populated File is not original template completion:
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_l3::decision::vapor::build_native_vapor_file_decisions;
/// fn forge(file: &FileArtifact<'_>) { let _ = build_native_vapor_file_decisions(file); }
/// ```
pub struct NativeTemplateVaporAnalysis<'owner, 'arena> {
    view: NativeTemplateView<'owner, 'arena>,
    analysis: NativeVaporFileAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> NativeTemplateVaporAnalysis<'owner, 'arena> {
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateFile<'arena> {
        self.view.owner()
    }
    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
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
pub fn build_native_vapor_file_decisions<'owner, 'arena>(
    view: NativeTemplateView<'owner, 'arena>,
) -> Result<NativeTemplateVaporAnalysis<'owner, 'arena>, DecisionBuildError> {
    let file = view.file().ok_or(DecisionBuildError::IncompleteFile)?;
    let analysis = super::file::build_original_file_decisions(file)?;
    Ok(NativeTemplateVaporAnalysis { view, analysis })
}
