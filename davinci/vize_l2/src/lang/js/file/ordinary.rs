//! Original empty-default-object eligibility in the sole Program walk.

use crate::file::build::Facts;
use crate::file::{
    Export, FileArtifact, Namespace, ScopeId, ScriptProfile, ScriptUnit, ScriptUnitId,
};
use oxc_parser::{AdmittedProgram, OriginalDefaultExport, ParseOptions};
use vize_l0::{SourceBlock, Span};
use vize_l1::container::vue::{ScriptRole, ScriptView};
use vize_l1::embed::Lang;

pub(crate) fn initial(admitted: &AdmittedProgram<'_, '_>) -> bool {
    let program = admitted.program();
    let profile = admitted.source_type();
    profile.is_module()
        && !profile.is_unambiguous()
        && !profile.is_typescript_definition()
        && !profile.is_jsx()
        && program.source_type == profile
        && admitted.options() == ParseOptions::default()
        && program.hashbang.is_none()
        && !admitted.has_legacy_literals()
        && admitted.sole_default_export().is_some()
}

pub(super) fn reject(facts: &mut Facts<'_>, id: ScriptUnitId) {
    if let Some(unit) = facts.units.last_mut()
        && unit.id == id
    {
        unit.origin.ordinary_empty_eligible = false;
    }
}

/// Original sole completed root script and its existing export row.
/// This borrows actual private File facts, not a supplied unit/span/eligibility.
/// It certifies neither descriptor membership nor template/product completion.
/// ```compile_fail
/// use vize_l2::lang::js::OrdinaryEmptyScript;
/// fn forge() { let _ = OrdinaryEmptyScript { unit: true }; }
/// ```
pub struct OrdinaryEmptyScript<'f> {
    unit: &'f ScriptUnit,
    export: &'f Export,
}

impl<'f> OrdinaryEmptyScript<'f> {
    pub(crate) fn from_facts(facts: &'f Facts<'_>) -> Option<Self> {
        let [unit] = facts.units.as_slice() else {
            return None;
        };
        let [export] = facts.exports.as_slice() else {
            return None;
        };
        let root = facts.scopes.first()?;
        if !facts.issues.is_empty()
            || !unit.walk_completed()
            || !unit.origin.ordinary_empty_eligible
            || !unit.origin.has_export
            || unit.origin.has_call
            || unit.origin.reserved_binding
            || !unit.profile.module
            || unit.profile.jsx
            || unit.scope != ScopeId(0)
            || root.id != ScopeId(0)
            || root.parent.is_some()
            || !facts.declarations.is_empty()
            || !facts.imports.is_empty()
            || !facts.references.is_empty()
            || export.unit != unit.id
            || export.name.as_str() != "default"
            || export.namespace != Namespace::Value
            || export.local.is_some()
            || export.source.is_some()
            || export.local_reference.is_some()
        {
            return None;
        }
        Some(Self { unit, export })
    }

    #[must_use]
    pub fn unit(&self) -> ScriptUnitId {
        self.unit.id
    }
    #[must_use]
    pub fn scope(&self) -> ScopeId {
        self.unit.scope
    }
    #[must_use]
    pub fn script_span(&self) -> Span {
        self.unit.span
    }
    #[must_use]
    pub fn statement_span(&self) -> Span {
        self.export.span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdinaryIssueKind {
    Source,
    Profile,
    Role,
    MissingUnit,
    ProgramOrigin,
    Scope,
    IncompleteFile,
    UnsupportedSyntax,
    Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrdinaryIssue {
    pub span: Span,
    pub kind: OrdinaryIssueKind,
}

/// A single original ordinary JS/TS module containing a direct empty default
/// object, actual prologue directives and empty statements, with zero bindings.
/// The complete File, descriptor and parser receipt remain borrowed together.
/// Options/Ref/template-context classification and output remain separate.
/// ```compile_fail
/// use vize_l2::lang::js::VueOrdinaryEmpty;
/// fn forge() { let _ = VueOrdinaryEmpty { file: true }; }
/// ```
/// ```compile_fail
/// use vize_l2::lang::js::VueOrdinaryEmpty;
/// fn copy(view: VueOrdinaryEmpty<'_, '_, '_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use oxc_parser::AdmittedProgram;
/// use vize_l1::container::vue::ScriptView;
/// use vize_l2::{file::FileArtifact, lang::js::VueOrdinaryEmpty};
/// fn discard<'d, 'p, 'a>(file: FileArtifact<'a>, script: ScriptView<'d, 'a>,
///     program: AdmittedProgram<'p, 'a>) {
///     let view = VueOrdinaryEmpty::checked(&file, script, program).unwrap();
///     drop(file);
///     let _ = view.source();
/// }
/// ```
/// ```compile_fail
/// use vize_l1::{container::vue::ScriptView, embed::syntax::NativeSyntax};
/// use vize_l2::{file::FileArtifact, lang::js::VueOrdinaryEmpty};
/// fn discard<'d, 'a>(file: &FileArtifact<'a>, script: ScriptView<'d, 'a>,
///     syntax: NativeSyntax<'a>) {
///     let view = VueOrdinaryEmpty::checked(file, script,
///         syntax.admitted_program().unwrap()).unwrap();
///     drop(syntax);
///     let _ = view.object_span();
/// }
/// ```
pub struct VueOrdinaryEmpty<'f, 'd, 'p, 'a> {
    file: &'f FileArtifact<'a>,
    script: ScriptView<'d, 'a>,
    original: OriginalDefaultExport<'p, 'a>,
    unit: ScriptUnitId,
}

impl<'f, 'd, 'p, 'a> VueOrdinaryEmpty<'f, 'd, 'p, 'a> {
    pub fn checked(
        file: &'f FileArtifact<'a>,
        script: ScriptView<'d, 'a>,
        admitted: AdmittedProgram<'p, 'a>,
    ) -> Result<Self, OrdinaryIssue> {
        let reject = |kind| OrdinaryIssue {
            span: script.block().span(),
            kind,
        };
        if script.role() != ScriptRole::Ordinary {
            return Err(reject(OrdinaryIssueKind::Role));
        }
        if !core::ptr::eq(file.artifact().source(), script.source())
            || !core::ptr::eq(admitted.source(), script.block().source())
        {
            return Err(reject(OrdinaryIssueKind::Source));
        }
        let profile = admitted.source_type();
        if !profile.is_module()
            || profile.is_unambiguous()
            || profile.is_jsx()
            || profile.is_typescript_definition()
            || admitted.program().source_type != profile
            || admitted.options() != ParseOptions::default()
            || profile.is_typescript() != (script.lang() == Lang::Ts)
            || !matches!(script.lang(), Lang::Js | Lang::Ts)
        {
            return Err(reject(OrdinaryIssueKind::Profile));
        }
        let [unit] = file.units() else {
            return Err(reject(OrdinaryIssueKind::MissingUnit));
        };
        if unit.id.index() as usize != script.container_index() {
            return Err(reject(OrdinaryIssueKind::MissingUnit));
        }
        if unit.span != script.block().span()
            || unit.profile
                != (ScriptProfile {
                    typescript: profile.is_typescript(),
                    jsx: false,
                    module: true,
                })
        {
            return Err(reject(OrdinaryIssueKind::Profile));
        }
        if unit.origin.body != admitted.program().body.as_ptr() as usize
            || unit.origin.length != admitted.program().body.len()
        {
            return Err(reject(OrdinaryIssueKind::ProgramOrigin));
        }
        if unit.scope != ScopeId(0)
            || !file.scopes().first().is_some_and(|root| {
                root.id == ScopeId(0)
                    && root.parent.is_none()
                    && root.span == Span::new(0, script.source().len() as u32)
            })
        {
            return Err(reject(OrdinaryIssueKind::Scope));
        }
        if !file.is_complete() || !unit.walk_completed() {
            return Err(reject(OrdinaryIssueKind::IncompleteFile));
        }
        let family = file
            .ordinary_empty_script()
            .ok_or_else(|| reject(OrdinaryIssueKind::UnsupportedSyntax))?;
        let original = admitted
            .sole_default_export()
            .ok_or_else(|| reject(OrdinaryIssueKind::UnsupportedSyntax))?;
        let statement = authored(script.block(), original.statement_span());
        let keyword = authored(script.block(), original.default_keyword_span());
        let object = authored(script.block(), original.declaration_span());
        if !matches!((statement, keyword, object), (Some(statement), Some(keyword), Some(object))
            if statement == family.statement_span()
                && statement.start.checked_add(6).is_some_and(|end| end <= keyword.start)
                && keyword.end.checked_sub(keyword.start) == Some(7)
                && keyword.end <= object.start && object.start < object.end
                && object.end <= statement.end)
        {
            return Err(reject(OrdinaryIssueKind::Span));
        }
        Ok(Self {
            file,
            script,
            original,
            unit: unit.id,
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
    pub fn script(&self) -> ScriptView<'d, 'a> {
        self.script
    }
    #[must_use]
    pub fn source(&self) -> SourceBlock<'a> {
        self.script.block()
    }
    #[must_use]
    pub fn program(&self) -> &AdmittedProgram<'p, 'a> {
        self.original.program()
    }
    #[must_use]
    pub fn original(&self) -> &OriginalDefaultExport<'p, 'a> {
        &self.original
    }
    #[must_use]
    pub fn statement_span(&self) -> Span {
        self.project(self.original.statement_span())
    }
    /// Clean original parsing starts this statement at its unescaped export token.
    #[must_use]
    pub fn export_keyword_span(&self) -> Span {
        let start = self.statement_span().start;
        Span::new(start, start + 6)
    }
    #[must_use]
    pub fn default_keyword_span(&self) -> Span {
        self.project(self.original.default_keyword_span())
    }
    #[must_use]
    pub fn object_span(&self) -> Span {
        self.project(self.original.declaration_span())
    }

    fn project(&self, span: oxc_span::Span) -> Span {
        let base = self.source().span().start;
        Span::new(base + span.start, base + span.end)
    }
}

fn authored(block: SourceBlock<'_>, span: oxc_span::Span) -> Option<Span> {
    block.source().get(span.start as usize..span.end as usize)?;
    Some(Span::new(
        block.span().start.checked_add(span.start)?,
        block.span().start.checked_add(span.end)?,
    ))
}
