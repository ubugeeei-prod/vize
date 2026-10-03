//! Whole setup eligibility classified in the sole original Program walk.

use crate::file::build::Facts;
use crate::file::vue::{ExposureIssueKind, VueExposure};
use crate::file::{BindingRef, FileArtifact, ScopeId, ScriptUnitId};
use alloc::{boxed::Box, vec::Vec};
use oxc_ast::ast::Program;
use oxc_parser::AdmittedProgram;
use vize_l0::{SourceBlock, Span};
use vize_l1::container::vue::ScriptView;
use vize_l1::embed::Lang;

/// A primitive keyword on the genuine original TypeScript annotation node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupPrimitiveType {
    BigInt,
    Boolean,
    Null,
    Number,
    String,
    Symbol,
    Undefined,
}

pub(crate) struct SetupAnnotationRecord {
    pub unit: ScriptUnitId,
    pub span: Span,
    pub kind: SetupPrimitiveType,
}

/// Deferred original receipts keep unannotated File facts to one pointer.
/// Only the original variable event creates this private storage owner.
#[derive(Default)]
pub(crate) struct SetupAnnotationStorage {
    rows: Vec<SetupAnnotationRecord>,
}

impl SetupAnnotationStorage {
    pub(crate) fn rows(&self) -> &[SetupAnnotationRecord] {
        &self.rows
    }
}

pub(super) fn record_annotation(
    facts: &mut Facts<'_>,
    unit: ScriptUnitId,
    span: Span,
    kind: SetupPrimitiveType,
) {
    facts
        .setup_annotations
        .get_or_insert_with(|| Box::new(SetupAnnotationStorage::default()))
        .rows
        .push(SetupAnnotationRecord { unit, span, kind });
}

/// An original annotation receipt borrowed from the same sealed File.
/// Its span includes the original colon and is never supplied by a caller.
/// ```compile_fail
/// use vize_l2::lang::js::SetupAnnotation;
/// fn forge() { let _ = SetupAnnotation { span: true }; }
/// ```
pub struct SetupAnnotation<'owner, 'arena> {
    file: &'owner FileArtifact<'arena>,
    original: &'owner SetupAnnotationRecord,
}

impl<'owner, 'arena> SetupAnnotation<'owner, 'arena> {
    #[must_use]
    pub fn file(&self) -> &'owner FileArtifact<'arena> {
        self.file
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.original.span
    }
    #[must_use]
    pub fn kind(&self) -> SetupPrimitiveType {
        self.original.kind
    }
}

pub(crate) fn initial(program: &Program<'_>) -> bool {
    let profile = program.source_type;
    profile.is_module()
        && !profile.is_unambiguous()
        && !profile.is_typescript_definition()
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

/// The original JS/TS setup body contains only direct const/let/var primitive
/// literals and empty statements, with only genuine primitive keyword annotations.
/// The actual File walk and original Program stay borrowed.
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
        if !matches!(exposure.script().lang(), Lang::Js | Lang::Ts) {
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

    /// Original source order from the sole variable walk, for this unit only.
    /// The complete File/Program capability grants the receipt, not raw indices.
    pub fn type_annotations(&self) -> impl Iterator<Item = SetupAnnotation<'owner, 'arena>> + '_ {
        self.exposure
            .file()
            .setup_annotations_for_unit(self.exposure.unit())
    }
}

impl<'arena> FileArtifact<'arena> {
    /// Private projection of real rows; public eligibility receipts choose the unit.
    pub(crate) fn setup_annotations_for_unit(
        &self,
        unit: ScriptUnitId,
    ) -> impl Iterator<Item = SetupAnnotation<'_, 'arena>> {
        self.setup_annotations()
            .iter()
            .filter(move |row| row.unit == unit)
            .map(move |original| SetupAnnotation {
                file: self,
                original,
            })
    }
}
