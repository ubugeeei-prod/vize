//! Only the File's genuine original handler receiver calls this attached mint.

use super::super::{ArtifactError, Builder, check};
use crate::op::{BindingOp, DynamicName, HandlerId, OnHandlerRef, OnOp};
use crate::resolution::HandlerResolution;
use vize_l0::{Box, Span, id::NodeId};

impl<'a> Builder<'a> {
    pub(in crate::artifact::builder) fn native_on(
        &mut self,
        handler: &HandlerResolution<'a>,
        span: Span,
    ) -> Result<(NodeId, core::ptr::NonNull<OnOp<'a>>), ArtifactError> {
        let frame = self
            .frames
            .last()
            .ok_or(ArtifactError::BindingWithoutOwner { span })?;
        if matches!(&frame.owner, super::super::Owner::OriginalFor(_)) {
            return Err(ArtifactError::BindingWithoutOwner { span });
        }
        if frame.children_started {
            return Err(ArtifactError::BindingAfterChild {
                node: frame.id,
                span,
            });
        }
        let id = self.prepare(span)?;
        let input = handler.input().operand();
        let source = input.syntax().source();
        if !core::ptr::eq(source.authored_root(), self.parts.source) {
            return Err(ArtifactError::MismatchedJsSource {
                node: id,
                span: source.span(),
            });
        }
        check::owned_span(self.parts.source, id, input.argument_span(), Some(span))?;
        check::owned_span(self.parts.source, id, source.span(), Some(span))?;
        let name = input.argument();
        if name.is_empty() || input.argument_span().slice(self.parts.source) != name {
            return Err(ArtifactError::InvalidBindingName {
                node: id,
                span: input.argument_span(),
            });
        }
        let frame = self
            .frames
            .last_mut()
            .ok_or(ArtifactError::BindingWithoutOwner { span })?;
        let _ = self.walk.mint_attached();
        let on = Box::new_in(
            OnOp {
                name: Some(DynamicName::Static(name)),
                modifiers: vize_l0::Vec::new_in(&self.allocator),
                handler: Some(OnHandlerRef::Body(HandlerId(id))),
                span,
            },
            &self.allocator,
        );
        // This stable arena address is compared only, never dereferenced by
        // the File table. Numeric IDs alone cannot prove original-On custody.
        let original = core::ptr::NonNull::from(on.as_ref());
        frame.bindings.push(BindingOp::On(on));
        Ok((id, original))
    }
}
