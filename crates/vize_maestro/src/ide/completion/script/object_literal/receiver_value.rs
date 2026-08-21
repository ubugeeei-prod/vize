//! Identity-preserving receiver flows through compound expressions.

use oxc_ast::ast::{
    ArrayExpressionElement, ArrowFunctionExpression, Expression, FunctionBody, ObjectPropertyKind,
    ReturnStatement, Statement, YieldExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_return_statement, walk_yield_expression},
};

/// Whether evaluating `expression` can produce a value that contains the
/// receiver object itself. Member reads intentionally do not qualify: they
/// consume the receiver but do not transfer its identity to another owner.
pub(super) fn carries_receiver(expression: &Expression<'_>, receiver: &str) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => identifier.name == receiver,
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                carries_receiver(&spread.argument, receiver)
            }
            ArrayExpressionElement::Elision(_) => false,
            element => element
                .as_expression()
                .is_some_and(|expression| carries_receiver(expression, receiver)),
        }),
        Expression::ObjectExpression(object) => {
            object.properties.iter().any(|property| match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    carries_receiver(&property.value, receiver)
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    carries_receiver(&spread.argument, receiver)
                }
            })
        }
        Expression::ConditionalExpression(conditional) => {
            carries_receiver(&conditional.consequent, receiver)
                || carries_receiver(&conditional.alternate, receiver)
        }
        Expression::LogicalExpression(logical) => {
            carries_receiver(&logical.left, receiver) || carries_receiver(&logical.right, receiver)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .is_some_and(|expression| carries_receiver(expression, receiver)),
        Expression::AssignmentExpression(assignment) => {
            carries_receiver(&assignment.right, receiver)
        }
        Expression::AwaitExpression(awaited) => carries_receiver(&awaited.argument, receiver),
        Expression::YieldExpression(yielded) => yielded
            .argument
            .as_ref()
            .is_some_and(|argument| carries_receiver(argument, receiver)),
        Expression::ArrowFunctionExpression(arrow) => arrow_returns_receiver(arrow, receiver),
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()
            .is_some_and(|body| body_returns_receiver(body, receiver)),
        _ => false,
    }
}

fn arrow_returns_receiver(arrow: &ArrowFunctionExpression<'_>, receiver: &str) -> bool {
    if arrow.expression {
        return arrow
            .body
            .statements
            .first()
            .and_then(|statement| match statement {
                Statement::ExpressionStatement(statement) => Some(&statement.expression),
                _ => None,
            })
            .is_some_and(|expression| carries_receiver(expression, receiver));
    }
    body_returns_receiver(&arrow.body, receiver)
}

fn body_returns_receiver(body: &FunctionBody<'_>, receiver: &str) -> bool {
    let mut flow = ReturnedReceiver {
        receiver,
        found: false,
    };
    flow.visit_function_body(body);
    flow.found
}

struct ReturnedReceiver<'s> {
    receiver: &'s str,
    found: bool,
}

impl<'a> Visit<'a> for ReturnedReceiver<'_> {
    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if statement
            .argument
            .as_ref()
            .is_some_and(|argument| carries_receiver(argument, self.receiver))
        {
            self.found = true;
            return;
        }
        walk_return_statement(self, statement);
    }

    fn visit_yield_expression(&mut self, expression: &YieldExpression<'a>) {
        if expression
            .argument
            .as_ref()
            .is_some_and(|argument| carries_receiver(argument, self.receiver))
        {
            self.found = true;
            return;
        }
        walk_yield_expression(self, expression);
    }
}
