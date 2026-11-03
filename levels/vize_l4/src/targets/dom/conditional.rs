//! Conditional syntax from checked branch identities and canonical root facts.

use vize_l0::id::NodeId;
use vize_l2::{
    expr::ExprRef,
    op::{IfOp, Op},
};
use vize_l3::decision::dom::ValueKind;
use vize_l3::decision::dom::control::{
    ConditionalFallback, DomBlockEligibility, DomConditionalBranch,
};

use super::write::helper;
use super::{DomError, DomErrorKind, DomMode, Emitter, ExpressionWriter, LinkSink};

impl<'owner, 'arena, E: ExpressionWriter, L: LinkSink> Emitter<'_, 'owner, 'arena, E, L> {
    pub(super) fn conditional(&mut self, id: NodeId, owner: &IfOp<'arena>) -> Result<(), DomError> {
        let conditional = self
            .facts
            .conditional(id)
            .ok_or_else(|| self.error(id, DomErrorKind::MissingNode))?;
        if !core::ptr::eq(conditional.owner(), owner) {
            return Err(self.error(id, DomErrorKind::InvalidGrouping));
        }
        let (first, last) = match conditional.branches() {
            [first] if conditional.fallback() == ConditionalFallback::Placeholder => (first, None),
            [first, last] if conditional.fallback() == ConditionalFallback::AuthoredElse => {
                (first, Some(last))
            }
            _ => return Err(self.error(id, DomErrorKind::InvalidGrouping)),
        };
        let condition = first
            .owner()
            .condition
            .filter(|expression| {
                first
                    .condition()
                    .is_some_and(|value| self.condition_supported(id, owner, *expression, value))
            })
            .ok_or_else(|| self.error(id, DomErrorKind::InvalidGrouping))?;
        self.writer.push("(");
        self.expression(id, condition)?;
        self.writer.push(")");
        self.writer.indent();
        self.writer.newline();
        self.writer.push("? ");
        self.conditional_branch(id, first)?;
        self.writer.newline();
        self.writer.push(": ");
        if let Some(last) = last {
            if last.condition().is_some() || last.owner().condition.is_some() {
                return Err(self.error(id, DomErrorKind::InvalidGrouping));
            }
            self.conditional_branch(id, last)?;
        } else {
            helper(&mut self.writer, self.vocabulary, self.helpers.comment);
            self.writer.push(match self.mode {
                DomMode::Development => "(\"v-if\", true)",
                DomMode::Production => "(\"\", true)",
            });
        }
        self.writer.deindent();
        Ok(())
    }

    fn condition_supported(
        &self,
        id: NodeId,
        owner: &IfOp<'arena>,
        expression: ExprRef<'arena>,
        value: ValueKind,
    ) -> bool {
        if value == ValueKind::ContextDependent {
            return true;
        }
        if !matches!(value, ValueKind::FileDependent | ValueKind::LiteralConstant) {
            return false;
        }
        let ExprRef::Js(expression) = expression else {
            return false;
        };
        let Some(row) = self.facts.file_expression(id) else {
            return false;
        };
        let resolution = row.resolution();
        let Some(record) = resolution.file().native_conditional(id) else {
            return false;
        };
        resolution.node() == id
            && resolution.scope() == Some(record.scope())
            && record.accepts(owner)
            && record.input().matches_expression(expression)
    }

    fn conditional_branch(
        &mut self,
        owner: NodeId,
        branch: &DomConditionalBranch<'owner, 'arena>,
    ) -> Result<(), DomError> {
        let root = branch.root();
        let fact = self
            .facts
            .node(root)
            .ok_or_else(|| self.error(root, DomErrorKind::MissingNode))?;
        let Op::Element(element) = fact.op() else {
            return Err(self.error(root, DomErrorKind::InvalidGrouping));
        };
        if branch.block() != DomBlockEligibility::NativeElement || !fact.block_eligible {
            return Err(self.error(owner, DomErrorKind::InvalidGrouping));
        }
        self.element(root, element, fact, Some(branch.discriminant().index()))
    }
}
