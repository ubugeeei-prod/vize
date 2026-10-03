//! Authentic setup declaration membership; runtime decisions belong to L3.

use super::{
    BindingRef, DeclarationKind, FileArtifact, Namespace, ScopeId, ScriptProfile, ScriptUnitId,
};
use oxc_parser::{AdmittedProgram, ParseOptions};
use vize_l0::Span;
use vize_l1::container::vue::{ScriptRole, ScriptView};
use vize_l1::embed::Lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExposureIssueKind {
    Source,
    Profile,
    Role,
    MissingUnit,
    ProgramOrigin,
    EmptyProgram,
    IncompleteFile,
    ScriptCall,
    ScriptExport,
    ReservedName,
    ForeignBinding,
    DirectDeclaration,
    Namespace,
    DeclarationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExposureIssue {
    pub kind: ExposureIssueKind,
    pub span: Span,
}

/// A privately checked view borrows the actual File and both original owners.
///
/// It proves declaration membership, not native template/head construction,
/// purity, ref classification, runtime spelling or finished SFC admission.
///
/// ```compile_fail
/// use vize_l2::file::vue::VueExposure;
/// fn copy(view: VueExposure<'_, '_, '_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use vize_l2::file::{FileArtifact, vue::VueExposure};
/// fn forge(file: &FileArtifact<'_>) { let _ = VueExposure { file }; }
/// ```
pub struct VueExposure<'f, 'd, 'p, 'a> {
    file: &'f FileArtifact<'a>,
    script: ScriptView<'d, 'a>,
    program: AdmittedProgram<'p, 'a>,
    unit: ScriptUnitId,
    scope: ScopeId,
}

impl<'f, 'd, 'p, 'a> VueExposure<'f, 'd, 'p, 'a> {
    pub fn checked(
        file: &'f FileArtifact<'a>,
        script: ScriptView<'d, 'a>,
        program: AdmittedProgram<'p, 'a>,
    ) -> Result<Self, ExposureIssue> {
        let reject = |kind| ExposureIssue {
            kind,
            span: script.block().span(),
        };
        if script.role() != ScriptRole::Setup {
            return Err(reject(ExposureIssueKind::Role));
        }
        if !core::ptr::eq(file.artifact().source(), script.source())
            || !core::ptr::eq(program.source(), script.block().source())
        {
            return Err(reject(ExposureIssueKind::Source));
        }
        let profile = program.source_type();
        if !profile.is_module()
            || profile.is_unambiguous()
            || profile.is_jsx()
            || profile.is_typescript_definition()
            || program.program().source_type != profile
            || program.options() != ParseOptions::default()
            || profile.is_typescript() != (script.lang() == Lang::Ts)
            || !matches!(script.lang(), Lang::Js | Lang::Ts)
        {
            return Err(reject(ExposureIssueKind::Profile));
        }
        let Some(unit) = file
            .units()
            .iter()
            .find(|unit| unit.id.index() as usize == script.container_index())
        else {
            return Err(reject(ExposureIssueKind::MissingUnit));
        };
        if unit.span != script.block().span()
            || unit.profile
                != (ScriptProfile {
                    typescript: profile.is_typescript(),
                    jsx: false,
                    module: true,
                })
        {
            return Err(reject(ExposureIssueKind::Profile));
        }
        if program.program().body.is_empty() {
            return Err(reject(ExposureIssueKind::EmptyProgram));
        }
        if unit.origin.body != program.program().body.as_ptr() as usize
            || unit.origin.length != program.program().body.len()
        {
            return Err(reject(ExposureIssueKind::ProgramOrigin));
        }
        if !file.is_complete() || !unit.walk_completed() {
            return Err(reject(ExposureIssueKind::IncompleteFile));
        }
        if unit.origin.has_call {
            return Err(reject(ExposureIssueKind::ScriptCall));
        }
        if unit.origin.has_export {
            return Err(reject(ExposureIssueKind::ScriptExport));
        }
        if unit.origin.reserved_binding {
            return Err(reject(ExposureIssueKind::ReservedName));
        }
        Ok(Self {
            file,
            script,
            program,
            unit: unit.id,
            scope: unit.scope,
        })
    }

    #[must_use]
    pub fn file(&self) -> &'f FileArtifact<'a> {
        self.file
    }

    #[must_use]
    pub fn unit(&self) -> ScriptUnitId {
        self.unit
    }

    #[must_use]
    pub fn scope(&self) -> ScopeId {
        self.scope
    }

    #[must_use]
    pub fn script(&self) -> ScriptView<'d, 'a> {
        self.script
    }

    #[must_use]
    pub fn program(&self) -> &AdmittedProgram<'p, 'a> {
        &self.program
    }

    /// Check a real borrowed row; numeric BindingIds from another file cannot pass.
    pub fn binding(
        &self,
        binding: BindingRef<'f, 'a>,
    ) -> Result<BindingRef<'f, 'a>, ExposureIssue> {
        let reject = |kind, span| ExposureIssue { kind, span };
        if !core::ptr::eq(binding.file(), self.file) {
            return Err(reject(
                ExposureIssueKind::ForeignBinding,
                self.script.block().span(),
            ));
        }
        let Some(declaration) = binding.declaration() else {
            return Err(reject(
                ExposureIssueKind::ForeignBinding,
                self.script.block().span(),
            ));
        };
        if declaration.script_unit() != Some(self.unit)
            || declaration.scope != self.scope
            || !declaration.is_direct_program()
        {
            return Err(reject(
                ExposureIssueKind::DirectDeclaration,
                declaration.span,
            ));
        }
        if declaration.namespace != Namespace::Value {
            return Err(reject(ExposureIssueKind::Namespace, declaration.span));
        }
        if reserved(declaration.name.as_str()) {
            return Err(reject(ExposureIssueKind::ReservedName, declaration.span));
        }
        if !matches!(
            declaration.kind,
            DeclarationKind::Let | DeclarationKind::Var
        ) {
            return Err(reject(ExposureIssueKind::DeclarationKind, declaration.span));
        }
        Ok(binding)
    }
}

pub(crate) fn reserved(name: &str) -> bool {
    matches!(
        name,
        "__proto__" | "__isScriptSetup" | "__returned__" | "__props" | "__expose"
    )
}
