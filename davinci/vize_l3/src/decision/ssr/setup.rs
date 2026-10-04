//! SSR reads retain the normal original setup and its completed template File.

use super::{NativeSsrBuildError, SsrFacts};
use crate::decision::{DecisionTables, NativeAnalysis, build};
use vize_l0::id::NodeId;
use vize_l2::{
    artifact::Artifact,
    file::{BindingRef, FileArtifact, FileResolution},
    lang::js::{NativeSelectedSetup, NativeTemplateFile, NativeTemplateView},
    resolution::Occurrence,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsrSetupReadKind {
    SetupConst,
    SetupLet,
}

/// One immutable actual occurrence, not a name-based access policy.
pub struct SsrSetupRead<'owner, 'arena> {
    pub(super) occurrence: &'owner Occurrence<'arena>,
    pub(super) binding: BindingRef<'owner, 'arena>,
    pub(super) kind: SsrSetupReadKind,
}
impl<'owner, 'arena> SsrSetupRead<'owner, 'arena> {
    #[must_use]
    pub fn occurrence(&self) -> &'owner Occurrence<'arena> {
        self.occurrence
    }
    #[must_use]
    pub const fn binding(&self) -> BindingRef<'owner, 'arena> {
        self.binding
    }
    #[must_use]
    pub const fn kind(&self) -> SsrSetupReadKind {
        self.kind
    }
}

/// Complete original resolution and read list published together or absent.
pub struct SsrSetupExpression<'owner, 'arena> {
    pub(super) resolution: FileResolution<'owner, 'arena>,
    pub(super) read: Option<SsrSetupRead<'owner, 'arena>>,
}
impl core::fmt::Debug for SsrSetupExpression<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SsrSetupExpression")
            .field("node", &self.resolution.node())
            .field("read_count", &self.read.as_slice().len())
            .finish_non_exhaustive()
    }
}
impl<'owner, 'arena> SsrSetupExpression<'owner, 'arena> {
    #[must_use]
    pub const fn resolution(&self) -> FileResolution<'owner, 'arena> {
        self.resolution
    }
    #[must_use]
    pub fn reads(&self) -> &[SsrSetupRead<'owner, 'arena>] {
        self.read.as_slice()
    }
}

/// Only the normally owned setup derives this same-File SSR analysis.
/// No neutral analysis, source, binding list or runtime policy can be paired.
/// ```compile_fail
/// use vize_l2::lang::js::{NativeSelectedSetup, NativeTemplateView};
/// use vize_l3::decision::{NativeAnalysis, ssr::NativeSelectedSetupSsrAnalysis};
/// fn forge<'v,'o,'a>(setup: &'v NativeSelectedSetup<'o,'a>,
///     template: NativeTemplateView<'o,'a>, analysis: NativeAnalysis<'o,'a>) {
///     let _ = NativeSelectedSetupSsrAnalysis { setup, template, analysis };
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// use vize_l3::decision::ssr::build_native_selected_setup_ssr_decisions;
/// fn discard(setup: NativeSelectedSetup<'_, '_>) {
///     let result = build_native_selected_setup_ssr_decisions(&setup).unwrap();
///     drop(setup);
///     let _ = result.setup();
/// }
/// ```
pub struct NativeSelectedSetupSsrAnalysis<'view, 'owner, 'arena> {
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
    template: NativeTemplateView<'owner, 'arena>,
    analysis: NativeAnalysis<'owner, 'arena>,
}
impl<'view, 'owner, 'arena> NativeSelectedSetupSsrAnalysis<'view, 'owner, 'arena> {
    #[must_use]
    pub fn setup(&self) -> &'view NativeSelectedSetup<'owner, 'arena> {
        self.setup
    }
    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
        self.setup.file()
    }
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateFile<'arena> {
        self.template.owner()
    }
    #[must_use]
    pub fn artifact(&self) -> &'owner Artifact<'arena> {
        self.file().artifact()
    }
    #[must_use]
    pub fn tables(&self) -> &DecisionTables {
        self.analysis.tables()
    }
    #[must_use]
    pub fn ssr(&self) -> Option<&SsrFacts<'owner, 'arena>> {
        self.analysis.ssr()
    }
    #[must_use]
    pub fn expression(&self, node: NodeId) -> Option<&SsrSetupExpression<'owner, 'arena>> {
        self.ssr()?.expression(node)
    }
}

/// Join original immutable reads in the existing sole canonical Enter stream.
/// Template resolution scope and setup declaration scope remain distinct.
pub fn build_native_selected_setup_ssr_decisions<'view, 'owner, 'arena>(
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
) -> Result<NativeSelectedSetupSsrAnalysis<'view, 'owner, 'arena>, NativeSsrBuildError> {
    let template = setup.owner().view().map_err(|_| {
        NativeSsrBuildError::Decision(crate::decision::DecisionBuildError::IncompleteFile)
    })?;
    if template.owner().selected().has_styles() {
        return Err(NativeSsrBuildError::StyledSource {
            span: vize_l0::Span::new(0, setup.file().artifact().source().len() as u32),
        });
    }
    let file = template.file().ok_or(NativeSsrBuildError::Decision(
        crate::decision::DecisionBuildError::IncompleteFile,
    ))?;
    if !core::ptr::eq(file, setup.file()) {
        return Err(NativeSsrBuildError::Decision(
            crate::decision::DecisionBuildError::IncompleteFile,
        ));
    }
    let analysis = build::build_with_ssr_setup(setup).map_err(NativeSsrBuildError::Decision)?;
    Ok(NativeSelectedSetupSsrAnalysis {
        setup,
        template,
        analysis,
    })
}
