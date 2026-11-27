//! Genuine original setup custody specializes Vapor's one canonical walk.

use super::VaporAnalysis;
use crate::decision::{DecisionAnalysis, DecisionBuildError, DecisionTables, build};
use vize_l2::{
    ir::L2Ir,
    lang::js::{NativeSelectedSetup, NativeTemplateL2Ir, NativeTemplateView},
    template::L2Template,
};

/// The real selected setup and its own completed template stay borrowed.
/// Neither a neutral File nor an independently supplied template grants this
/// capability; original expression rows are joined by the sole event walk.
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// use vize_l3::decision::vapor::SelectedSetupVaporAnalysis;
/// fn forge(setup: &NativeSelectedSetup<'_, '_>) {
///     let _ = SelectedSetupVaporAnalysis { setup };
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// use vize_l3::decision::vapor::build_selected_setup_vapor_decisions;
/// fn discard(setup: NativeSelectedSetup<'_, '_>) {
///     let analysis = build_selected_setup_vapor_decisions(&setup).unwrap();
///     drop(setup);
///     let _ = analysis.setup();
/// }
/// ```
pub struct SelectedSetupVaporAnalysis<'view, 'owner, 'arena> {
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
    template: NativeTemplateView<'owner, 'arena>,
    analysis: DecisionAnalysis<'owner, 'arena>,
}

impl<'view, 'owner, 'arena> SelectedSetupVaporAnalysis<'view, 'owner, 'arena> {
    #[must_use]
    pub fn setup(&self) -> &'view NativeSelectedSetup<'owner, 'arena> {
        self.setup
    }
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateL2Ir<'arena> {
        self.template.owner()
    }
    #[must_use]
    pub fn l2ir(&self) -> &'owner L2Ir<'arena> {
        self.setup.l2ir()
    }
    #[must_use]
    pub fn original_attributes(
        &self,
    ) -> Option<&crate::decision::OriginalAttributeAnalysis<'owner, 'arena>> {
        self.analysis.original_attributes()
    }
    #[must_use]
    pub fn artifact(&self) -> &'owner L2Template<'arena> {
        self.l2ir().artifact()
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

/// Derive both original File and template internally from the sealed setup.
/// No parsing, declaration walk, name lookup or independent analysis is added.
pub fn build_selected_setup_vapor_decisions<'view, 'owner, 'arena>(
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
) -> Result<SelectedSetupVaporAnalysis<'view, 'owner, 'arena>, DecisionBuildError> {
    let template = setup
        .owner()
        .view()
        .map_err(|_| DecisionBuildError::IncompleteFile)?;
    let file = template.file().ok_or(DecisionBuildError::IncompleteFile)?;
    if !core::ptr::eq(file, setup.l2ir()) {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build::build_with_vapor_setup(setup)?;
    Ok(SelectedSetupVaporAnalysis {
        setup,
        template,
        analysis,
    })
}
