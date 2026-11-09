//! Only the whole authenticated conditional input may introduce a native If.

use super::{ArtifactError, Builder, Frame, Owner, check};
use crate::lang::js::NativeConditionalInput;
use crate::op::{IfOp, Op, Region};
use vize_l0::{Span, id::NodeId};
use vize_l1::markup::NativeAttributeElse;

impl<'a> Builder<'a> {
    pub(super) fn begin_native_if(
        &mut self,
        input: &NativeConditionalInput<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        let operand = input.operand();
        let expression = input.expression().ok_or(ArtifactError::InvalidSpan {
            node: Some(id),
            span: operand.value_span(),
        })?;
        if !core::ptr::eq(operand.syntax().source().authored_root(), self.parts.source)
            || !input.matches_expression(expression)
        {
            return Err(ArtifactError::MismatchedJsSource {
                node: id,
                span: expression.span,
            });
        }
        check::owned_span(self.parts.source, id, operand.name_span(), Some(span))?;
        check::owned_span(self.parts.source, id, operand.value_span(), Some(span))?;
        check::expression(
            self.parts.source,
            id,
            crate::expr::ExprRef::Js(expression),
            span,
        )?;
        self.mint();
        self.frames.push(Frame {
            id,
            owner: Owner::NativeIf(expression),
            span,
            bindings: vize_l0::Vec::new_in(&self.allocator),
            children_started: false,
            ops: vize_l0::Vec::new_in(&self.allocator),
        });
        Ok(id)
    }

    /// Transition only the actual completed first Element to its real Else.
    /// The valueless original owner supplies the second and whole-chain extent.
    pub(super) fn begin_native_else(
        &mut self,
        node: NodeId,
        input: &NativeAttributeElse<'a>,
    ) -> Result<(), ArtifactError> {
        let frame = self
            .frames
            .last_mut()
            .ok_or(ArtifactError::UnfinishedOwner { node })?;
        let condition = match &frame.owner {
            Owner::NativeIf(condition) => *condition,
            _ => return Err(ArtifactError::UnfinishedOwner { node: frame.id }),
        };
        let else_span = input.element_span();
        if frame.id != node
            || !frame.bindings.is_empty()
            || !matches!(frame.ops.as_slice(), [Op::Element(_)])
            || frame.span.end != else_span.start
            || !core::ptr::eq(input.source_block().root_source(), self.parts.source)
        {
            return Err(ArtifactError::UnfinishedOwner { node: frame.id });
        }
        check::owned_span(self.parts.source, node, else_span, None)?;
        check::owned_span(self.parts.source, node, input.name_span(), Some(else_span))?;
        let first = Region {
            ops: core::mem::replace(&mut frame.ops, vize_l0::Vec::new_in(&self.allocator)),
        };
        let first_span = frame.span;
        frame.span = Span::new(first_span.start, else_span.end);
        frame.owner = Owner::NativeIfElse {
            condition,
            first,
            first_span,
            else_span,
        };
        Ok(())
    }

    pub(super) fn finish_native_if(
        &mut self,
        node: NodeId,
    ) -> Result<core::ptr::NonNull<IfOp<'a>>, ArtifactError> {
        let frame = self
            .frames
            .last()
            .ok_or(ArtifactError::UnfinishedOwner { node })?;
        if frame.id != node
            || !matches!(
                &frame.owner,
                Owner::NativeIf(_) | Owner::NativeIfElse { .. }
            )
            || (matches!(&frame.owner, Owner::NativeIfElse { .. })
                && !matches!(frame.ops.as_slice(), [Op::Element(_)]))
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
            .native_if
            .ok_or(ArtifactError::UnfinishedOwner { node })
    }
}
