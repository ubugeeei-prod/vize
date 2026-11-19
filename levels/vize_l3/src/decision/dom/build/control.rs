//! Conditional admission without revisiting any branch body.

use super::{DomBuilder, DomExpressionFacts, DomUnsupported, ValueKind};
use crate::decision::dom::vue::policy::FileReads;
use crate::decision::{DecisionBuildError, dom::control::build::ConditionalFrame};
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    expr::ExprRef,
    op::{IfOp, Op},
    resolution::Usage,
};

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
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
        let mut first_condition = ValueKind::ContextDependent;
        for (index, branch) in owner.branches.iter().enumerate() {
            if branch.region.ops.len() != 1 {
                admitted = false;
                self.reject(id, branch.span, DomUnsupported::ConditionalRoot);
            }
            if let Some(expression) = branch.condition {
                let value = self.value(id, expression);
                if value == Some(ValueKind::ContextDependent) {
                    continue;
                }
                if index == 0
                    && let Some(value) = value
                    && self.native_condition(id, owner, expression, value)
                {
                    first_condition = value;
                } else {
                    admitted = false;
                    self.reject(id, expression.span(), DomUnsupported::ConditionalCondition);
                }
            }
        }
        Some(if first_condition == ValueKind::ContextDependent {
            ConditionalFrame::new(owner, admitted)
        } else {
            ConditionalFrame::with_condition(owner, admitted, first_condition)
        })
    }

    fn native_condition(
        &self,
        id: NodeId,
        owner: &IfOp<'arena>,
        expression: ExprRef<'arena>,
        value: ValueKind,
    ) -> bool {
        use crate::decision::dom::vue::VueReadKind;
        if !R::NATIVE_SETUP
            || !matches!(value, ValueKind::FileDependent | ValueKind::LiteralConstant)
        {
            return false;
        }
        let Some(file) = self.file else { return false };
        let Some(record) = file.native_conditional(id).filter(|row| row.accepts(owner)) else {
            return false;
        };
        let original_branches = match (owner.branches.as_slice(), record.else_input()) {
            ([_], None) => true,
            ([first, last], Some(otherwise)) => {
                last.condition.is_none()
                    && last.span == otherwise.operand().element_span()
                    && first.span.start == owner.span.start
                    && first.span.end == last.span.start
                    && last.span.end == owner.span.end
                    && matches!(last.region.ops.as_slice(), [Op::Element(element)]
                        if otherwise.body_node().is_some_and(|node|
                            otherwise.accepts_body(node, element)))
            }
            _ => false,
        };
        if !original_branches {
            return false;
        }
        let ExprRef::Js(expression) = expression else {
            return false;
        };
        let Some(row) = self.facts.vue_expressions.get(id) else {
            return false;
        };
        let [read] = row.reads() else { return false };
        let resolution = row.resolution();
        record.input().matches_expression(expression)
            && core::ptr::eq(resolution.file(), file)
            && resolution.node() == id
            && resolution.scope() == Some(record.scope())
            && row.value() == value
            && record.binding() == Some(read.binding().id())
            && read.occurrence().usage == Usage::Read
            && matches!(read.kind(), VueReadKind::SetupConst | VueReadKind::SetupLet)
            && self.reads.classify(read.occurrence(), read.binding()) == Some(read.kind())
            && read.binding().declaration().is_some_and(|declaration| {
                declaration.scope == record.scope()
                    && declaration.is_direct_program()
                    && declaration.boolean_initializer().is_some()
            })
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
