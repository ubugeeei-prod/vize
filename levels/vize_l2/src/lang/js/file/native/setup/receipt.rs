//! A short original setup/Program/unit join, without a second syntax walk.

use super::{NativeSyntax, NativeTemplateFile};
use crate::file::{FileArtifact, ScopeId, ScriptUnit, ScriptUnitId};
use oxc_parser::AdmittedProgram;
use vize_l0::Span;
use vize_l1::markup::NativeScriptSelection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSetupIssueKind {
    MissingOwner,
    IncompleteFile,
    Source,
    Profile,
    MissingUnit,
    ProgramOrigin,
    EmptyProgram,
    Scope,
    UnsupportedSyntax,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeSetupIssue {
    pub span: Span,
    pub kind: NativeSetupIssueKind,
}

/// Same selected owner, whole original syntax, and actual completed unit row.
/// Only the existing sole declaration walk certifies bounded setup eligibility.
/// The whole descriptor envelope and output policy remain separate obligations.
///
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// fn forge() { let _ = NativeSelectedSetup { unit: 1 }; }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeSelectedSetup;
/// fn copy(view: NativeSelectedSetup<'_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateFile;
/// fn discard(owner: NativeTemplateFile<'_>) {
///     let setup = owner.setup().unwrap();
///     drop(owner);
///     let _ = setup.program();
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::NativeTemplateFile;
/// fn separate(owner: &mut NativeTemplateFile<'_>) { owner.retained_setup.take(); }
/// ```
/// A caller cannot replace the real Program with a foreign parse:
/// ```compile_fail
/// use vize_l1::embed::syntax::NativeSyntax;
/// use vize_l2::lang::js::NativeTemplateFile;
/// fn pair<'a>(owner: &NativeTemplateFile<'a>, foreign: NativeSyntax<'a>) {
///     let _ = owner.setup(foreign);
/// }
/// ```
/// The Program borrow cannot outlive the normally retained owner:
/// ```compile_fail
/// use oxc_ast::ast::Program;
/// use vize_l2::lang::js::NativeTemplateFile;
/// fn escape<'a>(owner: NativeTemplateFile<'a>) -> &'a Program<'a> {
///     owner.retained_setup().unwrap().program().unwrap()
/// }
/// ```
pub struct NativeSelectedSetup<'owner, 'arena> {
    owner: &'owner NativeTemplateFile<'arena>,
    syntax: &'owner NativeSyntax<'arena>,
    selected: NativeScriptSelection<'owner, 'arena>,
    program: AdmittedProgram<'owner, 'arena>,
    unit: &'owner ScriptUnit,
    file: &'owner FileArtifact<'arena>,
}
impl<'owner, 'arena> NativeSelectedSetup<'owner, 'arena> {
    #[must_use]
    pub fn owner(&self) -> &'owner NativeTemplateFile<'arena> {
        self.owner
    }
    #[must_use]
    pub fn syntax(&self) -> &'owner NativeSyntax<'arena> {
        self.syntax
    }
    #[must_use]
    pub fn selected(&self) -> &NativeScriptSelection<'owner, 'arena> {
        &self.selected
    }
    #[must_use]
    pub fn program(&self) -> &AdmittedProgram<'owner, 'arena> {
        &self.program
    }
    #[must_use]
    pub fn unit(&self) -> ScriptUnitId {
        self.unit.id
    }
    #[must_use]
    pub fn unit_record(&self) -> &'owner ScriptUnit {
        self.unit
    }
    #[must_use]
    pub fn scope(&self) -> ScopeId {
        self.unit.scope
    }
    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
        self.file
    }
}

pub(super) fn checked<'owner, 'arena>(
    owner: &'owner NativeTemplateFile<'arena>,
) -> Result<NativeSelectedSetup<'owner, 'arena>, NativeSetupIssue> {
    use NativeSetupIssueKind as Kind;
    let span = owner.selected().component().block().span();
    let reject = |kind| NativeSetupIssue { span, kind };
    let syntax = owner
        .retained_setup()
        .ok_or_else(|| reject(Kind::MissingOwner))?;
    let selected = owner
        .selected()
        .setup()
        .ok_or_else(|| reject(Kind::MissingUnit))?;
    let span = selected.block().span();
    let reject = |kind| NativeSetupIssue { span, kind };
    owner.view().map_err(|_| reject(Kind::IncompleteFile))?;
    let file = owner.file().ok_or_else(|| reject(Kind::IncompleteFile))?;
    let joined = super::identity::checked(
        owner.selected(),
        syntax,
        file.units(),
        file.scopes(),
        file.artifact().source(),
    )?;
    Ok(NativeSelectedSetup {
        owner,
        syntax,
        selected: joined.selected,
        program: joined.program,
        unit: joined.unit,
        file,
    })
}
