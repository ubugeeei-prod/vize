//! A separate original-Model read/write receipt, never a generic read policy.

use super::{DomFacts, PropertyRole, ValueKind, file::matches_expression};
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    expr::ExprRef,
    file::{BindingRef, FileResolution, NativeFileModel, NativeFileModelState},
    op::{BindingOp, ElementOp, ModelOp},
    resolution::{Occurrence, Usage},
};

/// The actual completed File, Model and mutable setup target stay borrowed.
/// Its checked role comes from the sole canonical Model binding visit. The
/// borrowed view is derived on demand, without a second stored fact table.
pub struct DomFileModel<'owner, 'arena> {
    original: &'owner NativeFileModel<'arena>,
    model: &'owner ModelOp<'arena>,
    resolution: FileResolution<'owner, 'arena>,
    binding: BindingRef<'owner, 'arena>,
    occurrence: &'owner Occurrence<'arena>,
    authored: Span,
}

pub(super) fn get<'owner, 'arena>(
    facts: &DomFacts<'owner, 'arena>,
    node: NodeId,
) -> Option<DomFileModel<'owner, 'arena>> {
    let fact = facts.binding(node)?;
    if fact.role != PropertyRole::Model || fact.value != ValueKind::FileDependent {
        return None;
    }
    let BindingOp::Model(model) = fact.binding() else {
        return None;
    };
    let resolution = facts.file_expression(node)?.resolution();
    let original = resolution.file().native_model(node)?;
    let (ExprRef::Js(read), ExprRef::Js(write)) = (model.contract.read, model.contract.write)
    else {
        return None;
    };
    if resolution.node() != node
        || resolution.scope() != Some(original.scope())
        || original.state() != NativeFileModelState::Admitted(node)
        || !original.accepts(model)
        || original.input().attribute_span() != Some(model.span)
        || !core::ptr::eq(read, write)
        || !original.input().matches_expression(read)
        || !matches_expression(resolution, read)
    {
        return None;
    }
    let [occurrence] = resolution.table()?.occurrences() else {
        return None;
    };
    let binding = resolution.binding(occurrence.binding)?;
    if occurrence.usage != Usage::ReadWrite
        || !resolution.accepts(binding)
        || original.binding() != Some(binding.id())
    {
        return None;
    }
    let authored = read.authored_span(occurrence.span)?;
    Some(DomFileModel {
        original,
        model,
        resolution,
        binding,
        occurrence,
        authored,
    })
}

impl core::fmt::Debug for DomFileModel<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DomFileModel")
            .field("node", &self.resolution.node())
            .field("scope", &self.resolution.scope())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> DomFileModel<'owner, 'arena> {
    #[must_use]
    pub const fn original(&self) -> &'owner NativeFileModel<'arena> {
        self.original
    }
    #[must_use]
    pub const fn model(&self) -> &'owner ModelOp<'arena> {
        self.model
    }
    #[must_use]
    pub const fn resolution(&self) -> FileResolution<'owner, 'arena> {
        self.resolution
    }
    #[must_use]
    pub const fn binding(&self) -> BindingRef<'owner, 'arena> {
        self.binding
    }
    /// One genuine source Identifier carries the Model's read/write use.
    #[must_use]
    pub const fn occurrence(&self) -> &'owner Occurrence<'arena> {
        self.occurrence
    }
    #[must_use]
    pub const fn authored_span(&self) -> Span {
        self.authored
    }
    #[must_use]
    pub fn accepts_model(&self, model: &ModelOp<'arena>) -> bool {
        core::ptr::eq(self.model, model) && self.original.accepts(model)
    }
    #[must_use]
    pub fn accepts_element(&self, node: vize_l0::id::NodeId, element: &ElementOp<'arena>) -> bool {
        self.original.accepts_element(node, element)
    }
}
