//! Genuine original styled-template custody through the same sole SSR walk.

use super::{SsrTemplateBuildError, SsrFileAnalysis, SsrFacts};
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
/// use vize_l3::decision::ssr::build_scoped_template_ssr_decisions;
/// fn promote(view: NativeTemplateView<'_, '_>) { let _ = build_scoped_template_ssr_decisions(view); }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeScopedTemplateView;
/// use vize_l3::decision::ssr::{SsrFileAnalysis, TemplateScopedSsrAnalysis};
/// fn forge<'o, 'a>(view: NativeScopedTemplateView<'o, 'a>, analysis: SsrFileAnalysis<'o, 'a>) {
///     let _ = TemplateScopedSsrAnalysis { view, analysis };
/// }
/// ```
/// ```compile_fail
/// use vize_l3::decision::ssr::TemplateScopedSsrAnalysis;
/// fn copy(receipt: TemplateScopedSsrAnalysis<'_, '_>) { let _ = receipt.clone(); }
/// ```
pub struct TemplateScopedSsrAnalysis<'owner, 'arena> {
    view: NativeScopedTemplateView<'owner, 'arena>,
    analysis: SsrFileAnalysis<'owner, 'arena>,
}
impl<'owner, 'arena> TemplateScopedSsrAnalysis<'owner, 'arena> {
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
    pub fn ssr(&self) -> Option<&SsrFacts<'owner, 'arena>> {
        self.analysis.ssr()
    }
}

/// Consume the genuine checked lower style receipt, derive its sole File and
/// run the existing SSR collector once. The ordinary StyledSource guard stays.
pub fn build_scoped_template_ssr_decisions<'owner, 'arena>(
    view: NativeScopedTemplateView<'owner, 'arena>,
) -> Result<TemplateScopedSsrAnalysis<'owner, 'arena>, SsrTemplateBuildError> {
    let analysis = super::file::build_original_file_decisions(view.file())
        .map_err(SsrTemplateBuildError::Decision)?;
    Ok(TemplateScopedSsrAnalysis { view, analysis })
}
