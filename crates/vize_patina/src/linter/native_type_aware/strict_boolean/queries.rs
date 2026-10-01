//! AST-defined boolean contexts; expressions retain their authored ranges.
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{LogicalOperator, UnaryOperator};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Condition {
    pub start: u32,
    pub end: u32,
    pub anchor: u32,
}

pub(super) struct Collector<'s> {
    pub source: &'s str,
    pub conditions: Vec<Condition>,
}

impl Collector<'_> {
    pub fn condition(&mut self, expression: &Expression<'_>) {
        match expression {
            Expression::ParenthesizedExpression(paren) => self.condition(&paren.expression),
            Expression::LogicalExpression(logical)
                if logical.operator != LogicalOperator::Coalesce =>
            {
                self.condition(&logical.left);
                self.condition(&logical.right);
            }
            Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
                self.condition(&unary.argument)
            }
            Expression::BinaryExpression(binary)
                if binary.operator.is_compare()
                    || binary.operator.is_equality()
                    || binary.operator.is_in()
                    || binary.operator.is_instance_of() => {}
            _ => {
                let span = expression.span();
                if span.end > span.start {
                    self.conditions.push(Condition {
                        start: span.start,
                        end: span.end,
                        anchor: anchor(expression, self.source),
                    });
                }
            }
        }
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_if_statement(&mut self, node: &IfStatement<'a>) {
        self.condition(&node.test);
        walk::walk_if_statement(self, node);
    }
    fn visit_while_statement(&mut self, node: &WhileStatement<'a>) {
        self.condition(&node.test);
        walk::walk_while_statement(self, node);
    }
    fn visit_do_while_statement(&mut self, node: &DoWhileStatement<'a>) {
        self.condition(&node.test);
        walk::walk_do_while_statement(self, node);
    }
    fn visit_for_statement(&mut self, node: &ForStatement<'a>) {
        if let Some(test) = &node.test {
            self.condition(test);
        }
        walk::walk_for_statement(self, node);
    }
    fn visit_conditional_expression(&mut self, node: &ConditionalExpression<'a>) {
        self.condition(&node.test);
        walk::walk_conditional_expression(self, node);
    }
    fn visit_unary_expression(&mut self, node: &UnaryExpression<'a>) {
        if node.operator == UnaryOperator::LogicalNot {
            self.condition(&node.argument);
        }
        walk::walk_unary_expression(self, node);
    }
    fn visit_logical_expression(&mut self, node: &LogicalExpression<'a>) {
        if node.operator != LogicalOperator::Coalesce {
            self.condition(&node.left);
        }
        walk::walk_logical_expression(self, node);
    }
}

fn punctuation(source: &str, range: Span, token: char) -> u32 {
    source
        .get(range.start as usize..range.end as usize)
        .and_then(|text| text.find(token))
        .map_or(range.start, |index| range.start + index as u32)
}

fn member_anchor(member: &StaticMemberExpression<'_>, source: &str) -> u32 {
    punctuation(
        source,
        Span::new(member.object.span().end, member.property.span.start),
        '.',
    )
}

fn anchor(expression: &Expression<'_>, source: &str) -> u32 {
    match expression {
        Expression::StaticMemberExpression(member) => member_anchor(member, source),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => member_anchor(member, source),
            _ => chain.span.end.saturating_sub(1),
        },
        Expression::BinaryExpression(binary) => binary.left.span().end,
        Expression::LogicalExpression(logical) => logical.left.span().end,
        Expression::ConditionalExpression(conditional) => punctuation(
            source,
            Span::new(
                conditional.test.span().end,
                conditional.consequent.span().start,
            ),
            '?',
        ),
        Expression::TSAsExpression(assertion) => assertion.expression.span().end,
        Expression::TSSatisfiesExpression(assertion) => assertion.expression.span().end,
        Expression::UnaryExpression(unary) => unary.span.start,
        Expression::ObjectExpression(object) => object.span.start,
        Expression::ArrayExpression(array) => array.span.start,
        Expression::ParenthesizedExpression(paren) => anchor(&paren.expression, source),
        _ => expression.span().end.saturating_sub(1),
    }
}
