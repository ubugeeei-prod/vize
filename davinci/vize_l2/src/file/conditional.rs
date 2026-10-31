//! Whole original conditional inputs, attached only at their actual If factory.

use super::{FileArtifact, RejectedFile, ScopeId};
use crate::lang::js::{NativeConditionalInput, NativeTemplateIssueKind};
use crate::{op::IfOp, resolution::BindingId};
use vize_l0::id::NodeId;
use vize_l1::markup::NativeAttributeExpressionFailure;

#[derive(Default)]
pub(crate) struct ConditionalStorage<'a> {
    rows: alloc::vec::Vec<NativeFileConditional<'a>>,
    failures: alloc::vec::Vec<NativeAttributeExpressionFailure<'a>>,
}

impl<'a> super::build::Facts<'a> {
    pub(crate) fn native_conditionals(&self) -> &[NativeFileConditional<'a>] {
        self.conditional_storage
            .as_deref()
            .map_or(&[], |storage| storage.rows.as_slice())
    }
    pub(crate) fn native_conditionals_mut(
        &mut self,
    ) -> &mut alloc::vec::Vec<NativeFileConditional<'a>> {
        &mut self
            .conditional_storage
            .get_or_insert_with(|| alloc::boxed::Box::new(ConditionalStorage::default()))
            .rows
    }
    pub(crate) fn native_conditional_failures(&self) -> &[NativeAttributeExpressionFailure<'a>] {
        self.conditional_storage
            .as_deref()
            .map_or(&[], |storage| storage.failures.as_slice())
    }
    pub(crate) fn native_conditional_failures_mut(
        &mut self,
    ) -> &mut alloc::vec::Vec<NativeAttributeExpressionFailure<'a>> {
        &mut self
            .conditional_storage
            .get_or_insert_with(|| alloc::boxed::Box::new(ConditionalStorage::default()))
            .failures
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeFileConditionalState {
    Pending,
    Admitted(NodeId),
    Refused(NativeTemplateIssueKind),
}

pub struct NativeFileConditional<'a> {
    pub(crate) input: NativeConditionalInput<'a>,
    pub(crate) state: NativeFileConditionalState,
    pub(crate) scope: ScopeId,
    pub(crate) binding: Option<BindingId>,
    pub(crate) original: Option<core::ptr::NonNull<IfOp<'a>>>,
}

impl<'a> NativeFileConditional<'a> {
    #[must_use]
    pub const fn input(&self) -> &NativeConditionalInput<'a> {
        &self.input
    }
    #[must_use]
    pub const fn state(&self) -> NativeFileConditionalState {
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
    pub fn accepts(&self, original: &IfOp<'a>) -> bool {
        matches!(self.state, NativeFileConditionalState::Admitted(_))
            && self
                .original
                .is_some_and(|pointer| core::ptr::eq(pointer.as_ptr(), original))
    }
}

impl<'a> FileArtifact<'a> {
    #[must_use]
    pub fn native_conditionals(&self) -> &[NativeFileConditional<'a>] {
        self.facts.native_conditionals()
    }
    #[must_use]
    pub fn native_conditional_failures(&self) -> &[NativeAttributeExpressionFailure<'a>] {
        self.facts.native_conditional_failures()
    }
    #[must_use]
    pub fn native_conditional(&self, node: NodeId) -> Option<&NativeFileConditional<'a>> {
        if !self.is_complete() {
            return None;
        }
        let [record] = self.facts.native_conditionals() else {
            return None;
        };
        matches!(record.state(), NativeFileConditionalState::Admitted(actual) if actual == node)
            .then_some(record)
    }
}

impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn native_conditionals(&self) -> &[NativeFileConditional<'a>] {
        self.facts.native_conditionals()
    }
    #[must_use]
    pub fn native_conditional_failures(&self) -> &[NativeAttributeExpressionFailure<'a>] {
        self.facts.native_conditional_failures()
    }
}
