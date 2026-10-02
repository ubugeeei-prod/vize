//! Conditional admission without revisiting any branch body.

use super::{DomBuilder, DomExpressionFacts, DomUnsupported, ValueKind};
use crate::decision::{DecisionBuildError, dom::control::build::ConditionalFrame};
use vize_l0::{Span, id::NodeId};
use vize_l2::op::Op;

impl<'owner, 'arena, F: DomExpressionFacts> DomBuilder<'_, 'owner, 'arena, F> {
    pub(super) fn prepare_conditional(
        &mut self,
        id: NodeId,
        op: &'owner Op<'arena>,
    ) -> Option<ConditionalFrame<'owner, 'arena>> {
        let Op::If(owner) = op else { return None };
        let mut admitted = self.frames.is_empty() && self.root_count == 1;
        if !admitted {
            self.reject(id, owner.span, DomUnsupported::ConditionalRoot);
        }
        let shape = matches!(owner.branches.as_slice(), [first] if first.condition.is_some())
            || matches!(owner.branches.as_slice(), [first, last]
                if first.condition.is_some() && last.condition.is_none());
        if !shape {
            admitted = false;
            self.reject(id, owner.span, DomUnsupported::ConditionalShape);
        }
        for branch in &owner.branches {
            if branch.region.ops.len() != 1 {
                admitted = false;
                self.reject(id, branch.span, DomUnsupported::ConditionalRoot);
            }
            if let Some(expression) = branch.condition
                && self.value(id, expression) != Some(ValueKind::ContextDependent)
            {
                admitted = false;
                self.reject(id, expression.span(), DomUnsupported::ConditionalCondition);
            }
        }
        Some(ConditionalFrame::new(owner, admitted))
    }

    pub(super) fn complete_conditional(
        &mut self,
        id: NodeId,
        span: Span,
        frame: ConditionalFrame<'owner, 'arena>,
    ) -> Result<(), DecisionBuildError> {
        if let Some(conditional) = frame.finish(id)? {
            self.facts.controls.insert(id, conditional);
        } else if !self.facts.unsupported.iter().any(|row| row.node == id) {
            self.reject(id, span, DomUnsupported::ConditionalRoot);
        }
        Ok(())
    }
}
