//! A short original setup/Program/unit join, without a second syntax walk.

use super::{NativeSyntax, NativeTemplateFile};
use crate::file::{FileArtifact, ScopeId, ScriptProfile, ScriptUnit, ScriptUnitId};
use oxc_parser::{AdmittedProgram, ParseOptions};
use vize_l0::Span;
use vize_l1::{embed::Lang, markup::NativeScriptSelection};

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
    let program = syntax
        .admitted_program()
        .ok_or_else(|| reject(Kind::UnsupportedSyntax))?;
    if owner.selected().ordinary().is_some() || owner.selected().has_styles() {
        return Err(reject(Kind::UnsupportedSyntax));
    }
    if syntax.source().span() != span
        || !core::ptr::eq(
            syntax.source().authored_root(),
            selected.block().root_source(),
        )
        || !core::ptr::eq(program.source(), selected.block().source())
        || !core::ptr::eq(file.artifact().source(), selected.block().root_source())
    {
        return Err(reject(Kind::Source));
    }
    let profile = program.source_type();
    if !profile.is_module()
        || profile.is_unambiguous()
        || profile.is_jsx()
        || profile.is_typescript_definition()
        || program.program().source_type != profile
        || program.options() != ParseOptions::default()
        || profile.is_typescript() != (selected.lang() == Lang::Ts)
        || !matches!(selected.lang(), Lang::Js | Lang::Ts)
    {
        return Err(reject(Kind::Profile));
    }
    if program.program().body.is_empty() {
        return Err(reject(Kind::EmptyProgram));
    }
    if file.units().len() != 1 {
        return Err(reject(Kind::MissingUnit));
    }
    let unit = file
        .units()
        .first()
        .ok_or_else(|| reject(Kind::MissingUnit))?;
    if unit.id.index() as usize != selected.container_index()
        || unit.span != span
        || unit.profile
            != (ScriptProfile {
                typescript: profile.is_typescript(),
                jsx: false,
                module: true,
            })
    {
        return Err(reject(Kind::Profile));
    }
    if !unit.walk_completed() {
        return Err(reject(Kind::IncompleteFile));
    }
    if unit.origin.body != program.program().body.as_ptr() as usize
        || unit.origin.length != program.program().body.len()
    {
        return Err(reject(Kind::ProgramOrigin));
    }
    let scope = file
        .scopes()
        .get(unit.scope.index() as usize)
        .ok_or_else(|| reject(Kind::Scope))?;
    if unit.scope.index() == 0
        || scope.id != unit.scope
        || scope.parent != Some(ScopeId(0))
        || scope.span != span
    {
        return Err(reject(Kind::Scope));
    }
    if !unit.origin.setup_eligible
        || unit.origin.has_call
        || unit.origin.has_export
        || unit.origin.reserved_binding
    {
        return Err(reject(Kind::UnsupportedSyntax));
    }
    Ok(NativeSelectedSetup {
        owner,
        syntax,
        selected,
        program,
        unit,
        file,
    })
}
