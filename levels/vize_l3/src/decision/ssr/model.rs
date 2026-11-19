//! A dedicated SSR value view of the actual original mutable Model target.

use super::{NativeSelectedSetupSsrAnalysis, SsrSetupAccess, SsrSetupExpression, SsrUnsupported};
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    file::{BindingRef, FileResolution, NativeFileModel, NativeFileModelState},
    lang::js::NativeSelectedSetup,
    op::{BindingOp, ElementOp, ModelOp},
    resolution::Occurrence,
};

/// The original mutable source is retained, but SSR receives no update writer.
/// Construction is private and the same current File must outlive this view.
pub struct SsrFileModel<'owner, 'arena> {
    original: &'owner NativeFileModel<'arena>,
    model: &'owner ModelOp<'arena>,
    resolution: FileResolution<'owner, 'arena>,
    binding: BindingRef<'owner, 'arena>,
    occurrence: &'owner Occurrence<'arena>,
    authored: Span,
}

pub(super) fn classify<'owner, 'arena>(
    setup: &NativeSelectedSetup<'owner, 'arena>,
    element_node: NodeId,
    element: &'owner ElementOp<'arena>,
    node: NodeId,
    binding: &'owner BindingOp<'arena>,
    model: &'owner ModelOp<'arena>,
) -> Result<SsrSetupExpression<'owner, 'arena>, SsrUnsupported> {
    let resolution =
        crate::decision::model::checked(setup.file(), element_node, element, node, binding, model)
            .ok_or(SsrUnsupported::FileBinding)?;
    let [occurrence] = resolution
        .table()
        .ok_or(SsrUnsupported::FileExpression)?
        .occurrences()
    else {
        return Err(SsrUnsupported::FileExpression);
    };
    let target = resolution
        .binding(occurrence.binding)
        .ok_or(SsrUnsupported::FileBinding)?;
    setup
        .binding(target)
        .map_err(|_| SsrUnsupported::SetupReadAccess)?;
    Ok(SsrSetupExpression {
        resolution,
        access: SsrSetupAccess::ModelValue,
    })
}

impl<'owner, 'arena> NativeSelectedSetupSsrAnalysis<'_, 'owner, 'arena> {
    /// Derive a Model only from its canonical role and original opening owner.
    /// A missing/mismatched Model is an error, never a static-output fallback.
    pub fn model(
        &self,
        element_node: NodeId,
        element: &'owner ElementOp<'arena>,
    ) -> Result<Option<SsrFileModel<'owner, 'arena>>, SsrUnsupported> {
        let file = self.file();
        let original = match file.native_models() {
            [] => return Ok(None),
            [original] => original,
            _ => return Err(SsrUnsupported::FileBinding),
        };
        let NativeFileModelState::Admitted(node) = original.state() else {
            return Err(SsrUnsupported::FileBinding);
        };
        let facts = self.ssr().ok_or(SsrUnsupported::FileExpression)?;
        if !facts.unsupported().is_empty()
            || facts.inherit_attrs() != Some(element_node)
            || facts.fragment()
        {
            return Err(SsrUnsupported::Binding);
        }
        let row = facts
            .expression(node)
            .filter(|row| row.is_model())
            .ok_or(SsrUnsupported::FileExpression)?;
        let [binding @ BindingOp::Model(model)] = element.bindings.as_slice() else {
            return Err(SsrUnsupported::FileBinding);
        };
        let checked = classify(self.setup(), element_node, element, node, binding, model)?;
        let resolution = row.resolution();
        if !core::ptr::eq(resolution.file(), checked.resolution().file())
            || resolution.node() != checked.resolution().node()
            || resolution.scope() != checked.resolution().scope()
            || !self
                .tables()
                .nodes
                .get(element_node)
                .is_some_and(|row| row.dynamic_bindings.as_slice() == [node])
        {
            return Err(SsrUnsupported::FileBinding);
        }
        let [occurrence] = resolution
            .table()
            .ok_or(SsrUnsupported::FileExpression)?
            .occurrences()
        else {
            return Err(SsrUnsupported::FileExpression);
        };
        let binding = resolution
            .binding(occurrence.binding)
            .ok_or(SsrUnsupported::FileBinding)?;
        let expression = resolution
            .table()
            .ok_or(SsrUnsupported::FileExpression)?
            .expression();
        let authored = expression
            .authored_span(occurrence.span)
            .ok_or(SsrUnsupported::FileExpression)?;
        Ok(Some(SsrFileModel {
            original,
            model,
            resolution,
            binding,
            occurrence,
            authored,
        }))
    }
}

impl core::fmt::Debug for SsrFileModel<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SsrFileModel")
            .field("node", &self.resolution.node())
            .field("scope", &self.resolution.scope())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> SsrFileModel<'owner, 'arena> {
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
    #[must_use]
    pub const fn occurrence(&self) -> &'owner Occurrence<'arena> {
        self.occurrence
    }
    #[must_use]
    pub const fn authored_span(&self) -> Span {
        self.authored
    }
}
