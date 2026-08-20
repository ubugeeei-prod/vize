//! Identity-preserving receiver flows through compound expressions.

use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind};

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
        _ => false,
    }
}
