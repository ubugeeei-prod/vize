use alloc::vec::Vec;
use oxc_ast::ast::{
    Argument, CallExpression, ChainElement, Expression, IdentifierReference, SimpleAssignmentTarget,
};
use oxc_span::GetSpan;
use vize_l0::Span;

use crate::expr::JsExpr;

mod compound;

use super::{BindingLookup, Occurrence, ResolutionError, ResolutionErrorKind, Usage};

pub(super) fn resolve<'a>(
    expression: &JsExpr<'a>,
    bindings: &impl BindingLookup,
) -> Result<Vec<Occurrence<'a>>, ResolutionError> {
    let mut resolver = Resolver {
        expression: *expression,
        bindings,
        occurrences: Vec::new(),
        visited: 0,
        in_new_callee: false,
    };
    resolver.expression(expression.ast, 0)?;
    Ok(resolver.occurrences)
}

struct Resolver<'a, 'b, B> {
    expression: JsExpr<'a>,
    bindings: &'b B,
    occurrences: Vec<Occurrence<'a>>,
    visited: usize,
    in_new_callee: bool,
}

impl<'a, B: BindingLookup> Resolver<'a, '_, B> {
    fn fail(&self, span: oxc_span::Span, kind: ResolutionErrorKind) -> ResolutionError {
        ResolutionError {
            span: self
                .expression
                .ast_span_to_source(span)
                .unwrap_or(Span::new(0, self.expression.source.len() as u32)),
            kind,
        }
    }

    fn visit(&mut self, span: oxc_span::Span, depth: usize) -> Result<Span, ResolutionError> {
        // Bound retained-tree recursion/work independently of parser admission.
        self.visited += 1;
        if depth > 64 || self.visited > 4096 {
            return Err(self.fail(span, ResolutionErrorKind::TraversalLimit));
        }
        self.expression
            .ast_span_to_source(span)
            .ok_or_else(|| self.fail(span, ResolutionErrorKind::InvalidSpan))
    }

    fn identifier(
        &mut self,
        identifier: &'a IdentifierReference<'a>,
        usage: Usage,
        shorthand: bool,
    ) -> Result<(), ResolutionError> {
        let ast_span = identifier.span;
        let span = self.visit(ast_span, 0)?;
        if span.start == span.end
            || self
                .occurrences
                .last()
                .is_some_and(|last| last.span.end > span.start)
        {
            return Err(self.fail(ast_span, ResolutionErrorKind::InvalidSpan));
        }
        let name = identifier.name.as_str();
        let binding = self
            .bindings
            .lookup(name)
            .ok_or_else(|| self.fail(ast_span, ResolutionErrorKind::MissingBinding))?;
        self.occurrences.push(Occurrence {
            span: Span::new(span.start, span.end),
            name,
            binding,
            usage,
            shorthand,
            constructor: self.in_new_callee,
        });
        Ok(())
    }

    fn expression(
        &mut self,
        expression: &'a Expression<'a>,
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

    fn call(&mut self, value: &'a CallExpression<'a>, next: usize) -> Result<(), ResolutionError> {
        if value.type_arguments.is_some()
            || matches!(value.callee.get_inner_expression(), Expression::Identifier(id) if id.name == "eval")
        {
            return Err(self.fail(value.span, ResolutionErrorKind::UnsupportedSyntax));
        }
        self.expression(&value.callee, next)?;
        for argument in &value.arguments {
            self.argument(argument, next)?;
        }
        Ok(())
    }

    fn argument(&mut self, argument: &'a Argument<'a>, next: usize) -> Result<(), ResolutionError> {
        self.visit(argument.span(), next)?;
        match argument {
            Argument::SpreadElement(spread) => self.expression(&spread.argument, next),
            other => self.expression(other.to_expression(), next),
        }
    }

    fn target(
        &mut self,
        target: &'a SimpleAssignmentTarget<'a>,
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
