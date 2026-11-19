//! Original Model inputs, sealed only by their normally closed Element factory.

use super::{FileArtifact, RejectedFile, ScopeId};
use crate::lang::js::{ModelInput, TemplateRouteIssueKind};
use crate::op::{ElementOp, ModelOp};
use crate::resolution::BindingId;
use vize_l0::id::NodeId;
use vize_l1::markup::AttributeExpressionFailure;

#[derive(Default)]
pub(crate) struct ModelStorage<'a> {
    rows: alloc::vec::Vec<FileModel<'a>>,
    failures: alloc::vec::Vec<AttributeExpressionFailure<'a>>,
}

impl<'a> super::build::Facts<'a> {
    pub(crate) fn file_models(&self) -> &[FileModel<'a>] {
        self.model_storage
            .as_deref()
            .map_or(&[], |storage| storage.rows.as_slice())
    }
    pub(crate) fn file_models_mut(&mut self) -> &mut alloc::vec::Vec<FileModel<'a>> {
        &mut self
            .model_storage
            .get_or_insert_with(|| alloc::boxed::Box::new(ModelStorage::default()))
            .rows
    }
    pub(crate) fn file_model_failures(&self) -> &[AttributeExpressionFailure<'a>] {
        self.model_storage
            .as_deref()
            .map_or(&[], |storage| storage.failures.as_slice())
    }
    pub(crate) fn file_model_failures_mut(
        &mut self,
    ) -> &mut alloc::vec::Vec<AttributeExpressionFailure<'a>> {
        &mut self
            .model_storage
            .get_or_insert_with(|| alloc::boxed::Box::new(ModelStorage::default()))
            .failures
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileModelState {
    Pending,
    Admitted(NodeId),
    Refused(TemplateRouteIssueKind),
}

/// Input behavior established by the normally closed original Element factory.
/// It grants neither mutation nor generic expression-read permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileModelKind {
    Text,
    Checkbox,
}

/// Original observation and its real attached Model/Element identity.
/// No caller-owned row, numeric ID or raw span can construct this record.
pub struct FileModel<'a> {
    pub(crate) input: ModelInput<'a>,
    pub(crate) state: FileModelState,
    pub(crate) scope: ScopeId,
    pub(crate) binding: Option<BindingId>,
    pub(crate) original: Option<core::ptr::NonNull<ModelOp<'a>>>,
    pub(crate) element: Option<(NodeId, core::ptr::NonNull<ElementOp<'a>>)>,
    pub(crate) kind: Option<FileModelKind>,
}

impl<'a> FileModel<'a> {
    #[must_use]
    pub const fn input(&self) -> &ModelInput<'a> {
        &self.input
    }
    #[must_use]
    pub const fn state(&self) -> FileModelState {
        self.state
    }
    #[must_use]
    pub const fn kind(&self) -> Option<FileModelKind> {
        self.kind
    }
    #[must_use]
    pub const fn scope(&self) -> ScopeId {
        self.scope
    }
    #[must_use]
    pub const fn binding(&self) -> Option<BindingId> {
        self.binding
    }
    #[must_use]
    pub fn element_node(&self) -> Option<NodeId> {
        self.element.map(|(node, _)| node)
    }
    #[must_use]
    pub fn accepts(&self, original: &ModelOp<'a>) -> bool {
        matches!(self.state, FileModelState::Admitted(_))
            && self
                .original
                .is_some_and(|pointer| core::ptr::eq(pointer.as_ptr(), original))
    }
    #[must_use]
    pub fn accepts_element(&self, node: NodeId, original: &ElementOp<'a>) -> bool {
        matches!(self.state, FileModelState::Admitted(_))
            && self.element.is_some_and(|(actual, pointer)| {
                actual == node && core::ptr::eq(pointer.as_ptr(), original)
            })
    }
}

impl<'a> FileArtifact<'a> {
    #[must_use]
    pub fn file_models(&self) -> &[FileModel<'a>] {
        self.facts.file_models()
    }
    #[must_use]
    pub fn file_model_failures(&self) -> &[AttributeExpressionFailure<'a>] {
        self.facts.file_model_failures()
    }
    #[must_use]
    pub fn file_model(&self, node: NodeId) -> Option<&FileModel<'a>> {
        if !self.is_complete() {
            return None;
        }
        let [record] = self.facts.file_models() else {
            return None;
        };
        matches!(record.state(), FileModelState::Admitted(actual) if actual == node)
            .then_some(record)
    }
}

impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn file_models(&self) -> &[FileModel<'a>] {
        self.facts.file_models()
    }
    #[must_use]
    pub fn file_model_failures(&self) -> &[AttributeExpressionFailure<'a>] {
        self.facts.file_model_failures()
    }
}
