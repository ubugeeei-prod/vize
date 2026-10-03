//! Same original setup/template owner specializes the sole canonical DOM walk.

use crate::decision::{
    DecisionBuildError, DecisionTables, NativeAnalysis, build,
    dom::{
        DomFacts, LiteralExpressions,
        vue::{VueReadKind, VueRenderExpression, policy::FileReads},
    },
    policy::TargetPolicy,
};
use vize_l0::id::NodeId;
use vize_l2::{
    artifact::Artifact,
    file::{BindingRef, DeclarationKind, FileArtifact, InitializerKind},
    lang::js::{NativeSelectedSetup, NativeTemplateView},
    resolution::{Occurrence, Usage},
};

/// Native runtime reads borrow the normal whole setup owner and its own template.
/// A caller cannot pair another File or substitute a neutral analysis.
/// ```compile_fail
/// use vize_l3::decision::native::NativeSelectedSetupDomAnalysis;
/// fn forge() { let _ = NativeSelectedSetupDomAnalysis { analysis: true }; }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// use vize_l3::decision::native::build_native_selected_setup_dom_decisions;
/// fn discard(setup: NativeSelectedSetup<'_, '_>) {
///     let result = build_native_selected_setup_dom_decisions(&setup).unwrap();
///     drop(setup);
///     let _ = result.setup();
/// }
/// ```
/// ```compile_fail
/// use vize_l3::decision::{NativeAnalysis, native::NativeSelectedSetupDomAnalysis};
/// fn neutral<'v, 'o, 'a>(result: NativeSelectedSetupDomAnalysis<'v, 'o, 'a>)
///     -> NativeAnalysis<'o, 'a> { result.analysis }
/// ```
pub struct NativeSelectedSetupDomAnalysis<'view, 'owner, 'arena> {
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
    template: NativeTemplateView<'owner, 'arena>,
    analysis: NativeAnalysis<'owner, 'arena>,
}

impl<'view, 'owner, 'arena> NativeSelectedSetupDomAnalysis<'view, 'owner, 'arena> {
    #[must_use]
    pub fn setup(&self) -> &'view NativeSelectedSetup<'owner, 'arena> {
        self.setup
    }
    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
        self.setup.file()
    }
    #[must_use]
    pub fn owner(&self) -> &'owner vize_l2::lang::js::NativeTemplateFile<'arena> {
        self.template.owner()
    }
    #[must_use]
    pub fn artifact(&self) -> &'owner Artifact<'arena> {
        self.file().artifact()
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
    pub fn dom(&self) -> Option<&DomFacts<'owner, 'arena>> {
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
    const NATIVE_SETUP: bool = true;
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
            DeclarationKind::Const
                if declaration.initializer == InitializerKind::PrimitiveLiteral =>
            {
                Some(VueReadKind::SetupConst)
            }
            DeclarationKind::Let | DeclarationKind::Var => Some(VueReadKind::SetupLet),
            _ => None,
        }
    }
}

/// No external File/template, script role, completeness flag or runtime policy.
/// The existing Enter visit joins each original operand and immutable occurrence;
/// no AST walk, lookup or new resolution occurs. For remains an Operation hole.
pub fn build_native_selected_setup_dom_decisions<'view, 'owner, 'arena>(
    setup: &'view NativeSelectedSetup<'owner, 'arena>,
) -> Result<NativeSelectedSetupDomAnalysis<'view, 'owner, 'arena>, DecisionBuildError> {
    let template = setup
        .owner()
        .view()
        .map_err(|_| DecisionBuildError::IncompleteFile)?;
    let file = template.file().ok_or(DecisionBuildError::IncompleteFile)?;
    if !core::ptr::eq(file, setup.file()) {
        return Err(DecisionBuildError::IncompleteFile);
    }
    let analysis = build::build_with(
        file.artifact(),
        TargetPolicy::Dom,
        &LiteralExpressions,
        Some(file),
        &SelectedReads(setup),
    )?;
    Ok(NativeSelectedSetupDomAnalysis {
        setup,
        template,
        analysis,
    })
}
