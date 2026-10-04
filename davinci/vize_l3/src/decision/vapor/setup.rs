//! Genuine original setup custody specializes Vapor's one canonical walk.

use super::VaporFacts;
use crate::decision::{DecisionBuildError, DecisionTables, NativeAnalysis, build};
use vize_l2::{
    artifact::Artifact,
    file::FileArtifact,
    lang::js::{NativeSelectedSetup, NativeTemplateFile, NativeTemplateView},
};

/// The real selected setup and its own completed template stay borrowed.
/// Neither a neutral File nor an independently supplied template grants this
/// capability; original expression rows are joined by the sole event walk.
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// use vize_l3::decision::vapor::NativeSelectedSetupVaporAnalysis;
/// fn forge(setup: &NativeSelectedSetup<'_, '_>) {
///     let _ = NativeSelectedSetupVaporAnalysis { setup };
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// use vize_l3::decision::vapor::build_native_selected_setup_vapor_decisions;
/// fn discard(setup: NativeSelectedSetup<'_, '_>) {
///     let analysis = build_native_selected_setup_vapor_decisions(&setup).unwrap();
///     drop(setup);
///     let _ = analysis.setup();
/// }
/// ```
pub struct NativeSelectedSetupVaporAnalysis<'view, 'owner, 'arena> {
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
    template: NativeTemplateView<'owner, 'arena>,
    analysis: NativeAnalysis<'owner, 'arena>,
}

impl<'view, 'owner, 'arena> NativeSelectedSetupVaporAnalysis<'view, 'owner, 'arena> {
    #[must_use]
    pub fn setup(&self) -> &'view NativeSelectedSetup<'owner, 'arena> {
        self.setup
    }
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateFile<'arena> {
        self.template.owner()
    }
    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
        self.setup.file()
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
    pub fn vapor(&self) -> Option<&VaporFacts<'owner, 'arena>> {
        self.analysis.vapor()
    }
}

/// Derive both original File and template internally from the sealed setup.
/// No parsing, declaration walk, name lookup or independent analysis is added.
pub fn build_native_selected_setup_vapor_decisions<'view, 'owner, 'arena>(
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
) -> Result<NativeSelectedSetupVaporAnalysis<'view, 'owner, 'arena>, DecisionBuildError> {
    let template = setup
        .owner()
        .view()
        .map_err(|_| DecisionBuildError::IncompleteFile)?;
    let file = template.file().ok_or(DecisionBuildError::IncompleteFile)?;
    if !core::ptr::eq(file, setup.file()) {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build::build_with_vapor_setup(setup)?;
    Ok(NativeSelectedSetupVaporAnalysis {
        setup,
        template,
        analysis,
    })
}
