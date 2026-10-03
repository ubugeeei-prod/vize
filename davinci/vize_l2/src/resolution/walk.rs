use oxc_ast::ast::{
    Argument, CallExpression, ChainElement, Expression, IdentifierReference, SimpleAssignmentTarget,
};
use oxc_span::GetSpan;
use vize_l0::Span;

use crate::expr::JsExpr;

mod compound;

use super::sink::{ReferenceEvent, ReferenceSink};
use super::source::ReferenceSource;
use super::{ResolutionError, ResolutionErrorKind, Usage};

pub(super) fn expression<'a>(
    expression: &JsExpr<'a>,
    sink: &mut impl ReferenceSink<'a>,
) -> Result<(), ResolutionError> {
    retained(
        ReferenceSource::Expression(*expression),
        expression.ast,
        sink,
    )
}

pub(super) fn retained<'a>(
    source: ReferenceSource<'a>,
    expression: &Expression<'a>,
    sink: &mut impl ReferenceSink<'a>,
) -> Result<(), ResolutionError> {
    let checkpoint = sink.checkpoint();
    let mut resolver = Resolver::new(source, sink);
    let result = resolver.expression(expression, 0);
    if result.is_err() {
        resolver.sink.rollback(checkpoint);
    }
    result
}

pub(super) fn export_local<'a>(
    source: ReferenceSource<'a>,
    span: oxc_span::Span,
    name: &'a str,
    sink: &mut impl ReferenceSink<'a>,
) -> Result<(), ResolutionError> {
    let checkpoint = sink.checkpoint();
    let mut resolver = Resolver::new(source, sink);
    let result = resolver.reference(span, name, Usage::Read, false);
    if result.is_err() {
        resolver.sink.rollback(checkpoint);
    }
    result
}

struct Resolver<'a, 'b, S> {
    source: ReferenceSource<'a>,
    sink: &'b mut S,
    last_end: Option<u32>,
    visited: usize,
    in_new_callee: bool,
}

impl<'a, 'b, S: ReferenceSink<'a>> Resolver<'a, 'b, S> {
    fn new(source: ReferenceSource<'a>, sink: &'b mut S) -> Self {
        Self {
            source,
            sink,
            last_end: None,
            visited: 0,
            in_new_callee: false,
        }
    }

    fn fail(&self, span: oxc_span::Span, kind: ResolutionErrorKind) -> ResolutionError {
        ResolutionError {
            span: self
                .source
                .span(span)
                .unwrap_or(Span::new(0, self.source.length())),
            kind,
        }
    }

    fn visit(&mut self, span: oxc_span::Span, depth: usize) -> Result<Span, ResolutionError> {
        // Bound retained-tree recursion/work independently of parser admission.
        self.visited += 1;
        if depth > 64 || self.visited > 4096 {
            return Err(self.fail(span, ResolutionErrorKind::TraversalLimit));
        }
        self.source
            .span(span)
            .ok_or_else(|| self.fail(span, ResolutionErrorKind::InvalidSpan))
    }

    fn identifier(
        &mut self,
        identifier: &IdentifierReference<'a>,
        usage: Usage,
        shorthand: bool,
    ) -> Result<(), ResolutionError> {
        self.reference(identifier.span, identifier.name.as_str(), usage, shorthand)
    }

    fn reference(
        &mut self,
        ast_span: oxc_span::Span,
        name: &'a str,
        usage: Usage,
        shorthand: bool,
    ) -> Result<(), ResolutionError> {
        let span = self.visit(ast_span, 0)?;
        if span.start == span.end || self.last_end.is_some_and(|last| last > span.start) {
            return Err(self.fail(ast_span, ResolutionErrorKind::InvalidSpan));
        }
        self.sink
            .reference(ReferenceEvent::new(
                span,
                name,
                usage,
                shorthand,
                self.in_new_callee,
            ))
            .map_err(|kind| self.fail(ast_span, kind))?;
        self.last_end = Some(span.end);
        Ok(())
    }

    fn expression(
        &mut self,
        expression: &Expression<'a>,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        self.visit(expression.span(), depth)?;
        let next = depth + 1;
        match expression {
            Expression::Identifier(identifier) => self.identifier(identifier, Usage::Read, false),
            Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::StringLiteral(_) => Ok(()),
            Expression::TemplateLiteral(value) => {
                for expression in &value.expressions {
                    self.expression(expression, next)?;
                }
                Ok(())
            }
            Expression::TaggedTemplateExpression(value) if value.type_arguments.is_none() => {
                self.invocation(expression)?;
                self.expression(&value.tag, next)?;
                for expression in &value.quasi.expressions {
                    self.expression(expression, next)?;
                }
                Ok(())
            }
            Expression::ParenthesizedExpression(value) => self.expression(&value.expression, next),
            Expression::UnaryExpression(value) if !value.operator.is_delete() => {
                self.expression(&value.argument, next)
            }
            Expression::BinaryExpression(value) => {
                self.expression(&value.left, next)?;
                self.expression(&value.right, next)
            }
            Expression::LogicalExpression(value) => {
                self.expression(&value.left, next)?;
                self.expression(&value.right, next)
            }
            Expression::ConditionalExpression(value) => {
                self.expression(&value.test, next)?;
                self.expression(&value.consequent, next)?;
                self.expression(&value.alternate, next)
            }
            Expression::SequenceExpression(value) => {
                for expression in &value.expressions {
                    self.expression(expression, next)?;
                }
                Ok(())
            }
            Expression::StaticMemberExpression(value) => self.expression(&value.object, next),
            Expression::ComputedMemberExpression(value) => {
                self.expression(&value.object, next)?;
                self.expression(&value.expression, next)
            }
            Expression::CallExpression(value) => self.call(value, next),
            Expression::NewExpression(value) if value.type_arguments.is_none() => {
                self.invocation(expression)?;
                let previous = self.in_new_callee;
                self.in_new_callee = true;
                let result = self.expression(&value.callee, next);
                self.in_new_callee = previous;
                result?;
                for argument in &value.arguments {
                    self.argument(argument, next)?;
                }
                Ok(())
            }
            Expression::ChainExpression(value) => match &value.expression {
                ChainElement::CallExpression(call) => self.call(call, next),
                ChainElement::StaticMemberExpression(member) => {
                    self.expression(&member.object, next)
                }
                ChainElement::ComputedMemberExpression(member) => {
                    self.expression(&member.object, next)?;
                    self.expression(&member.expression, next)
                }
                other => Err(self.fail(other.span(), ResolutionErrorKind::UnsupportedSyntax)),
            },
            Expression::ImportExpression(value) if value.phase.is_none() => {
                self.invocation(expression)?;
                self.expression(&value.source, next)?;
                if let Some(options) = &value.options {
                    self.expression(options, next)?;
                }
                Ok(())
            }
            Expression::ArrayExpression(value) => self.array(value, next),
            Expression::ObjectExpression(value) => self.object(value, next),
            Expression::AssignmentExpression(value) => {
                let Some(target) = value.left.as_simple_assignment_target() else {
                    return Err(
                        self.fail(value.left.span(), ResolutionErrorKind::UnsupportedSyntax)
                    );
                };
                let usage = if value.operator.is_assign() {
                    Usage::Write
                } else {
                    Usage::ReadWrite
                };
                self.target(target, usage, next)?;
                self.expression(&value.right, next)
            }
            Expression::UpdateExpression(value) => {
                self.target(&value.argument, Usage::ReadWrite, next)
            }
            other => Err(self.fail(other.span(), ResolutionErrorKind::UnsupportedSyntax)),
        }
    }

    fn invocation(&mut self, expression: &Expression<'a>) -> Result<(), ResolutionError> {
        let span = self
            .source
            .span(expression.span())
            .ok_or_else(|| self.fail(expression.span(), ResolutionErrorKind::InvalidSpan))?;
        self.sink
            .observe_invocation(expression, span)
            .map_err(|kind| self.fail(expression.span(), kind))
    }

    fn call(&mut self, value: &CallExpression<'a>, next: usize) -> Result<(), ResolutionError> {
        if value.type_arguments.is_some()
            || matches!(value.callee.get_inner_expression(), Expression::Identifier(id) if id.name == "eval")
        {
            return Err(self.fail(value.span, ResolutionErrorKind::UnsupportedSyntax));
        }
        let span = self
            .source
            .span(value.span)
            .ok_or_else(|| self.fail(value.span, ResolutionErrorKind::InvalidSpan))?;
        self.sink
            .observe_call(value, span)
            .map_err(|kind| self.fail(value.span, kind))?;
        self.expression(&value.callee, next)?;
        for argument in &value.arguments {
            self.argument(argument, next)?;
        }
        Ok(())
    }

    fn argument(&mut self, argument: &Argument<'a>, next: usize) -> Result<(), ResolutionError> {
        self.visit(argument.span(), next)?;
        match argument {
            Argument::SpreadElement(spread) => self.expression(&spread.argument, next),
            other => self.expression(other.to_expression(), next),
        }
    }

    fn target(
        &mut self,
        target: &SimpleAssignmentTarget<'a>,
        usage: Usage,
        depth: usize,
    ) -> Result<(), ResolutionError> {
        self.visit(target.span(), depth)?;
        match target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                self.identifier(identifier, usage, false)
            }
            SimpleAssignmentTarget::StaticMemberExpression(value) if !value.optional => {
                // Assigning obj.x reads obj; it does not assign obj itself.
                self.expression(&value.object, depth + 1)
            }
            SimpleAssignmentTarget::ComputedMemberExpression(value) if !value.optional => {
                self.expression(&value.object, depth + 1)?;
                self.expression(&value.expression, depth + 1)
            }
            other => Err(self.fail(other.span(), ResolutionErrorKind::UnsupportedSyntax)),
        }
    }
}
