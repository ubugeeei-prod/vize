//! Branch header accounting inside the existing DOM event frame.

use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId};
use vize_l2::op::{IfOp, Namespace, Op};

use super::{
    BranchDiscriminant, ConditionalFallback, DomBlockEligibility, DomConditional,
    DomConditionalBranch,
};
use crate::decision::{DecisionBuildError, dom::ValueKind};

pub(in crate::decision::dom) struct ConditionalFrame<'owner, 'arena> {
    owner: &'owner IfOp<'arena>,
    rows: Vec<DomConditionalBranch<'owner, 'arena>>,
    cursor: usize,
    remaining: usize,
    admitted: bool,
}

#[cfg(test)]
mod tests;

impl<'owner, 'arena> ConditionalFrame<'owner, 'arena> {
    pub(in crate::decision::dom) fn new(owner: &'owner IfOp<'arena>, admitted: bool) -> Self {
        Self {
            owner,
            rows: Vec::new(),
            cursor: 0,
            remaining: owner
                .branches
                .first()
                .map_or(0, |branch| branch.region.ops.len()),
            admitted,
        }
    }

    pub(in crate::decision::dom) fn child(
        &mut self,
        id: NodeId,
        op: &'owner Op<'arena>,
        owner_span: Option<Span>,
    ) -> Result<bool, DecisionBuildError> {
        self.skip_finished();
        let branch = self
            .owner
            .branches
            .get(self.cursor)
            .ok_or(DecisionBuildError::InvalidTraversal { node: id })?;
        if owner_span != Some(branch.span) {
            return Err(DecisionBuildError::InvalidTraversal { node: id });
        }
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(DecisionBuildError::InvalidTraversal { node: id })?;
        let eligible = matches!(op, Op::Element(element)
            if element.namespace == Namespace::Html && element.tag != "template"
            && !element.attributes.iter().any(|attribute|
                matches!(attribute.name, "class" | "style" | "key" | "ref")));
        if branch.region.ops.len() != 1 || !eligible {
            self.admitted = false;
        }
        if self.admitted {
            self.rows.push(DomConditionalBranch {
                owner: branch,
                discriminant: BranchDiscriminant(
                    u32::try_from(self.cursor)
                        .map_err(|_| DecisionBuildError::InvalidTraversal { node: id })?,
                ),
                root: id,
                condition: branch.condition.map(|_| ValueKind::ContextDependent),
                block: DomBlockEligibility::NativeElement,
            });
        }
        Ok(self.admitted)
    }

    pub(in crate::decision::dom) fn finish(
        mut self,
        id: NodeId,
    ) -> Result<Option<DomConditional<'owner, 'arena>>, DecisionBuildError> {
        self.skip_finished();
        if self.cursor != self.owner.branches.len() {
            return Err(DecisionBuildError::InvalidTraversal { node: id });
        }
        if !self.admitted || self.rows.len() != self.owner.branches.len() {
            return Ok(None);
        }
        Ok(Some(DomConditional {
            owner: self.owner,
            fallback: if self
                .owner
                .branches
                .last()
                .is_some_and(|branch| branch.condition.is_none())
            {
                ConditionalFallback::AuthoredElse
            } else {
                ConditionalFallback::Placeholder
            },
            branches: self.rows,
        }))
    }

    pub(in crate::decision::dom) fn closes_first_branch(&self, id: NodeId) -> bool {
        self.admitted && self.rows.first().is_some_and(|branch| branch.root == id)
    }

    fn skip_finished(&mut self) {
        while self.remaining == 0 && self.cursor < self.owner.branches.len() {
            self.cursor += 1;
            self.remaining = self
                .owner
                .branches
                .get(self.cursor)
                .map_or(0, |branch| branch.region.ops.len());
        }
    }
}
