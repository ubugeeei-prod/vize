//! Only a retained original Model can introduce this attached contract.

use super::super::{ArtifactError, Builder, Owner, check};
use crate::expr::ExprRef;
use crate::lang::js::NativeModelInput;
use crate::op::{Attribute, BindingContract, BindingOp, ModelOp, Namespace};
use vize_l0::{Box, id::NodeId};

impl<'a> Builder<'a> {
    pub(in crate::artifact::builder) fn native_model(
        &mut self,
        input: &NativeModelInput<'a>,
    ) -> Result<(NodeId, core::ptr::NonNull<ModelOp<'a>>), ArtifactError> {
        let span = input.attribute_span().ok_or(ArtifactError::InvalidSpan {
            node: None,
            span: input.operand().name_span(),
        })?;
        let expression = input.expression().ok_or(ArtifactError::InvalidSpan {
            node: None,
            span: input.operand().value_span(),
        })?;
        let frame = self
            .frames
            .last()
            .ok_or(ArtifactError::BindingWithoutOwner { span })?;
        if !matches!(&frame.owner, Owner::Element { tag: "input", namespace: Namespace::Html, attributes } if attributes.is_empty())
            || !frame.bindings.is_empty()
        {
            return Err(ArtifactError::BindingWithoutOwner { span });
        }
        if frame.children_started {
            return Err(ArtifactError::BindingAfterChild {
                node: frame.id,
                span,
            });
        }
        let id = self.prepare(span)?;
        if !input.matches_expression(expression)
            || !core::ptr::eq(
                input.operand().syntax().source().authored_root(),
                self.parts.source,
            )
        {
            return Err(ArtifactError::MismatchedJsSource {
                node: id,
                span: expression.span,
            });
        }
        check::owned_span(
            self.parts.source,
            id,
            input.operand().name_span(),
            Some(span),
        )?;
        check::owned_span(
            self.parts.source,
            id,
            input.operand().value_span(),
            Some(span),
        )?;
        check::expression(self.parts.source, id, ExprRef::Js(expression), span)?;
        let mut attributes = vize_l0::Vec::new_in(&self.allocator);
        if let Some(modifier) = input
            .operand()
            .trim_span()
            .or(input.operand().lazy_span())
            .or(input.operand().number_span())
        {
            check::owned_span(self.parts.source, id, modifier, Some(span))?;
            attributes.push(Attribute {
                name: modifier.slice(self.parts.source),
                value: None,
                span: modifier,
            });
        }
        let model = Box::new_in(
            ModelOp {
                contract: BindingContract {
                    read: ExprRef::Js(expression),
                    write: ExprRef::Js(expression),
                },
                argument: None,
                attributes,
                span,
            },
            &self.allocator,
        );
        let original = core::ptr::NonNull::from(model.as_ref());
        let frame = self
            .frames
            .last_mut()
            .ok_or(ArtifactError::BindingWithoutOwner { span })?;
        let _ = self.walk.mint_attached();
        frame.bindings.push(BindingOp::Model(model));
        Ok((id, original))
    }
}
