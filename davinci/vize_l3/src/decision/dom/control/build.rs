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
                matches!(attribute.name, "style" | "key" | "ref")));
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

#[cfg(test)]
mod tests {
    extern crate std;

    use super::ConditionalFrame;
    use crate::decision::DecisionBuildError;
    use alloc::vec::Vec as Owned;
    use vize_l0::{Allocator, Box, Span, Vec, id::NodeId, side_table::SideTable};
    use vize_l2::{
        artifact::{Artifact, ArtifactParts},
        expr::{ExprRef, JsExpr},
        op::{ElementOp, IfBranch, IfOp, Namespace, Op, Region},
        walk::{NodeEvent, NodeRef},
    };

    fn fixture(a: &Allocator, first_roots: usize) -> Artifact<'_> {
        let source = "ok";
        let span = Span::new(0, 2);
        let condition = ExprRef::Js(JsExpr::parse_in(a, source, span).unwrap());
        let mut branches = Vec::new_in(&a);
        for (index, condition) in [Some(condition), None].into_iter().enumerate() {
            let mut ops = Vec::new_in(&a);
            for _ in 0..if index == 0 { first_roots } else { 1 } {
                ops.push(Op::Element(Box::new_in(
                    ElementOp {
                        tag: "p",
                        namespace: Namespace::Html,
                        attributes: Vec::new_in(&a),
                        bindings: Vec::new_in(&a),
                        children: Region {
                            ops: Vec::new_in(&a),
                        },
                        span,
                    },
                    &a,
                )));
            }
            branches.push(IfBranch {
                condition,
                region: Region { ops },
                span,
            });
        }
        let mut ops = Vec::new_in(&a);
        ops.push(Op::If(Box::new_in(IfOp { branches, span }, &a)));
        // This checks canonical event ownership, not native directive admission.
        Artifact::try_new(ArtifactParts {
            source,
            root: Region { ops },
            provenance: Owned::new(),
            scopes: SideTable::new(),
        })
        .unwrap()
    }

    #[test]
    fn incomplete_direct_children_cannot_publish_a_completed_control() {
        let a = Allocator::default();
        let artifact = fixture(&a, 1);
        let Op::If(owner) = artifact.root().ops.first().unwrap() else {
            panic!("If")
        };
        let mut frame = ConditionalFrame::new(owner, true);
        let mut first = None;
        artifact
            .visit_events(&mut |event| {
                if first.is_none()
                    && let NodeEvent::Enter {
                        id,
                        node: NodeRef::Op(op),
                        parent: Some(parent),
                        owner_span,
                    } = event
                    && parent == NodeId::FIRST
                {
                    first = Some(frame.child(id, op, owner_span));
                }
            })
            .unwrap();
        assert!(first.unwrap().unwrap());
        assert!(matches!(frame.finish(NodeId::FIRST),
        Err(DecisionBuildError::InvalidTraversal { node }) if node == NodeId::FIRST));
    }

    #[test]
    fn wrong_header_owner_span_is_a_typed_invariant_error() {
        let a = Allocator::default();
        let artifact = fixture(&a, 1);
        let Op::If(owner) = artifact.root().ops.first().unwrap() else {
            panic!("If")
        };
        let mut frame = ConditionalFrame::new(owner, true);
        let mut error = None;
        artifact
            .visit_events(&mut |event| {
                if error.is_none()
                    && let NodeEvent::Enter {
                        id,
                        node: NodeRef::Op(op),
                        parent: Some(parent),
                        ..
                    } = event
                    && parent == NodeId::FIRST
                {
                    error = Some(frame.child(id, op, Some(Span::new(0, 1))));
                }
            })
            .unwrap();
        assert!(
            matches!(error.unwrap(), Err(DecisionBuildError::InvalidTraversal { node })
        if node.index() == 1)
        );
    }

    #[test]
    fn skipped_empty_header_does_not_consume_the_next_actual_root() {
        let a = Allocator::default();
        let artifact = fixture(&a, 0);
        let Op::If(owner) = artifact.root().ops.first().unwrap() else {
            panic!("If")
        };
        let mut frame = ConditionalFrame::new(owner, false);
        let mut observed = Owned::new();
        artifact
            .visit_events(&mut |event| {
                if let NodeEvent::Enter {
                    id,
                    node: NodeRef::Op(op),
                    parent: Some(parent),
                    owner_span,
                } = event
                    && parent == NodeId::FIRST
                {
                    observed.push(frame.child(id, op, owner_span));
                }
            })
            .unwrap();
        assert_eq!(observed.len(), 1);
        assert!(observed.first().unwrap().is_ok());
        assert!(frame.finish(NodeId::FIRST).unwrap().is_none());
    }
}
