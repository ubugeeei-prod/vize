//! Genuine original styled-template custody through the same sole SSR walk.

use super::{NativeSsrBuildError, NativeSsrFileAnalysis, SsrFacts, build_ssr_file_decisions};
use crate::decision::DecisionTables;
use vize_l2::{
    artifact::Artifact,
    file::FileArtifact,
    lang::js::{NativeScopedTemplateView, NativeTemplateFile},
};

/// Original style/parser/selected-File authority cannot be replaced by neutral
/// analysis, generated CSS, a scope flag or independently paired decisions.
///
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateView;
/// use vize_l3::decision::ssr::build_native_scoped_ssr_file_decisions;
/// fn promote(view: NativeTemplateView<'_, '_>) { let _ = build_native_scoped_ssr_file_decisions(view); }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeScopedTemplateView;
/// use vize_l3::decision::ssr::{NativeSsrFileAnalysis, NativeTemplateScopedSsrAnalysis};
/// fn forge<'o, 'a>(view: NativeScopedTemplateView<'o, 'a>, analysis: NativeSsrFileAnalysis<'o, 'a>) {
///     let _ = NativeTemplateScopedSsrAnalysis { view, analysis };
/// }
/// ```
/// ```compile_fail
/// use vize_l3::decision::ssr::NativeTemplateScopedSsrAnalysis;
/// fn copy(receipt: NativeTemplateScopedSsrAnalysis<'_, '_>) { let _ = receipt.clone(); }
/// ```
pub struct NativeTemplateScopedSsrAnalysis<'owner, 'arena> {
    view: NativeScopedTemplateView<'owner, 'arena>,
    analysis: NativeSsrFileAnalysis<'owner, 'arena>,
}
impl<'owner, 'arena> NativeTemplateScopedSsrAnalysis<'owner, 'arena> {
    #[must_use]
    pub fn scoped(&self) -> &NativeScopedTemplateView<'owner, 'arena> {
        &self.view
    }
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
    pub fn tables(&self) -> &DecisionTables {
        self.analysis.tables()
    }
    #[must_use]
    pub fn ssr(&self) -> Option<&SsrFacts<'owner, 'arena>> {
        self.analysis.ssr()
    }
}

/// Consume the genuine checked lower style receipt, derive its sole File and
/// run the existing SSR collector once. The ordinary StyledSource guard stays.
pub fn build_native_scoped_ssr_file_decisions<'owner, 'arena>(
    view: NativeScopedTemplateView<'owner, 'arena>,
) -> Result<NativeTemplateScopedSsrAnalysis<'owner, 'arena>, NativeSsrBuildError> {
    let analysis = build_ssr_file_decisions(view.file()).map_err(NativeSsrBuildError::Decision)?;
    Ok(NativeTemplateScopedSsrAnalysis { view, analysis })
}
