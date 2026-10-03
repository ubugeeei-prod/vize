//! Attached factories check phase and ownership in the construction walk.

use super::{ArtifactError, Builder, check};
use crate::expr::ExprRef;
use crate::op::{BindOp, BindingOp, DynamicName};
use vize_l0::{Box, Span, id::NodeId};

mod native;

impl<'a> Builder<'a> {
    pub(super) fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        value: ExprRef<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        let frame = self
            .frames
            .last()
            .ok_or(ArtifactError::BindingWithoutOwner { span })?;
        if matches!(&frame.owner, super::Owner::OriginalFor(_)) {
            return Err(ArtifactError::BindingWithoutOwner { span });
        }
        if frame.children_started {
            return Err(ArtifactError::BindingAfterChild {
                node: frame.id,
                span,
            });
        }
        let id = self.prepare(span)?;
        check::owned_span(self.parts.source, id, name_span, Some(span))?;
        if name.is_empty()
            || self
                .parts
                .source
                .get(name_span.start as usize..name_span.end as usize)
                != Some(name)
        {
            return Err(ArtifactError::InvalidBindingName {
                node: id,
                span: name_span,
            });
        }
        check::expression(self.parts.source, id, value, span)?;
        // The phase and exact next index are checked above. This attached mint
        // leaves the child-region phase unchanged; no traversal is involved.
        let frame = self
            .frames
            .last_mut()
            .ok_or(ArtifactError::BindingWithoutOwner { span })?;
        let _ = self.walk.mint_attached();
        frame.bindings.push(BindingOp::Bind(Box::new_in(
            BindOp {
                name: Some(DynamicName::Static(name)),
                modifiers: vize_l0::Vec::new_in(&self.allocator),
                value: Some(value),
                span,
            },
            &self.allocator,
        )));
        Ok(id)
    }
}

#[cfg(test)]
mod tests;
