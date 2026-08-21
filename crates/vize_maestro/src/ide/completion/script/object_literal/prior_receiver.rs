//! Mutations and identity escapes before a local object's completion site.

use oxc_ast::ast::{
    Argument, AssignmentExpression, CallExpression, Expression, NewExpression, Program,
    ReturnStatement, SimpleAssignmentTarget, UnaryExpression, UpdateExpression, VariableDeclarator,
    YieldExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_assignment_expression, walk_call_expression, walk_new_expression,
        walk_return_statement, walk_unary_expression, walk_update_expression,
        walk_variable_declarator, walk_yield_expression,
    },
};
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use super::receiver_value;

pub(super) fn is_inexact(
    program: &Program<'_>,
    receiver: &str,
    declaration_end: u32,
    cursor_offset: u32,
) -> bool {
    let mut visitor = PriorReceiverInexactness {
        receiver,
        declaration_end,
        cursor_offset,
        found: false,
    };
    visitor.visit_program(program);
    visitor.found
}

struct PriorReceiverInexactness<'s> {
    receiver: &'s str,
    declaration_end: u32,
    cursor_offset: u32,
    found: bool,
}

impl PriorReceiverInexactness<'_> {
    fn is_prior(&self, span: Span) -> bool {
        span.start >= self.declaration_end && span.end <= self.cursor_offset
    }

    fn targets_receiver(&self, target: &SimpleAssignmentTarget<'_>) -> bool {
        target_root_reference(target).is_some_and(|name| name == self.receiver)
    }

    fn is_receiver_value(&self, expression: &Expression<'_>) -> bool {
        receiver_value::carries_receiver(expression, self.receiver)
    }

    fn argument_escapes_receiver(&self, argument: &Argument<'_>) -> bool {
        match argument {
            Argument::SpreadElement(spread) => self.is_receiver_value(&spread.argument),
            argument => argument
                .as_expression()
                .is_some_and(|expression| self.is_receiver_value(expression)),
        }
    }

    fn call_escapes_receiver(&self, callee: &Expression<'_>, arguments: &[Argument<'_>]) -> bool {
        root_reference(callee).is_some_and(|name| name == self.receiver)
            || arguments
                .iter()
                .any(|argument| self.argument_escapes_receiver(argument))
    }
}

impl<'a> Visit<'a> for PriorReceiverInexactness<'_> {
    fn visit_assignment_expression(&mut self, expression: &AssignmentExpression<'a>) {
        if self.is_prior(expression.span)
            && (expression
                .left
                .as_simple_assignment_target()
                .is_some_and(|target| self.targets_receiver(target))
                || self.is_receiver_value(&expression.right))
        {
            self.found = true;
            return;
        }
        walk_assignment_expression(self, expression);
    }

    fn visit_update_expression(&mut self, expression: &UpdateExpression<'a>) {
        if self.is_prior(expression.span) && self.targets_receiver(&expression.argument) {
            self.found = true;
            return;
        }
        walk_update_expression(self, expression);
    }

    fn visit_unary_expression(&mut self, expression: &UnaryExpression<'a>) {
        if expression.operator == UnaryOperator::Delete
            && self.is_prior(expression.span)
            && root_reference(&expression.argument).is_some_and(|name| name == self.receiver)
        {
            self.found = true;
            return;
        }
        walk_unary_expression(self, expression);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if self.is_prior(declarator.span)
            && declarator
                .init
                .as_ref()
                .is_some_and(|init| self.is_receiver_value(init))
        {
            self.found = true;
            return;
        }
        walk_variable_declarator(self, declarator);
    }

    fn visit_call_expression(&mut self, expression: &CallExpression<'a>) {
        if self.is_prior(expression.span)
            && self.call_escapes_receiver(&expression.callee, &expression.arguments)
        {
            self.found = true;
            return;
        }
        walk_call_expression(self, expression);
    }

    fn visit_new_expression(&mut self, expression: &NewExpression<'a>) {
        if self.is_prior(expression.span)
            && self.call_escapes_receiver(&expression.callee, &expression.arguments)
        {
            self.found = true;
            return;
        }
        walk_new_expression(self, expression);
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if self.is_prior(statement.span)
            && statement
                .argument
                .as_ref()
                .is_some_and(|argument| self.is_receiver_value(argument))
        {
            self.found = true;
            return;
        }
        walk_return_statement(self, statement);
    }

    fn visit_yield_expression(&mut self, expression: &YieldExpression<'a>) {
        if self.is_prior(expression.span)
            && expression
                .argument
                .as_ref()
                .is_some_and(|argument| self.is_receiver_value(argument))
        {
            self.found = true;
            return;
        }
        walk_yield_expression(self, expression);
    }
}

fn target_root_reference<'a>(target: &'a SimpleAssignmentTarget<'a>) -> Option<&'a str> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some(identifier.name.as_str())
        }
        other if other.is_member_expression() => {
            root_reference(other.to_member_expression().object())
        }
        other => other.get_expression().and_then(root_reference),
    }
}

fn root_reference<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        expression if expression.is_member_expression() => {
            root_reference(expression.to_member_expression().object())
        }
        _ => None,
    }
}
