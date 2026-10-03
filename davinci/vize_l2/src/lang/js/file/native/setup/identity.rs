//! Original setup identity and sole-walk eligibility shared by both checkpoints.
use super::{NativeSetupIssue, NativeSetupIssueKind};
use crate::file::{Scope, ScopeId, ScriptProfile, ScriptUnit};
use oxc_parser::{AdmittedProgram, ParseOptions};
use vize_l1::{
    embed::{Lang, syntax::NativeSyntax},
    markup::{NativeScriptSelection, NativeTemplateComponent},
};

pub(super) struct Joined<'owner, 'arena> {
    pub(super) selected: NativeScriptSelection<'owner, 'arena>,
    pub(super) program: AdmittedProgram<'owner, 'arena>,
    pub(super) unit: &'owner ScriptUnit,
}
pub(super) fn checked<'owner, 'arena>(
    component: &'owner NativeTemplateComponent<'arena>,
    syntax: &'owner NativeSyntax<'arena>,
    units: &'owner [ScriptUnit],
    scopes: &[Scope],
    source: &str,
) -> Result<Joined<'owner, 'arena>, NativeSetupIssue> {
    use NativeSetupIssueKind as Kind;
    let span = component.component().block().span();
    let reject = |kind| NativeSetupIssue { span, kind };
    let selected = component.setup().ok_or_else(|| reject(Kind::MissingUnit))?;
    let span = selected.block().span();
    let reject = |kind| NativeSetupIssue { span, kind };
    let program = syntax
        .admitted_program()
        .ok_or_else(|| reject(Kind::UnsupportedSyntax))?;
    if component.ordinary().is_some() || component.has_styles() {
        return Err(reject(Kind::UnsupportedSyntax));
    }
    if syntax.source().span() != span
        || !core::ptr::eq(
            syntax.source().authored_root(),
            selected.block().root_source(),
        )
        || !core::ptr::eq(program.source(), selected.block().source())
        || !core::ptr::eq(source, selected.block().root_source())
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
    if units.len() != 1 {
        return Err(reject(Kind::MissingUnit));
    }
    let unit = units.first().ok_or_else(|| reject(Kind::MissingUnit))?;
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
    let scope = scopes
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
    Ok(Joined {
        selected,
        program,
        unit,
    })
}
