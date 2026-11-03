//! Readonly original script receipts and Vue issue/declaration observations.

use oxc_span::SourceType;
use vize_l0::Span;
use vize_l2::file::{Namespace, ScopeId, ScriptProfile, ScriptUnitId};
use vize_l2::resolution::BindingId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueScriptRole {
    Ordinary,
    Setup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VueScriptReceipt {
    pub(super) unit: ScriptUnitId,
    pub(super) scope: ScopeId,
    pub(super) span: Span,
    pub(super) profile: ScriptProfile,
    pub(super) source_type: SourceType,
}
impl VueScriptReceipt {
    #[must_use]
    pub fn unit(&self) -> ScriptUnitId {
        self.unit
    }
    #[must_use]
    pub fn scope(&self) -> ScopeId {
        self.scope
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
    #[must_use]
    pub fn profile(&self) -> ScriptProfile {
        self.profile
    }
    /// Original parser profile retained by this actual role receipt.
    #[must_use]
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }
}

/// Actual declaration identity/role only; this grants no Vue runtime access kind.
#[derive(Debug)]
pub struct VueDeclaration {
    pub binding: BindingId,
    pub role: VueScriptRole,
    pub namespace: Namespace,
    pub initializer_span: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueFileIssueKind {
    DuplicateRole,
    OrdinaryAfterSetup,
    ScriptAfterTemplate,
    DuplicateTemplate,
    PreviousIssues,
    InvalidProgram(vize_l2::file::FileIssueKind),
    SetupExport,
    UnsupportedOptions,
    UnsupportedCall { optional: bool },
    UnsupportedInvocation,
    InvalidTemplateProfile,
    ConflictingScriptProfiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VueFileIssue {
    pub span: Span,
    pub unit: Option<ScriptUnitId>,
    pub scope: Option<ScopeId>,
    pub kind: VueFileIssueKind,
}
