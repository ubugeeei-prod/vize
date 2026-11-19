//! A conditional whose branches arrive in authored order, checked as they open.

use super::{ArtifactError, Builder, Frame, Owner, RegionBuilder, check};
use crate::artifact::IfShape;
use crate::expr::ExprRef;
use crate::op::{IfBranch, Region};
use vize_l0::{Span, id::NodeId};

/// The branch scope of one `ui.if`; it can only add branches to that owner.
pub struct BranchBuilder<'s, 'a> {
    builder: &'s mut Builder<'a>,
    node: NodeId,
}

impl<'a> BranchBuilder<'_, 'a> {
    /// Add the next branch. Only the last branch may omit its condition.
    pub fn branch(
        &mut self,
        condition: Option<ExprRef<'a>>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<(), ArtifactError> {
        self.builder.branch(self.node, condition, span, children)
    }
}

impl<'a> Builder<'a> {
    pub(super) fn conditional(
        &mut self,
        span: Span,
        branches: impl FnOnce(&mut BranchBuilder<'_, 'a>, NodeId),
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        self.mint();
        self.frames.push(Frame {
            id,
            owner: Owner::Conditional {
                branches: vize_l0::Vec::new_in(&self.allocator),
            },
            span,
            bindings: vize_l0::Vec::new_in(&self.allocator),
            children_started: false,
            ops: vize_l0::Vec::new_in(&self.allocator),
        });
        let depth = self.frames.len();
        branches(
            &mut BranchBuilder {
                builder: self,
                node: id,
            },
            id,
        );
        if self.frames.len() != depth {
            return Err(ArtifactError::UnfinishedOwner { node: id });
        }
        match self.frames.last() {
            Some(Frame {
                owner: Owner::Conditional { branches },
                ..
            }) if branches.is_empty() => {
                // Keep the owner pending: finish retains it as unfinished
                // instead of sealing a branchless conditional.
                return Err(ArtifactError::InvalidIf {
                    node: id,
                    shape: IfShape::Empty,
                });
            }
            Some(frame) if frame.id == id => {}
            _ => return Err(ArtifactError::UnfinishedOwner { node: id }),
        }
        let frame = self
            .frames
            .pop()
            .ok_or(ArtifactError::UnfinishedOwner { node: id })?;
        self.close(frame);
        Ok(id)
    }

    fn branch(
        &mut self,
        node: NodeId,
        condition: Option<ExprRef<'a>>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<(), ArtifactError> {
        let frame = self
            .frames
            .last()
            .ok_or(ArtifactError::UnfinishedOwner { node })?;
        let Owner::Conditional { branches } = &frame.owner else {
            return Err(ArtifactError::UnfinishedOwner { node: frame.id });
        };
        if frame.id != node {
            return Err(ArtifactError::UnfinishedOwner { node: frame.id });
        }
        let shape = match branches.last() {
            None if condition.is_none() => Some(IfShape::LeadingElse),
            Some(last) if last.condition.is_none() => Some(IfShape::NonTrailingElse),
            _ => None,
        };
        if let Some(shape) = shape {
            return Err(ArtifactError::InvalidIf { node, shape });
        }
        check::owned_span(self.parts.source, node, span, Some(frame.span))?;
        if let Some(last) = branches.last()
            && span.start < last.span.end
        {
            return Err(ArtifactError::OutsideOwner {
                node,
                span,
                owner: frame.span,
            });
        }
        if let Some(condition) = condition {
            check::expression(self.parts.source, node, condition, span)?;
        }
        self.frames.push(Frame {
            id: node,
            owner: Owner::Branch { condition },
            span,
            bindings: vize_l0::Vec::new_in(&self.allocator),
            children_started: false,
            ops: vize_l0::Vec::new_in(&self.allocator),
        });
        let depth = self.frames.len();
        children(&mut RegionBuilder { builder: self }, node);
        if self.frames.len() != depth
            || !matches!(
                self.frames.last(),
                Some(Frame {
                    owner: Owner::Branch { .. },
                    ..
                })
            )
        {
            return Err(ArtifactError::UnfinishedOwner { node });
        }
        let frame = self
            .frames
            .pop()
            .ok_or(ArtifactError::UnfinishedOwner { node })?;
        self.close(frame);
        Ok(())
    }

    /// A closed branch joins its conditional; it is not an op of its own.
    pub(super) fn close_branch(
        &mut self,
        condition: Option<ExprRef<'a>>,
        region: Region<'a>,
        span: Span,
    ) {
        if let Some(Frame {
            owner: Owner::Conditional { branches },
            ..
        }) = self.frames.last_mut()
        {
            branches.push(IfBranch {
                condition,
                region,
                span,
            });
        }
    }
}
