//! Same original setup/template owner specializes the sole canonical DOM walk.

use crate::decision::{
    DecisionAnalysis, DecisionBuildError, DecisionTables, build,
    dom::{
        DomAnalysis, LiteralExpressions,
        vue::{VueReadKind, VueRenderExpression, policy::FileReads},
    },
    policy::TargetPolicy,
};
use vize_l0::id::NodeId;
use vize_l2::{
    artifact::L2Template,
    file::{BindingRef, DeclarationKind, L2Ir},
    lang::js::{NativeSelectedSetup, NativeTemplateView},
    resolution::{Occurrence, Usage},
};

/// Native runtime reads borrow the normal whole setup owner and its own template.
/// A caller cannot pair another File or substitute a neutral analysis.
/// ```compile_fail
/// use vize_l2::lang::js::{NativeSelectedSetup, NativeTemplateView};
/// use vize_l3::decision::DecisionAnalysis;
/// use vize_l3::decision::template::SelectedSetupDomAnalysis;
/// fn forge<'v, 'o, 'a>(setup: &'v NativeSelectedSetup<'o, 'a>,
///     template: NativeTemplateView<'o, 'a>, analysis: DecisionAnalysis<'o, 'a>) {
///     let _ = SelectedSetupDomAnalysis { setup, template, analysis };
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// use vize_l3::decision::template::build_selected_setup_dom_decisions;
/// fn discard(setup: NativeSelectedSetup<'_, '_>) {
///     let result = build_selected_setup_dom_decisions(&setup).unwrap();
///     drop(setup);
///     let _ = result.setup();
/// }
/// ```
/// ```compile_fail
/// use vize_l3::decision::{DecisionAnalysis, template::SelectedSetupDomAnalysis};
/// fn neutral<'v, 'o, 'a>(result: SelectedSetupDomAnalysis<'v, 'o, 'a>)
///     -> DecisionAnalysis<'o, 'a> { result.analysis }
/// ```
pub struct SelectedSetupDomAnalysis<'view, 'owner, 'arena> {
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
    template: NativeTemplateView<'owner, 'arena>,
    analysis: DecisionAnalysis<'owner, 'arena>,
}

impl<'view, 'owner, 'arena> SelectedSetupDomAnalysis<'view, 'owner, 'arena> {
    #[must_use]
    pub fn setup(&self) -> &'view NativeSelectedSetup<'owner, 'arena> {
        self.setup
    }
    #[must_use]
    pub fn l2ir(&self) -> &'owner L2Ir<'arena> {
        self.setup.l2ir()
    }
    #[must_use]
    pub fn owner(&self) -> &'owner vize_l2::lang::js::NativeTemplateFile<'arena> {
        self.template.owner()
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
    pub fn expression(&self, node: NodeId) -> Option<&VueRenderExpression<'owner, 'arena>> {
        self.dom()?.vue_expressions.get(node)
    }
}

struct SelectedReads<'view, 'owner, 'arena>(&'view NativeSelectedSetup<'owner, 'arena>);
impl<'owner, 'arena> FileReads<'owner, 'arena> for SelectedReads<'_, 'owner, 'arena> {
    const RECORD: bool = true;
    const SELECTED_SETUP: bool = true;
    const ORIGINAL_FOR: bool = true;
    fn classify(
        &self,
        occurrence: &Occurrence<'arena>,
        binding: BindingRef<'owner, 'arena>,
    ) -> Option<VueReadKind> {
        if occurrence.usage != Usage::Read {
            return None;
        }
        let declaration = self.0.binding(binding).ok()?.declaration()?;
        match declaration.kind {
            DeclarationKind::Const if declaration.initializer.is_primitive() => {
                Some(VueReadKind::SetupConst)
            }
            DeclarationKind::Let | DeclarationKind::Var => Some(VueReadKind::SetupLet),
            _ => None,
        }
    }
}

/// No external File/template, script role, completeness flag or runtime policy.
/// The existing Enter visit joins each original operand and immutable occurrence;
/// no AST walk, lookup or new resolution occurs. Only the bounded primitive
/// original-root For/static-or-current-value-singleton family receives separate
/// runtime eligibility; other callback body families retain typed refusals.
pub fn build_selected_setup_dom_decisions<'view, 'owner, 'arena>(
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
) -> Result<SelectedSetupDomAnalysis<'view, 'owner, 'arena>, DecisionBuildError> {
    let template = setup
        .owner()
        .view()
        .map_err(|_| DecisionBuildError::IncompleteFile)?;
    let file = template.file().ok_or(DecisionBuildError::IncompleteFile)?;
    if !core::ptr::eq(file, setup.l2ir()) {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build::build_with_original(
        file,
        TargetPolicy::Dom,
        &LiteralExpressions,
        &SelectedReads(setup),
    )?;
    Ok(SelectedSetupDomAnalysis {
        setup,
        template,
        analysis,
    })
}
