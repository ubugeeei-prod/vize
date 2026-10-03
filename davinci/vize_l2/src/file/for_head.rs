//! Whole original heads and declarations stay beside their checked File node.

use super::{BindingRef, FileArtifact, ScopeId};
use crate::lang::js::{NativeForInput, RejectedNativeForInput};
use crate::op::{OriginalForId, OriginalForOp};
use crate::resolution::{
    BindingId, ForAliasDeclaration, ForAliasRole, ForResolution, ForResolutionError,
};
use alloc::boxed::Box;
use vize_l1::markup::NativeAttributeForHeadFailure;

pub(crate) struct ForRecord<'a> {
    pub original: Option<core::ptr::NonNull<OriginalForOp<'a>>>,
    pub enclosing: ScopeId,
    pub scope: ScopeId,
    pub resolution: ForResolution<'a>,
    pub value: Option<BindingId>,
    pub key: Option<BindingId>,
}

pub(crate) struct PendingFor<'a> {
    pub input: Option<NativeForInput<'a>>,
    pub resolution: Option<ForResolution<'a>>,
    pub enclosing: ScopeId,
    pub span: vize_l0::Span,
}

pub(crate) struct TemplateDeclarationRow {
    pub owner: OriginalForId,
    pub role: ForAliasRole,
}

#[derive(Debug)]
pub enum RejectedFileFor<'a> {
    Observation(NativeAttributeForHeadFailure<'a>),
    Syntax(Box<RejectedNativeForInput<'a>>),
    Resolution {
        input: Box<NativeForInput<'a>>,
        error: ForResolutionError,
    },
}

/// A short original declaration view, minted only from a real attached row.
/// Numeric IDs and an original AST alone do not mint this whole-owner borrow.
/// ```compile_fail
/// use vize_l2::{file::TemplateDeclaration, resolution::BindingId};
/// fn overwrite(declaration: &mut TemplateDeclaration<'_, '_>) {
///     declaration.id = BindingId::new(0);
/// }
/// ```
pub struct TemplateDeclaration<'f, 'a> {
    pub(crate) record: &'f ForRecord<'a>,
    pub(crate) row: &'f TemplateDeclarationRow,
    pub(crate) id: BindingId,
}

impl<'f, 'a> TemplateDeclaration<'f, 'a> {
    #[must_use]
    pub const fn id(&self) -> BindingId {
        self.id
    }
    #[must_use]
    pub const fn origin(&self) -> OriginalForId {
        self.row.owner
    }
    #[must_use]
    pub const fn scope(&self) -> ScopeId {
        self.record.scope
    }
    #[must_use]
    pub fn original(&self) -> Option<ForAliasDeclaration<'f, 'a>> {
        match self.row.role {
            ForAliasRole::Value => Some(self.record.resolution.value_declaration()),
            ForAliasRole::Key => self.record.resolution.key_declaration(),
        }
    }
}

/// The exact normally owned File remains borrowed beside the declaration row.
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// fn discard(file: FileArtifact<'_>) {
///     let original = file.template_declarations().next().unwrap();
///     drop(file);
///     let _ = original.declaration().original();
/// }
/// ```
pub struct FileTemplateDeclaration<'f, 'a> {
    file: &'f FileArtifact<'a>,
    declaration: TemplateDeclaration<'f, 'a>,
}

impl<'f, 'a> FileTemplateDeclaration<'f, 'a> {
    #[must_use]
    pub const fn file(&self) -> &'f FileArtifact<'a> {
        self.file
    }
    #[must_use]
    pub const fn declaration(&self) -> &TemplateDeclaration<'f, 'a> {
        &self.declaration
    }
    #[must_use]
    pub fn same_owner(&self, other: &FileTemplateDeclaration<'_, '_>) -> bool {
        core::ptr::eq(self.file, other.file)
    }
}

/// Numeric lookup is File-local; `for_head_for` joins an actual arena allocation.
#[derive(Clone, Copy)]
pub struct FileForHead<'f, 'a> {
    file: &'f FileArtifact<'a>,
    id: OriginalForId,
}

impl<'a> FileArtifact<'a> {
    /// Borrow genuine alias rows from this File; no script-unit is invented.
    pub fn template_declarations(&self) -> impl Iterator<Item = FileTemplateDeclaration<'_, 'a>> {
        self.facts.template_declarations.iter().filter_map(|row| {
            let record = self.facts.for_heads.get(row.owner.node())?;
            let id = match row.role {
                ForAliasRole::Value => record.value?,
                ForAliasRole::Key => record.key?,
            };
            Some(FileTemplateDeclaration {
                file: self,
                declaration: TemplateDeclaration { record, row, id },
            })
        })
    }
    #[must_use]
    pub fn for_head(&self, id: OriginalForId) -> Option<FileForHead<'_, 'a>> {
        self.facts.for_heads.get(id.node())?;
        Some(FileForHead { file: self, id })
    }
    #[must_use]
    pub fn for_head_for(&self, original: &OriginalForOp<'a>) -> Option<FileForHead<'_, 'a>> {
        let head = self.for_head(original.id())?;
        head.accepts(original).then_some(head)
    }
    #[must_use]
    pub fn rejected_for_heads(&self) -> &[RejectedFileFor<'a>] {
        &self.facts.rejected_for_heads
    }
    pub fn unattached_for_heads(&self) -> impl Iterator<Item = &NativeForInput<'a>> {
        self.facts
            .pending_for_heads
            .iter()
            .filter_map(|head| {
                head.input
                    .as_ref()
                    .or_else(|| head.resolution.as_ref().map(ForResolution::input))
            })
            .chain(
                self.facts.for_heads.iter().filter_map(|(_, head)| {
                    head.original.is_none().then_some(head.resolution.input())
                }),
            )
    }
}

impl<'f, 'a> FileForHead<'f, 'a> {
    #[must_use]
    pub const fn file(self) -> &'f FileArtifact<'a> {
        self.file
    }
    #[must_use]
    pub const fn id(self) -> OriginalForId {
        self.id
    }
    #[must_use]
    pub fn resolution(self) -> Option<&'f ForResolution<'a>> {
        Some(&self.file.facts.for_heads.get(self.id.node())?.resolution)
    }
    #[must_use]
    pub fn scope(self) -> Option<ScopeId> {
        Some(self.file.facts.for_heads.get(self.id.node())?.scope)
    }
    #[must_use]
    pub fn enclosing_scope(self) -> Option<ScopeId> {
        Some(self.file.facts.for_heads.get(self.id.node())?.enclosing)
    }
    #[must_use]
    pub fn accepts(self, original: &OriginalForOp<'a>) -> bool {
        self.file
            .facts
            .for_heads
            .get(self.id.node())
            .and_then(|record| record.original)
            .is_some_and(|pointer| core::ptr::eq(pointer.as_ptr(), original))
    }
    #[must_use]
    pub fn value(self) -> Option<BindingRef<'f, 'a>> {
        self.file
            .binding(self.file.facts.for_heads.get(self.id.node())?.value?)
    }
    #[must_use]
    pub fn key(self) -> Option<BindingRef<'f, 'a>> {
        self.file
            .binding(self.file.facts.for_heads.get(self.id.node())?.key?)
    }
}

impl<'f, 'a> BindingRef<'f, 'a> {
    #[must_use]
    pub fn template_declaration(self) -> Option<FileTemplateDeclaration<'f, 'a>> {
        Some(FileTemplateDeclaration {
            file: self.file(),
            declaration: self.file().facts.template_declaration(self.id())?,
        })
    }
}

impl<'a> super::RejectedFile<'a> {
    /// Read original owners from a refused or interrupted factory; no completion
    /// or actual arena allocation association is granted by these observations.
    pub fn original_for_inputs(&self) -> impl Iterator<Item = &NativeForInput<'a>> {
        self.facts
            .pending_for_heads
            .iter()
            .filter_map(|head| {
                head.input
                    .as_ref()
                    .or_else(|| head.resolution.as_ref().map(ForResolution::input))
            })
            .chain(
                self.facts
                    .for_heads
                    .iter()
                    .map(|(_, head)| head.resolution.input()),
            )
    }
    #[must_use]
    pub fn rejected_for_heads(&self) -> &[RejectedFileFor<'a>] {
        &self.facts.rejected_for_heads
    }
}
