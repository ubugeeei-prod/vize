//! Expression scoring over the retained AST: rules `logical`,
//! `conditional` and `unknown` of `complexity-metrics.md`.
//!
//! A position whose [`ExprRef`] is not [`ExprRef::Js`] has no AST to count
//! over; it adds nothing and is recorded as one `unknown` row, so a
//! consumer can tell "simple" from "not analysed" (pessimal law 1 of the
//! opaque escape: nothing may be concluded from its text).

use oxc_ast::ast::{ConditionalExpression, Expression, LogicalExpression, LogicalOperator};
use oxc_ast_visit::{Visit, walk};
use vize_davinci::id::NodeId;
use vize_s0::Span;
use vize_s2::expr::ExprRef;

use super::{ComplexityFacts, Contribution, DecisionKind};

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
            let mut scorer = Scorer {
                base: js.span.start,
                op,
                depth,
                conditionals: 0,
                facts,
            };
            scorer.visit_expression(js.ast);
        }
        ExprRef::Opaque(_) | ExprRef::Foreign(_) | ExprRef::Filter(_) => {
            facts.push(Contribution {
                span: expr.span(),
                kind: DecisionKind::Unknown,
                op,
                nesting: depth,
                cyclomatic: 0,
                cognitive: 0,
            });
        }
    }
}

/// The expression walk. `conditionals` counts the `?:` enclosing the
/// current node inside this expression — the expression-internal half of
/// the nesting a `?:` pays.
struct Scorer<'f> {
    /// File offset of the expression text: the retained AST's spans are
    /// relative to that text (`JsExpr`'s docs).
    base: u32,
    op: Option<NodeId>,
    depth: u32,
    conditionals: u32,
    facts: &'f mut ComplexityFacts,
}

/// One maximal operator tree, flattened in source order.
struct Run {
    operators: u32,
    runs: u32,
    last: Option<LogicalOperator>,
}

impl Scorer<'_> {
    fn file_span(&self, span: oxc_span::Span) -> Span {
        Span::new(
            self.base.saturating_add(span.start),
            self.base.saturating_add(span.end),
        )
    }

    fn nesting(&self) -> u32 {
        self.depth.saturating_add(self.conditionals)
    }

    /// In-order flattening: left operand's operators, this operator, then
    /// the right operand's. Parentheses are transparent; any other node
    /// ends the tree and is scored as an ordinary sub-expression.
    fn flatten<'a>(&mut self, logical: &LogicalExpression<'a>, run: &mut Run) {
        self.operand(&logical.left, run);
        run.operators = run.operators.saturating_add(1);
        if run.last != Some(logical.operator) {
            run.runs = run.runs.saturating_add(1);
            run.last = Some(logical.operator);
        }
        self.operand(&logical.right, run);
    }

    fn operand<'a>(&mut self, operand: &Expression<'a>, run: &mut Run) {
        match operand.without_parentheses() {
            Expression::LogicalExpression(inner) => {
                vize_s0::ensure_sufficient_stack(|| self.flatten(inner, run));
            }
            _ => self.visit_expression(operand),
        }
    }
}

impl<'a> Visit<'a> for Scorer<'_> {
    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        // This node is a tree root: a logical operand behind parentheses
        // is flattened by `operand` and never reaches this hook.
        let index = self.facts.contributions.len();
        self.facts.push(Contribution {
            span: self.file_span(it.span),
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
        let nesting = self.nesting();
        self.facts.push(Contribution {
            span: self.file_span(it.span),
            kind: DecisionKind::Conditional,
            op: self.op,
            nesting,
            cyclomatic: 1,
            cognitive: 1 + nesting,
        });
        self.conditionals += 1;
        vize_s0::ensure_sufficient_stack(|| walk::walk_conditional_expression(self, it));
        self.conditionals -= 1;
    }
}
