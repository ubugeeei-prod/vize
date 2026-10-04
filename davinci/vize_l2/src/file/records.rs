//! Neutral file declarations and references; no framework access spelling.

use crate::resolution::{BindingId, Usage};
use oxc_parser::AdmittedProgram;
use vize_l0::{Span, String};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptUnitId(pub(crate) u32);

impl ScriptUnitId {
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopeId(pub(crate) u32);

impl ScopeId {
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Value,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptProfile {
    pub typescript: bool,
    pub jsx: bool,
    pub module: bool,
}

#[derive(Debug)]
pub struct ScriptUnit {
    pub id: ScriptUnitId,
    pub span: Span,
    pub profile: ScriptProfile,
    pub scope: ScopeId,
    walk: ProgramWalkState,
    pub(crate) origin: ProgramOrigin,
}

/// Original arena storage survives movement of the normally owned Program root.
#[derive(Debug)]
pub(crate) struct ProgramOrigin {
    pub(crate) body: usize,
    pub(crate) length: usize,
    pub(crate) has_call: bool,
    has_comments: bool,
    pub(crate) has_export: bool,
    pub(crate) reserved_binding: bool,
    pub(crate) setup_eligible: bool,
    pub(crate) ordinary_empty_eligible: bool,
}

impl ProgramOrigin {
    pub(crate) fn checked(admitted: &AdmittedProgram<'_, '_>) -> Self {
        let program = admitted.program();
        Self {
            body: program.body.as_ptr() as usize,
            length: program.body.len(),
            has_call: false,
            has_comments: !program.comments.is_empty(),
            has_export: false,
            reserved_binding: false,
            setup_eligible: crate::lang::js::file::setup::initial(program)
                && !admitted.has_legacy_literals(),
            ordinary_empty_eligible: crate::lang::js::file::ordinary::initial(admitted),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProgramWalkState {
    Pending,
    Complete,
    Interrupted,
}

impl ScriptUnit {
    pub(crate) fn new(
        id: ScriptUnitId,
        span: Span,
        profile: ScriptProfile,
        scope: ScopeId,
        origin: ProgramOrigin,
    ) -> Self {
        Self {
            id,
            span,
            profile,
            scope,
            walk: ProgramWalkState::Pending,
            origin,
        }
    }
    pub(crate) fn complete_walk(&mut self) {
        if self.walk == ProgramWalkState::Pending {
            self.walk = ProgramWalkState::Complete;
        }
    }
    pub(crate) fn interrupt_walk(&mut self) {
        if self.walk == ProgramWalkState::Pending {
            self.walk = ProgramWalkState::Interrupted;
        }
    }
    pub(crate) fn walk_completed(&self) -> bool {
        self.walk == ProgramWalkState::Complete
    }
    /// Whether the existing walk observed an original call, constructor, tagged
    /// template or dynamic import. This observation survives rollback and unwind.
    ///
    /// A false observation on an incomplete File does not prove absence. This
    /// getter grants neither completed-walk nor native File admission authority.
    #[must_use]
    pub fn has_invocations(&self) -> bool {
        self.origin.has_call
    }
    /// Whether the original admitted Program contains parser-retained comments.
    /// This fact is retained before observer callbacks and survives interruption.
    /// It grants neither completed-walk nor authored filename authority.
    #[must_use]
    pub fn has_comments(&self) -> bool {
        self.origin.has_comments
    }
    /// Actual unit interruption is separate from the stored syntax issue slice.
    #[must_use]
    pub fn interruption(&self) -> Option<FileIssue> {
        (self.walk == ProgramWalkState::Interrupted).then_some(FileIssue {
            unit: self.id,
            span: self.span,
            kind: FileIssueKind::InterruptedProgram,
        })
    }
}

#[derive(Debug)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    /// Authored introduction/origin, not the extent of lookup visibility.
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Const,
    Let,
    Var,
    Import,
    Function,
    Parameter,
    Class,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitializerKind {
    PrimitiveLiteral,
    Function,
    Unknown,
    /// The original decoded StringLiteral contains NUL or carriage return.
    /// This is a source value fact; each target owns its admission policy.
    PrimitiveStringWithNulOrCr,
    /// The original StringLiteral's OXC flag records unpaired UTF-16 units.
    PrimitiveStringWithLoneSurrogates,
}

impl InitializerKind {
    /// Preserve the common primitive family across source value refinements.
    #[must_use]
    pub const fn is_primitive(self) -> bool {
        matches!(
            self,
            Self::PrimitiveLiteral
                | Self::PrimitiveStringWithNulOrCr
                | Self::PrimitiveStringWithLoneSurrogates
        )
    }
}

#[derive(Debug)]
pub struct Declaration {
    pub id: BindingId,
    pub unit: ScriptUnitId,
    pub scope: ScopeId,
    pub name: String,
    pub span: Span,
    pub namespace: Namespace,
    pub kind: DeclarationKind,
    pub initializer: InitializerKind,
    pub import_source: Option<String>,
    pub imported_name: Option<String>,
    pub(crate) direct_program: bool,
}

impl Declaration {
    /// The original statement event was directly in the admitted Program body.
    #[must_use]
    pub fn is_direct_program(&self) -> bool {
        self.direct_program
    }

    #[must_use]
    pub fn script_unit(&self) -> Option<ScriptUnitId> {
        Some(self.unit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceTarget {
    Resolved(BindingId),
    Unresolved,
}

#[derive(Debug)]
pub struct Reference {
    pub unit: ScriptUnitId,
    pub scope: ScopeId,
    pub name: String,
    pub span: Span,
    pub usage: Usage,
    pub shorthand: bool,
    pub constructor: bool,
    pub namespace: Namespace,
    pub target: ReferenceTarget,
}

#[derive(Debug)]
pub struct Export {
    pub unit: ScriptUnitId,
    pub name: String,
    pub local: Option<BindingId>,
    pub source: Option<String>,
    pub namespace: Namespace,
    pub span: Span,
    pub(crate) local_reference: Option<usize>,
}

#[derive(Debug)]
pub struct Import {
    pub unit: ScriptUnitId,
    pub source: String,
    pub namespace: Namespace,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileIssue {
    pub unit: ScriptUnitId,
    pub span: Span,
    pub kind: FileIssueKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileIssueKind {
    InvalidSource,
    InvalidProfile,
    InvalidSpan,
    DuplicateUnit,
    DuplicateDeclaration,
    UnresolvedReference,
    UnsupportedSyntax,
    BindingLimit,
    UnsupportedComponent,
    InterruptedProgram,
    InterruptedTemplate,
    ActiveTemplateWalk,
}

/// A retained failure at a real template factory, without a fabricated unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateIssue {
    pub node: Option<vize_l0::id::NodeId>,
    pub span: Span,
    pub kind: FileIssueKind,
}
