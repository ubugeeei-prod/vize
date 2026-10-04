//! The genuine File receiver alone supplies an original whole iteration head.

use super::{ArtifactError, Builder, Frame, Owner, check};
use crate::op::{OriginalForId, OriginalForOp};
use crate::resolution::ForResolution;
use vize_l0::{Span, id::NodeId};

impl<'a> Builder<'a> {
    pub(super) fn begin_original_for(
        &mut self,
        head: &ForResolution<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        let operand = head.input().operand();
        let source = operand.syntax().source();
        if !core::ptr::eq(source.authored_root(), self.parts.source) {
            return Err(ArtifactError::MismatchedJsSource {
                node: id,
                span: source.span(),
            });
        }
        check::owned_span(self.parts.source, id, operand.name_span(), Some(span))?;
        check::owned_span(self.parts.source, id, source.span(), Some(span))?;
        self.mint();
        self.frames.push(Frame {
            id,
            owner: Owner::OriginalFor(OriginalForId(id)),
            span,
            bindings: vize_l0::Vec::new_in(&self.allocator),
            children_started: false,
            ops: vize_l0::Vec::new_in(&self.allocator),
        });
        Ok(id)
    }

    pub(super) fn finish_original_for(
        &mut self,
        node: NodeId,
    ) -> Result<core::ptr::NonNull<OriginalForOp<'a>>, ArtifactError> {
        let frame = self
            .frames
            .last()
            .ok_or(ArtifactError::UnfinishedOwner { node })?;
        if frame.id != node || !matches!(&frame.owner, Owner::OriginalFor(id) if id.node() == node)
        {
            return Err(ArtifactError::UnfinishedOwner { node: frame.id });
        }
        if !frame.bindings.is_empty() {
            return Err(ArtifactError::BindingWithoutOwner { span: frame.span });
        }
        let frame = self
            .frames
            .pop()
            .ok_or(ArtifactError::UnfinishedOwner { node })?;
        self.close(frame)
            .original_for
            .ok_or(ArtifactError::UnfinishedOwner { node })
    }
}
