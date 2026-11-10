//! `isConstant` / `isLogicalIdentity` of ESLint's `ast-utils`, over OXC.
//!
//! ESTree has no parenthesized node, so parentheses are looked through; a
//! TypeScript wrapper such as `true as boolean` is not, as in the core rule.
//! Template expressions have no resolvable global scope, so `undefined` and
//! `Boolean(...)` are never constant, matching eslint-plugin-vue.

use super::{CoreKind, visitor::CoreVisitor};
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_span::GetSpan;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator};

/// Loops report at exit, unless a `yield` of the same function ran inside.
impl CoreVisitor<'_, '_> {
    pub(super) fn enter_loop(&mut self, test: Option<&Expression<'_>>) {
        if self.kind == CoreKind::NoConstantCondition {
            self.pending_loops
                .push(test.is_some_and(|test| is_constant(test, true)));
        }
    }

    pub(super) fn exit_loop(&mut self, test: Option<&Expression<'_>>) {
        if self.kind == CoreKind::NoConstantCondition
            && self.pending_loops.pop() == Some(true)
            && let Some(test) = test
        {
            self.report(test.without_parentheses().span());
        }
    }
}

/// Whether the truthiness of `expression` is fixed. Outside a boolean
/// position, whether its string and number coercions are fixed too.
pub(super) fn is_constant(expression: &Expression<'_>, in_boolean_position: bool) -> bool {
    match expression.without_parentheses() {
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::ObjectExpression(_) => true,
        Expression::TemplateLiteral(template) => {
            in_boolean_position
                && template.quasis.iter().any(|quasi| {
                    quasi
                        .value
                        .cooked
                        .as_ref()
                        .is_some_and(|cooked| !cooked.is_empty())
                })
                || template
                    .expressions
                    .iter()
                    .all(|expression| is_constant(expression, false))
        }
        Expression::ArrayExpression(array) => {
            in_boolean_position
                || array.elements.iter().all(|element| match element {
                    ArrayExpressionElement::Elision(_) => true,
                    ArrayExpressionElement::SpreadElement(spread) => {
                        is_constant(&spread.argument, false)
                    }
                    element => element
                        .as_expression()
                        .is_some_and(|element| is_constant(element, false)),
                })
        }
        Expression::UnaryExpression(unary) => match unary.operator {
            UnaryOperator::Void => true,
            UnaryOperator::Typeof if in_boolean_position => true,
            UnaryOperator::LogicalNot => is_constant(&unary.argument, true),
            _ => is_constant(&unary.argument, false),
        },
        Expression::BinaryExpression(binary) => {
            binary.operator != BinaryOperator::In
                && is_constant(&binary.left, false)
                && is_constant(&binary.right, false)
        }
        Expression::LogicalExpression(logical) => {
            let left = is_constant(&logical.left, in_boolean_position);
            let right = is_constant(&logical.right, in_boolean_position);
            left && right
                || left && is_logical_identity(&logical.left, logical.operator)
                || in_boolean_position
                    && right
                    && is_logical_identity(&logical.right, logical.operator)
        }
        Expression::NewExpression(_) => in_boolean_position,
        Expression::AssignmentExpression(assignment) => match assignment.operator {
            AssignmentOperator::Assign => is_constant(&assignment.right, in_boolean_position),
            AssignmentOperator::LogicalOr if in_boolean_position => {
                is_logical_identity(&assignment.right, LogicalOperator::Or)
            }
            AssignmentOperator::LogicalAnd if in_boolean_position => {
                is_logical_identity(&assignment.right, LogicalOperator::And)
            }
            _ => false,
        },
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .is_some_and(|last| is_constant(last, in_boolean_position)),
        _ => false,
    }
}

/// Whether `expression` short-circuits `operator` (`true` for `||`, `false`
/// or `void` for `&&`).
fn is_logical_identity(expression: &Expression<'_>, operator: LogicalOperator) -> bool {
    match expression.without_parentheses() {
        Expression::LogicalExpression(logical) => {
            logical.operator == operator
                && (is_logical_identity(&logical.left, operator)
                    || is_logical_identity(&logical.right, operator))
        }
        Expression::AssignmentExpression(assignment) => {
            let inner = match assignment.operator {
                AssignmentOperator::LogicalOr => LogicalOperator::Or,
                AssignmentOperator::LogicalAnd => LogicalOperator::And,
                _ => return false,
            };
            inner == operator && is_logical_identity(&assignment.right, operator)
        }
        Expression::UnaryExpression(unary) => {
            operator == LogicalOperator::And && unary.operator == UnaryOperator::Void
        }
        expression => match (operator, literal_truthiness(expression)) {
            (LogicalOperator::Or, Some(truthy)) => truthy,
            (LogicalOperator::And, Some(truthy)) => !truthy,
            _ => false,
        },
    }
}

/// `getBooleanValue` of a literal.
fn literal_truthiness(expression: &Expression<'_>) -> Option<bool> {
    Some(match expression {
        Expression::BooleanLiteral(literal) => literal.value,
        Expression::NullLiteral(_) => false,
        Expression::NumericLiteral(literal) => literal.value != 0.0 && !literal.value.is_nan(),
        Expression::StringLiteral(literal) => !literal.value.is_empty(),
        Expression::BigIntLiteral(literal) => !literal.value.bytes().all(|byte| byte == b'0'),
        Expression::RegExpLiteral(_) => true,
        _ => return None,
    })
}
