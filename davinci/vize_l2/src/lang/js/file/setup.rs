//! Whole setup eligibility classified in the sole original Program walk.

use crate::file::build::Facts;
use crate::file::vue::{ExposureIssueKind, VueExposure};
use crate::file::{BindingRef, FileArtifact, ScopeId, ScriptUnitId};
use oxc_ast::ast::Program;
use oxc_parser::AdmittedProgram;
use vize_l0::{SourceBlock, Span};
use vize_l1::container::vue::ScriptView;
use vize_l1::embed::Lang;

pub(crate) fn initial(program: &Program<'_>) -> bool {
    let profile = program.source_type;
    profile.is_module()
        && !profile.is_unambiguous()
        && !profile.is_typescript()
        && !profile.is_jsx()
        && program.directives.is_empty()
        && program.hashbang.is_none()
}

/// The current original unit was appended immediately before its one walk.
/// Rejection is sticky and cannot complete an interrupted walk.
pub(super) fn reject(facts: &mut Facts<'_>, id: ScriptUnitId) {
    if let Some(unit) = facts.units.last_mut()
        && unit.id == id
    {
        unit.origin.setup_eligible = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupIssueKind {
    Exposure(ExposureIssueKind),
    Profile,
    Scope,
    UnsupportedSyntax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupIssue {
    pub span: Span,
    pub kind: SetupIssueKind,
}

/// The original JS setup body contains only direct let/var primitive literals
/// and empty statements. The actual File walk and original Program stay borrowed.
/// This is script eligibility, not native template custody or product completion.
///
/// A caller cannot supply an eligibility flag or construct the carrier:
/// ```compile_fail
/// use vize_l2::lang::js::VueSetup;
/// fn forge() { let _ = VueSetup { exposure: true }; }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::VueSetup;
/// fn clone(view: VueSetup<'_, '_, '_, '_>) { let _ = view.clone(); }
/// ```
/// A live capability borrows the actual complete File:
/// ```compile_fail
/// use oxc_parser::AdmittedProgram;
/// use vize_l1::container::vue::ScriptView;
/// use vize_l2::{file::FileArtifact, lang::js::VueSetup};
/// fn discard<'d, 'p, 'a>(file: FileArtifact<'a>, script: ScriptView<'d, 'a>,
///     program: AdmittedProgram<'p, 'a>) {
///     let setup = VueSetup::checked(&file, script, program).unwrap();
///     drop(file);
///     let _ = setup.source();
/// }
/// ```
/// It also keeps the original parser observation borrowed:
/// ```compile_fail
/// use vize_l1::{container::vue::ScriptView, embed::syntax::NativeSyntax};
/// use vize_l2::{file::FileArtifact, lang::js::VueSetup};
/// fn discard<'d, 'a>(file: &FileArtifact<'a>, script: ScriptView<'d, 'a>,
///     syntax: NativeSyntax<'a>) {
///     let setup = VueSetup::checked(file, script, syntax.admitted_program().unwrap()).unwrap();
///     drop(syntax);
///     let _ = setup.source();
/// }
/// ```
pub struct VueSetup<'owner, 'descriptor, 'program, 'arena> {
    exposure: VueExposure<'owner, 'descriptor, 'program, 'arena>,
}

impl<'owner, 'descriptor, 'program, 'arena> VueSetup<'owner, 'descriptor, 'program, 'arena> {
    pub fn checked(
        file: &'owner FileArtifact<'arena>,
        script: ScriptView<'descriptor, 'arena>,
        program: AdmittedProgram<'program, 'arena>,
    ) -> Result<Self, SetupIssue> {
        let span = script.block().span();
        let reject = |kind| SetupIssue { span, kind };
        let exposure = VueExposure::checked(file, script, program)
            .map_err(|issue| reject(SetupIssueKind::Exposure(issue.kind)))?;
        if exposure.script().lang() != Lang::Js {
            return Err(reject(SetupIssueKind::Profile));
        }
        let unit = file
            .units()
            .iter()
            .find(|unit| unit.id == exposure.unit())
            .ok_or_else(|| reject(SetupIssueKind::Scope))?;
        if unit.scope.index() == 0
            || !file
                .scopes()
                .get(unit.scope.index() as usize)
                .is_some_and(|scope| {
                    scope.id == unit.scope && scope.parent == Some(ScopeId(0)) && scope.span == span
                })
        {
            return Err(reject(SetupIssueKind::Scope));
        }
        if !unit.origin.setup_eligible {
            return Err(reject(SetupIssueKind::UnsupportedSyntax));
        }
        Ok(Self { exposure })
    }

    #[must_use]
    pub fn exposure(&self) -> &VueExposure<'owner, 'descriptor, 'program, 'arena> {
        &self.exposure
    }

    #[must_use]
    pub fn source(&self) -> SourceBlock<'arena> {
        self.exposure.script().block()
    }

    /// Original declaration order, actual unit/root scope and same owner only.
    pub fn bindings(&self) -> impl Iterator<Item = BindingRef<'owner, 'arena>> + '_ {
        self.exposure
            .file()
            .bindings()
            .filter(|binding| self.exposure.binding(*binding).is_ok())
    }
}
