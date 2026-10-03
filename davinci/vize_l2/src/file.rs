//! File-local semantic facts sealed with their actual canonical artifact.
//!
//! Bare indices identify rows only inside this owner. Construction belongs to
//! checked language producers, never a caller-supplied declaration/name list.

use crate::artifact::Artifact;
use crate::resolution::{BindingId, ResolutionTable};
use alloc::boxed::Box;
use vize_l0::id::NodeId;

pub(crate) mod build;
pub(crate) mod region;
pub use build::FileBuilder;
pub use region::{
    TemplateBody, TemplateChildRegion, TemplatePolicy, TemplateRegion, TemplateScope,
    TemplateWalkRegion,
};
mod for_head;
mod handler;
mod interpolation;
pub use for_head::{FileForHead, FileTemplateDeclaration, RejectedFileFor, TemplateDeclaration};
mod query;
pub use handler::{FileHandler, RejectedFileHandler};
pub use interpolation::{NativeFileInterpolation, NativeFileInterpolationState};
mod records;
pub use query::{
    HandlerLocalRef, PositionQueryError, ReferenceRef, ScopeRef, TemplateQueryError,
    TemplateSiteRef, TemplateSiteScope, TemplateSymbolRef,
};
mod template;
pub(crate) use records::ProgramOrigin;
pub mod vue;
pub use records::{
    Declaration, DeclarationKind, Export, FileIssue, FileIssueKind, Import, InitializerKind,
    Namespace, Reference, ReferenceTarget, Scope, ScopeId, ScriptProfile, ScriptUnit, ScriptUnitId,
    TemplateIssue,
};

/// Immutable file facts and the exact canonical tree built with them.
///
/// Arbitrary declarations cannot establish file ownership:
/// ```compile_fail
/// use vize_l2::{artifact::Artifact, file::FileArtifact};
/// fn forge(artifact: Artifact<'_>) {
///     let _ = FileArtifact { artifact, facts: Default::default() };
/// }
/// ```
pub struct FileArtifact<'a> {
    artifact: Artifact<'a>,
    facts: build::Facts<'a>,
}

impl<'a> FileArtifact<'a> {
    #[must_use]
    pub fn artifact(&self) -> &Artifact<'a> {
        &self.artifact
    }

    #[must_use]
    pub fn units(&self) -> &[ScriptUnit] {
        &self.facts.units
    }

    #[must_use]
    pub fn scopes(&self) -> &[Scope] {
        &self.facts.scopes
    }

    #[must_use]
    pub fn binding(&self, id: BindingId) -> Option<BindingRef<'_, 'a>> {
        if self.facts.declarations.get(id.index() as usize).is_none()
            && self.facts.template_declaration(id).is_none()
        {
            return None;
        }
        Some(BindingRef { file: self, id })
    }

    /// Script rows followed by authentic original-template rows in mint order.
    pub fn bindings(&self) -> impl Iterator<Item = BindingRef<'_, 'a>> {
        self.facts
            .declarations
            .iter()
            .map(|declaration| BindingRef {
                file: self,
                id: declaration.id,
            })
            .chain(self.template_declarations().map(|declaration| BindingRef {
                file: self,
                id: declaration.declaration().id(),
            }))
    }

    #[must_use]
    pub fn references(&self) -> &[Reference] {
        &self.facts.references
    }

    #[must_use]
    pub fn exports(&self) -> &[Export] {
        &self.facts.exports
    }

    #[must_use]
    pub fn imports(&self) -> &[Import] {
        &self.facts.imports
    }

    /// Same completed File's original bounded ordinary script family.
    #[must_use]
    pub fn ordinary_empty_script(&self) -> Option<crate::lang::js::OrdinaryEmptyScript<'_>> {
        if !self.is_complete() {
            return None;
        }
        crate::lang::js::file::ordinary::OrdinaryEmptyScript::from_facts(&self.facts)
    }

    pub(crate) fn setup_annotations(
        &self,
    ) -> &[crate::lang::js::file::setup::SetupAnnotationRecord] {
        self.facts
            .setup_annotations
            .as_deref()
            .map_or(&[], |storage| storage.rows())
    }

    /// Borrowed name lookup inside an actual file scope; no context fallback.
    #[must_use]
    pub fn lookup(
        &self,
        scope: ScopeId,
        name: &str,
        namespace: Namespace,
    ) -> Option<BindingRef<'_, 'a>> {
        self.binding(self.facts.lookup(scope, name, namespace)?)
    }

    #[must_use]
    pub fn issues(&self) -> &[FileIssue] {
        &self.facts.issues
    }

    pub fn interrupted_programs(&self) -> impl Iterator<Item = FileIssue> + '_ {
        self.facts.units.iter().filter_map(ScriptUnit::interruption)
    }

    /// Interruption of the actual surrounding template walk, separate from syntax issues.
    #[must_use]
    pub fn template_interruption(&self) -> Option<TemplateIssue> {
        self.facts.template_walk.interruption()
    }

    /// Completeness of the admitted script family, not of Vue/product output.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.facts.issues.is_empty()
            && self.facts.template_issues.is_empty()
            && self.facts.template_walk.is_complete()
            && self.facts.units.iter().all(ScriptUnit::walk_completed)
    }

    #[must_use]
    pub fn template_issues(&self) -> &[TemplateIssue] {
        &self.facts.template_issues
    }

    #[must_use]
    pub fn expression(&self, node: NodeId) -> Option<FileResolution<'_, 'a>> {
        self.facts.expressions.get(node)?;
        Some(FileResolution { file: self, node })
    }

    pub(crate) fn from_producer(artifact: Artifact<'a>, facts: build::Facts<'a>) -> Self {
        Self { artifact, facts }
    }
}

/// A query handle retains the actual file; matching numeric IDs are insufficient.
///
/// ```compile_fail
/// use vize_l2::{file::{BindingRef, FileArtifact}, resolution::BindingId};
/// fn forge(file: &FileArtifact<'_>) {
///     let _ = BindingRef { file, id: BindingId::new(0) };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct BindingRef<'f, 'a> {
    file: &'f FileArtifact<'a>,
    id: BindingId,
}

impl<'f, 'a> BindingRef<'f, 'a> {
    #[must_use]
    pub fn file(self) -> &'f FileArtifact<'a> {
        self.file
    }

    #[must_use]
    pub fn id(self) -> BindingId {
        self.id
    }

    /// Original script declaration only; template aliases retain their whole
    /// original parameter through `template_declaration` instead.
    #[must_use]
    pub fn declaration(self) -> Option<&'f Declaration> {
        self.file.facts.declarations.get(self.id.index() as usize)
    }

    #[must_use]
    pub fn same_owner(self, other: BindingRef<'_, '_>) -> bool {
        core::ptr::eq(self.file, other.file)
    }
}

/// A complete expression entry remains associated with its actual factory node.
///
/// ```compile_fail
/// use vize_l0::id::NodeId;
/// use vize_l2::file::{FileArtifact, FileResolution};
/// fn forge(file: &FileArtifact<'_>) {
///     let _ = FileResolution { file, node: NodeId::FIRST };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct FileResolution<'f, 'a> {
    file: &'f FileArtifact<'a>,
    node: NodeId,
}

impl<'f, 'a> FileResolution<'f, 'a> {
    #[must_use]
    pub fn file(self) -> &'f FileArtifact<'a> {
        self.file
    }

    #[must_use]
    pub fn node(self) -> NodeId {
        self.node
    }

    #[must_use]
    pub fn table(self) -> Option<&'f ResolutionTable<'a>> {
        Some(&self.file.facts.expressions.get(self.node)?.table)
    }

    /// The lexical scope captured by the actual expression factory.
    #[must_use]
    pub fn scope(self) -> Option<ScopeId> {
        Some(self.file.facts.expressions.get(self.node)?.scope)
    }

    #[must_use]
    pub fn binding(self, id: BindingId) -> Option<BindingRef<'f, 'a>> {
        self.file.binding(id)
    }

    #[must_use]
    pub fn accepts(self, binding: BindingRef<'_, '_>) -> bool {
        core::ptr::eq(self.file, binding.file)
    }
}

/// A rejected canonical construction retains its private semantic facts.
pub struct RejectedFile<'a> {
    pub(crate) artifact: crate::artifact::RejectedArtifact<'a>,
    pub(crate) facts: Box<build::Facts<'a>>,
}

impl core::fmt::Debug for RejectedFile<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RejectedFile")
            .field("artifact", &self.artifact)
            .field("issues", &self.facts.issues)
            .field("template_issues", &self.facts.template_issues)
            .finish_non_exhaustive()
    }
}

impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn template_interruption(&self) -> Option<TemplateIssue> {
        self.facts.template_walk.interruption()
    }
    #[must_use]
    pub fn scopes(&self) -> &[Scope] {
        &self.facts.scopes
    }
    #[must_use]
    pub fn artifact(&self) -> &crate::artifact::RejectedArtifact<'a> {
        &self.artifact
    }
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.artifact.parts.source
    }
    #[must_use]
    pub fn units(&self) -> &[ScriptUnit] {
        &self.facts.units
    }
    #[must_use]
    pub fn declarations(&self) -> &[Declaration] {
        &self.facts.declarations
    }
    #[must_use]
    pub fn issues(&self) -> &[FileIssue] {
        &self.facts.issues
    }
    pub fn interrupted_programs(&self) -> impl Iterator<Item = FileIssue> + '_ {
        self.facts.units.iter().filter_map(ScriptUnit::interruption)
    }
    #[must_use]
    pub fn template_issues(&self) -> &[TemplateIssue] {
        &self.facts.template_issues
    }
}
