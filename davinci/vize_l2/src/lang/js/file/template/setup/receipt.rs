//! A short original setup/Program/unit join, without a second syntax walk.

use super::{EmbedSyntax, TemplateFile};
use crate::file::{FileArtifact, ScopeId, ScriptUnit, ScriptUnitId};
use oxc_parser::AdmittedProgram;
use vize_l0::Span;
use vize_l1::markup::ScriptSelection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedSetupIssueKind {
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
pub struct SelectedSetupIssue {
    pub span: Span,
    pub kind: SelectedSetupIssueKind,
}

/// Same selected owner, whole original syntax, and actual completed unit row.
/// Only the existing sole declaration walk certifies bounded setup eligibility.
/// The whole descriptor envelope and output policy remain separate obligations.
///
/// ```compile_fail
/// use vize_l2::lang::js::SelectedSetup;
/// fn forge() { let _ = SelectedSetup { unit: 1 }; }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::SelectedSetup;
/// fn copy(view: SelectedSetup<'_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::TemplateFile;
/// fn discard(owner: TemplateFile<'_>) {
///     let setup = owner.setup().unwrap();
///     drop(owner);
///     let _ = setup.program();
/// }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::TemplateFile;
/// fn separate(owner: &mut TemplateFile<'_>) { owner.retained_setup.take(); }
/// ```
/// A caller cannot replace the real Program with a foreign parse:
/// ```compile_fail
/// use vize_l1::embed::syntax::EmbedSyntax;
/// use vize_l2::lang::js::TemplateFile;
/// fn pair<'a>(owner: &TemplateFile<'a>, foreign: EmbedSyntax<'a>) {
///     let _ = owner.setup(foreign);
/// }
/// ```
/// The Program borrow cannot outlive the normally retained owner:
/// ```compile_fail
/// use oxc_ast::ast::Program;
/// use vize_l2::lang::js::TemplateFile;
/// fn escape<'a>(owner: TemplateFile<'a>) -> &'a Program<'a> {
///     owner.retained_setup().unwrap().program().unwrap()
/// }
/// ```
pub struct SelectedSetup<'owner, 'arena> {
    owner: &'owner TemplateFile<'arena>,
    syntax: &'owner EmbedSyntax<'arena>,
    selected: ScriptSelection<'owner, 'arena>,
    program: AdmittedProgram<'owner, 'arena>,
    unit: &'owner ScriptUnit,
    file: &'owner FileArtifact<'arena>,
}
impl<'owner, 'arena> SelectedSetup<'owner, 'arena> {
    #[must_use]
    pub fn owner(&self) -> &'owner TemplateFile<'arena> {
        self.owner
    }
    #[must_use]
    pub fn syntax(&self) -> &'owner EmbedSyntax<'arena> {
        self.syntax
    }
    #[must_use]
    pub fn selected(&self) -> &ScriptSelection<'owner, 'arena> {
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
    owner: &'owner TemplateFile<'arena>,
) -> Result<SelectedSetup<'owner, 'arena>, SelectedSetupIssue> {
    use SelectedSetupIssueKind as Kind;
    let span = owner.selected().component().block().span();
    let reject = |kind| SelectedSetupIssue { span, kind };
    let syntax = owner
        .retained_setup()
        .ok_or_else(|| reject(Kind::MissingOwner))?;
    let selected = owner
        .selected()
        .setup()
        .ok_or_else(|| reject(Kind::MissingUnit))?;
    let span = selected.block().span();
    let reject = |kind| SelectedSetupIssue { span, kind };
    owner.view().map_err(|_| reject(Kind::IncompleteFile))?;
    let file = owner.file().ok_or_else(|| reject(Kind::IncompleteFile))?;
    let joined = super::identity::checked(
        owner.selected(),
        syntax,
        file.units(),
        file.scopes(),
        file.artifact().source(),
    )?;
    Ok(SelectedSetup {
        owner,
        syntax,
        selected: joined.selected,
        program: joined.program,
        unit: joined.unit,
        file,
    })
}
