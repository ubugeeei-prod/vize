//! Expression scoring over the retained AST: rules `logical`,
//! `conditional` and `unknown` of `complexity-metrics.md`.
//!
//! A position whose [`ExprRef`] is not [`ExprRef::Js`] has no AST to count
//! over; it adds nothing and is recorded as one `unknown` row, so a
//! consumer can tell "simple" from "not analysed" (pessimal law 1 of the
//! opaque escape: nothing may be concluded from its text).
//! A retained expression whose contribution cannot project exactly through its
//! checked wrapper/decode coordinates is likewise one whole `unknown` position.

use crate::expr::{ExprRef, JsExpr};
use oxc_ast::ast::{ConditionalExpression, Expression, LogicalExpression, LogicalOperator};
use oxc_ast_visit::{Visit, walk};
use vize_l0::Span;
use vize_l0::id::NodeId;

use super::{ComplexityFacts, Contribution, DecisionKind};

#[cfg(test)]
mod tests;

/// Score one evaluated expression position standing at template nesting
/// `depth`, carried by op `op`.
pub(super) fn score(
    expr: ExprRef<'_>,
    op: Option<NodeId>,
    depth: u32,
    facts: &mut ComplexityFacts,
) {
    match expr {
        ExprRef::Js(js) => {
            let previous = (facts.contributions.len(), facts.cyclomatic, facts.cognitive);
            let mut scorer = Scorer {
                js,
                op,
                depth,
                conditionals: 0,
                facts,
                unprojectable: false,
            };
            scorer.visit_expression(js.ast);
            if scorer.unprojectable {
                // A contribution inside an indivisible decoded segment has no
                // exact authored range. Keep no prefix of that expression's
                // scores, and preserve every earlier expression's facts.
                facts.contributions.truncate(previous.0);
                facts.cyclomatic = previous.1;
                facts.cognitive = previous.2;
                unknown(expr, op, depth, facts);
            }
        }
        ExprRef::Opaque(_) | ExprRef::Foreign(_) | ExprRef::Filter(_) => {
            unknown(expr, op, depth, facts);
        }
    }
}

fn unknown(expr: ExprRef<'_>, op: Option<NodeId>, depth: u32, facts: &mut ComplexityFacts) {
    facts.push(Contribution {
        span: expr.span(),
        kind: DecisionKind::Unknown,
        op,
        nesting: depth,
        cyclomatic: 0,
        cognitive: 0,
    });
}

/// The expression walk. `conditionals` counts the `?:` enclosing the
/// current node inside this expression — the expression-internal half of
/// the nesting a `?:` pays.
struct Scorer<'f, 'a> {
    /// Original retained AST, decoded text and checked wrapper/source bridge.
    js: &'a JsExpr<'a>,
    op: Option<NodeId>,
    depth: u32,
    conditionals: u32,
    facts: &'f mut ComplexityFacts,
    unprojectable: bool,
}

/// One maximal operator tree, flattened in source order.
struct Run {
    operators: u32,
    runs: u32,
    last: Option<LogicalOperator>,
}

impl<'a> Scorer<'_, 'a> {
    fn file_span(&mut self, span: oxc_span::Span) -> Option<Span> {
        // Identity load-path AST offsets already address the decoded text.
        // `authored_span` checks that range and its UTF-8 boundaries, so avoid
        // doing the same validation again through `ast_span_to_source`.
        let decoded = if self.js.coordinates.is_none() {
            Some(Span::new(span.start, span.end))
        } else {
            self.js.ast_span_to_source(span)
        };
        let authored = decoded.and_then(|decoded| self.js.authored_span(decoded));
        self.unprojectable |= authored.is_none();
        authored
    }

    fn nesting(&self) -> u32 {
        self.depth.saturating_add(self.conditionals)
    }

    /// In-order flattening: left operand's operators, this operator, then
    /// the right operand's. Parentheses are transparent; any other node
    /// ends the tree and is scored as an ordinary sub-expression.
    fn flatten(&mut self, logical: &LogicalExpression<'a>, run: &mut Run) {
        self.operand(&logical.left, run);
        run.operators = run.operators.saturating_add(1);
        let operator = Some(logical.operator);
        if run.last != operator {
            run.runs = run.runs.saturating_add(1);
            run.last = operator;
        }
        self.operand(&logical.right, run);
    }

    fn operand(&mut self, operand: &Expression<'a>, run: &mut Run) {
        match operand.without_parentheses() {
            Expression::LogicalExpression(inner) => {
                vize_l0::ensure_sufficient_stack(|| self.flatten(inner, run));
            }
            _ => self.visit_expression(operand),
        }
    }
}

impl<'a> Visit<'a> for Scorer<'_, 'a> {
    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        // This node is a tree root: a logical operand behind parentheses
        // is flattened by `operand` and never reaches this hook.
        let Some(span) = self.file_span(it.span) else {
            return;
        };
        let index = self.facts.contributions.len();
        self.facts.push(Contribution {
            span,
            kind: DecisionKind::Logical,
            op: self.op,
            nesting: self.nesting(),
            cyclomatic: 0,
            cognitive: 0,
        });
        let mut run = Run {
            operators: 0,
            runs: 0,
            last: None,
        };
        self.flatten(it, &mut run);
        self.facts.settle(index, run.operators, run.runs);
    }

    fn visit_conditional_expression(&mut self, it: &ConditionalExpression<'a>) {
        let Some(span) = self.file_span(it.span) else {
            return;
        };
        let nesting = self.nesting();
        self.facts.push(Contribution {
            span,
            kind: DecisionKind::Conditional,
            op: self.op,
            nesting,
            cyclomatic: 1,
            cognitive: 1 + nesting,
        });
        self.conditionals += 1;
        vize_l0::ensure_sufficient_stack(|| walk::walk_conditional_expression(self, it));
        self.conditionals -= 1;
    }
}
