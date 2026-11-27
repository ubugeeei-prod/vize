//! Attached factories check phase and ownership in the construction walk.

use super::{Builder, L2TemplateError, check};
use crate::expr::ExprRef;
use crate::op::{BindOp, BindingOp, DynamicName};
use vize_l0::{Box, Span, id::NodeId};

mod model;
mod native;

impl<'a> Builder<'a> {
    pub(super) fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        value: ExprRef<'a>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError> {
        let id = self.prepare_binding(span)?;
        check::owned_span(self.parts.source, id, name_span, Some(span))?;
        if name.is_empty()
            || self
                .parts
                .source
                .get(name_span.start as usize..name_span.end as usize)
                != Some(name)
        {
            return Err(L2TemplateError::InvalidBindingName {
                node: id,
                span: name_span,
            });
        }
        self.attach(id, Some(DynamicName::Static(name)), value, span)
    }

    /// Attach an object spread (`{...value}`), ordered by span among bindings.
    pub(super) fn spread(
        &mut self,
        value: ExprRef<'a>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError> {
        let id = self.prepare_binding(span)?;
        self.attach(id, None, value, span)
    }

    /// A binding whose name the lowering supplies: `span` is the authored
    /// value's source (TSRX `<{tag}>` binds its tag as `is`), not the name.
    pub(super) fn synthesized_bind(
        &mut self,
        name: &'a str,
        value: ExprRef<'a>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError> {
        let id = self.prepare_binding(span)?;
        self.attach(id, Some(DynamicName::Static(name)), value, span)
    }

    fn prepare_binding(&mut self, span: Span) -> Result<NodeId, L2TemplateError> {
        let frame = self
            .frames
            .last()
            .ok_or(L2TemplateError::BindingWithoutOwner { span })?;
        if !frame.owner.accepts_bindings() {
            return Err(L2TemplateError::BindingWithoutOwner { span });
        }
        if frame.children_started {
            return Err(L2TemplateError::BindingAfterChild {
                node: frame.id,
                span,
            });
        }
        self.prepare(span)
    }

    fn attach(
        &mut self,
        id: NodeId,
        name: Option<DynamicName<'a>>,
        value: ExprRef<'a>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError> {
        check::expression(self.parts.source, id, value, span)?;
        // The phase and exact next index are checked above. This attached mint
        // leaves the child-region phase unchanged; no traversal is involved.
        let frame = self
            .frames
            .last_mut()
            .ok_or(L2TemplateError::BindingWithoutOwner { span })?;
        let _ = self.walk.mint_attached();
        frame.bindings.push(BindingOp::Bind(Box::new_in(
            BindOp {
                name,
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

#[cfg(feature = "component-listener-queries")]
mod listener;
