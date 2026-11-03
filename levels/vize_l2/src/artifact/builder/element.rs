//! Transient actual Element allocation from the same normal frame close.

use super::{ArtifactError, Builder, Owner, RegionBuilder};
use crate::op::{Attribute, ElementOp, Namespace};
use core::ptr::NonNull;
use vize_l0::{Span, id::NodeId};

pub(crate) struct ElementAllocation<'a> {
    original: NonNull<ElementOp<'a>>,
    attributes: NonNull<[Attribute<'a>]>,
}
impl<'a> ElementAllocation<'a> {
    pub(super) fn original(element: &ElementOp<'a>) -> Self {
        Self {
            original: NonNull::from(element),
            attributes: NonNull::from(element.attributes.as_slice()),
        }
    }
    pub(crate) fn matches_storage(&self, storage: NonNull<[Attribute<'a>]>) -> bool {
        core::ptr::eq(self.attributes.as_ptr(), storage.as_ptr())
    }
    pub(crate) fn pointer(&self) -> NonNull<ElementOp<'a>> {
        self.original
    }
}
impl<'a> Builder<'a> {
    pub(super) fn native_element(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<(NodeId, ElementAllocation<'a>), ArtifactError> {
        let id = self.prepare_owner(&attributes, span)?;
        let closed = self.children(
            Owner::Element {
                tag,
                namespace,
                attributes,
            },
            span,
            id,
            children,
        )?;
        let element = closed
            .element
            .ok_or(ArtifactError::UnfinishedOwner { node: id })?;
        Ok((id, element))
    }
}
