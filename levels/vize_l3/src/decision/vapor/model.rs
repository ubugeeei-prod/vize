//! Dedicated mutable custody from the actual original Model binding event.

use super::VaporUnsupported;
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    file::{BindingRef, FileResolution, NativeFileModel},
    lang::js::NativeSelectedSetup,
    op::{BindingOp, ElementOp, ModelOp},
    resolution::Occurrence,
};

/// No caller-supplied ID, span, expression or binding can mint this role.
/// Its compact carrier reuses the original File's immutable resolution table.
#[derive(Clone, Copy)]
pub struct VaporModel<'owner, 'arena> {
    resolution: FileResolution<'owner, 'arena>,
}

pub(super) fn classify<'owner, 'arena>(
    setup: &NativeSelectedSetup<'owner, 'arena>,
    element_node: NodeId,
    element: &'owner ElementOp<'arena>,
    node: NodeId,
    binding: &'owner BindingOp<'arena>,
    model: &'owner ModelOp<'arena>,
) -> Result<VaporModel<'owner, 'arena>, VaporUnsupported> {
    let resolution =
        crate::decision::model::checked(setup.file(), element_node, element, node, binding, model)
            .ok_or(VaporUnsupported::ModelOrigin)?;
    let [occurrence] = resolution
        .table()
        .ok_or(VaporUnsupported::ModelOrigin)?
        .occurrences()
    else {
        return Err(VaporUnsupported::ModelOrigin);
    };
    let binding = resolution
        .binding(occurrence.binding)
        .ok_or(VaporUnsupported::ModelOrigin)?;
    setup
        .binding(binding)
        .map_err(|_| VaporUnsupported::ModelOrigin)?;
    Ok(VaporModel { resolution })
}

impl core::fmt::Debug for VaporModel<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("VaporModel")
            .field("node", &self.resolution.node())
            .field("scope", &self.resolution.scope())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> VaporModel<'owner, 'arena> {
    #[must_use]
    pub const fn resolution(&self) -> FileResolution<'owner, 'arena> {
        self.resolution
    }

    #[must_use]
    pub fn original(&self) -> Option<&'owner NativeFileModel<'arena>> {
        self.resolution.file().native_model(self.resolution.node())
    }

    #[must_use]
    pub fn occurrence(&self) -> Option<&'owner Occurrence<'arena>> {
        let [occurrence] = self.resolution.table()?.occurrences() else {
            return None;
        };
        Some(occurrence)
    }

    #[must_use]
    pub fn binding(&self) -> Option<BindingRef<'owner, 'arena>> {
        self.resolution.binding(self.occurrence()?.binding)
    }

    #[must_use]
    pub fn authored_span(&self) -> Option<Span> {
        self.resolution
            .table()?
            .expression()
            .authored_span(self.occurrence()?.span)
    }
}
