//! Original Model inputs, sealed only by their normally closed Element factory.

use super::{FileArtifact, RejectedFile, ScopeId};
use crate::lang::js::{NativeModelInput, NativeTemplateIssueKind};
use crate::op::{ElementOp, ModelOp};
use crate::resolution::BindingId;
use vize_l0::id::NodeId;
use vize_l1::markup::NativeAttributeExpressionFailure;

#[derive(Default)]
pub(crate) struct ModelStorage<'a> {
    rows: alloc::vec::Vec<NativeFileModel<'a>>,
    failures: alloc::vec::Vec<NativeAttributeExpressionFailure<'a>>,
}

impl<'a> super::build::Facts<'a> {
    pub(crate) fn native_models(&self) -> &[NativeFileModel<'a>] {
        self.model_storage
            .as_deref()
            .map_or(&[], |storage| storage.rows.as_slice())
    }
    pub(crate) fn native_models_mut(&mut self) -> &mut alloc::vec::Vec<NativeFileModel<'a>> {
        &mut self
            .model_storage
            .get_or_insert_with(|| alloc::boxed::Box::new(ModelStorage::default()))
            .rows
    }
    pub(crate) fn native_model_failures(&self) -> &[NativeAttributeExpressionFailure<'a>] {
        self.model_storage
            .as_deref()
            .map_or(&[], |storage| storage.failures.as_slice())
    }
    pub(crate) fn native_model_failures_mut(
        &mut self,
    ) -> &mut alloc::vec::Vec<NativeAttributeExpressionFailure<'a>> {
        &mut self
            .model_storage
            .get_or_insert_with(|| alloc::boxed::Box::new(ModelStorage::default()))
            .failures
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeFileModelState {
    Pending,
    Admitted(NodeId),
    Refused(NativeTemplateIssueKind),
}

/// Original observation and its real attached Model/Element identity.
/// No caller-owned row, numeric ID or raw span can construct this record.
pub struct NativeFileModel<'a> {
    pub(crate) input: NativeModelInput<'a>,
    pub(crate) state: NativeFileModelState,
    pub(crate) scope: ScopeId,
    pub(crate) binding: Option<BindingId>,
    pub(crate) original: Option<core::ptr::NonNull<ModelOp<'a>>>,
    pub(crate) element: Option<(NodeId, core::ptr::NonNull<ElementOp<'a>>)>,
}

impl<'a> NativeFileModel<'a> {
    #[must_use]
    pub const fn input(&self) -> &NativeModelInput<'a> {
        &self.input
    }
    #[must_use]
    pub const fn state(&self) -> NativeFileModelState {
        self.state
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
        matches!(self.state, NativeFileModelState::Admitted(_))
            && self
                .original
                .is_some_and(|pointer| core::ptr::eq(pointer.as_ptr(), original))
    }
    #[must_use]
    pub fn accepts_element(&self, node: NodeId, original: &ElementOp<'a>) -> bool {
        matches!(self.state, NativeFileModelState::Admitted(_))
            && self.element.is_some_and(|(actual, pointer)| {
                actual == node && core::ptr::eq(pointer.as_ptr(), original)
            })
    }
}

impl<'a> FileArtifact<'a> {
    #[must_use]
    pub fn native_models(&self) -> &[NativeFileModel<'a>] {
        self.facts.native_models()
    }
    #[must_use]
    pub fn native_model_failures(&self) -> &[NativeAttributeExpressionFailure<'a>] {
        self.facts.native_model_failures()
    }
    #[must_use]
    pub fn native_model(&self, node: NodeId) -> Option<&NativeFileModel<'a>> {
        if !self.is_complete() {
            return None;
        }
        let [record] = self.facts.native_models() else {
            return None;
        };
        matches!(record.state(), NativeFileModelState::Admitted(actual) if actual == node)
            .then_some(record)
    }
}

impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn native_models(&self) -> &[NativeFileModel<'a>] {
        self.facts.native_models()
    }
    #[must_use]
    pub fn native_model_failures(&self) -> &[NativeAttributeExpressionFailure<'a>] {
        self.facts.native_model_failures()
    }
}
