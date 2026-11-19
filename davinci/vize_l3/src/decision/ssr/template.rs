//! Original selected-template authority retained through the sole SSR walk.

use super::{SsrFileAnalysis, SsrFacts};
use crate::decision::{DecisionBuildError, DecisionTables};
use vize_l0::Span;
use vize_l2::{
    artifact::Artifact,
    file::FileArtifact,
    lang::js::{NativeTemplateFile, NativeTemplateView},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsrTemplateBuildError {
    Decision(DecisionBuildError),
    /// No scoped/global style semantics have been consumed for this SFC.
    StyledSource {
        span: Span,
    },
}

/// Original Descriptor-selected custody, distinct from neutral File analysis.
/// A caller cannot independently pair source, Component, File or decisions.
///
/// A complete neutral File cannot mint original-template output authority:
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_l3::decision::ssr::build_template_ssr_decisions;
/// fn promote(file: &FileArtifact<'_>) { let _ = build_template_ssr_decisions(file); }
/// ```
/// Diagnostic SSR analysis cannot construct this receipt:
/// ```compile_fail
/// use vize_l3::decision::ssr::{SsrFileAnalysis, TemplateSsrAnalysis};
/// fn forge<'f, 'a>(analysis: SsrFileAnalysis<'f, 'a>) {
///     let _ = TemplateSsrAnalysis { analysis };
/// }
/// ```
/// The neutral analysis and original completion capability remain private:
/// ```compile_fail
/// use vize_l3::decision::ssr::{SsrFileAnalysis, TemplateSsrAnalysis};
/// fn detach<'f, 'a>(receipt: TemplateSsrAnalysis<'f, 'a>)
///     -> SsrFileAnalysis<'f, 'a> { receipt.analysis }
/// ```
/// ```compile_fail
/// use vize_l3::decision::ssr::TemplateSsrAnalysis;
/// fn duplicate(receipt: TemplateSsrAnalysis<'_, '_>) { let _ = receipt.clone(); }
/// ```
/// The original owner cannot be dropped while the result remains live:
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateFile;
/// use vize_l3::decision::ssr::build_template_ssr_decisions;
/// fn discard(owner: NativeTemplateFile<'_>) {
///     let receipt = build_template_ssr_decisions(owner.view().unwrap()).unwrap();
///     drop(owner);
///     let _ = receipt.owner();
/// }
/// ```
pub struct TemplateSsrAnalysis<'owner, 'arena> {
    view: NativeTemplateView<'owner, 'arena>,
    analysis: SsrFileAnalysis<'owner, 'arena>,
}

impl<'owner, 'arena> TemplateSsrAnalysis<'owner, 'arena> {
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

/// Consume actual lower completion and derive its sole File internally.
/// Styles refuse from Descriptor metadata before the existing single walk.
/// No reconstructed File, additional AST pass or caller eligibility flag exists.
pub fn build_template_ssr_decisions<'owner, 'arena>(
    view: NativeTemplateView<'owner, 'arena>,
) -> Result<TemplateSsrAnalysis<'owner, 'arena>, SsrTemplateBuildError> {
    let selected = view.owner().selected();
    if selected.has_styles() {
        return Err(SsrTemplateBuildError::StyledSource {
            span: Span::new(0, selected.component().block().root_source().len() as u32),
        });
    }
    let file = view.file().ok_or(SsrTemplateBuildError::Decision(
        DecisionBuildError::IncompleteFile,
    ))?;
    let analysis =
        super::file::build_original_file_decisions(file).map_err(SsrTemplateBuildError::Decision)?;
    Ok(TemplateSsrAnalysis { view, analysis })
}
